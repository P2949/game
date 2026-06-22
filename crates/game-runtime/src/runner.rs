use std::collections::HashSet;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::{Duration, Instant};

#[cfg(feature = "real-backends")]
use anyhow::Context;
use anyhow::Result;
use game_core::app::{Ctx, RenderFrame, StartCtx, extract_entity_sprites, extract_tilemap_sprites};
use game_core::assets::AssetRegistry;
use game_core::audio::{Audio, AudioCommands};
use game_core::backend::{AudioBackend, PlatformBackend, RenderBackend, RenderOutcome};
use game_core::builder::{GameBuilder, MapId, MapRegistry, PrefabRegistry, RuntimeContent};
use game_core::camera::Camera2D;
use game_core::commands::{
    AssetReloadRequest, AssetReloadStatus, Command, CommandQueue, MapReload,
};
use game_core::gfx::Gfx;
use game_core::input::{ActionId, Input, InputRegistry};
use game_core::plugin::GamePlugin;
use game_core::schedule::Schedule;
use game_core::world::World;

#[cfg(feature = "real-backends")]
use game_audio::AudioSystem;
#[cfg(feature = "real-backends")]
use game_core::assets::AssetValidator;
#[cfg(feature = "real-backends")]
use game_core::schedule::ScheduleValidator;
#[cfg(feature = "real-backends")]
use game_platform_sdl::window::Platform;
#[cfg(feature = "real-backends")]
use game_renderer_vulkan::assets::{asset_root, validate_builtin_assets};
#[cfg(feature = "real-backends")]
use game_renderer_vulkan::context::VulkanContext;

use crate::fixed_timestep::FixedTimestep;

const RESIZE_IDLE_SLEEP: Duration = Duration::from_millis(16);
const LAG_WARNING_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Clone, Debug)]
pub struct RuntimeConfig {
    title: String,
    width: u32,
    height: u32,
    sim_hz: f64,
}

impl RuntimeConfig {
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn window_size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn sim_hz(mut self, sim_hz: f64) -> Self {
        self.sim_hz = sim_hz;
        self
    }
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            title: "Arena".to_owned(),
            width: 1280,
            height: 720,
            sim_hz: 120.0,
        }
    }
}

/// The real game loop, parameterized over platform, renderer, and audio
/// implementations. The SDL/Vulkan path and headless tests run this exact type.
pub struct Runner<P, R, A> {
    platform: P,
    renderer: R,
    audio: Option<A>,
    asset_root: PathBuf,
    assets: AssetRegistry,
    input_registry: InputRegistry,
    maps: MapRegistry,
    prefabs: Rc<PrefabRegistry>,
    start_map: MapId,
    schedule: Schedule,
    active_map: ActiveMap,
    timestep: FixedTimestep,
    camera: Camera2D,
    world: World,
    pending_pressed: HashSet<ActionId>,
    last_lag_warning: Option<Instant>,
}

impl<P, R, A> Runner<P, R, A>
where
    P: PlatformBackend,
    R: RenderBackend,
    A: AudioBackend,
{
    /// Builds a running game from validated content and already-created
    /// backends. `asset_root` is used only for explicit F5 asset reloads.
    pub fn new(
        config: RuntimeConfig,
        builder: GameBuilder,
        asset_root: impl Into<PathBuf>,
        platform: P,
        renderer: R,
        audio: Option<A>,
    ) -> Result<Self> {
        let RuntimeContent {
            assets,
            input: input_registry,
            maps,
            prefabs,
            start_map,
            mut schedule,
        } = builder.into_parts()?;
        let active_map = ActiveMap::load(&maps, start_map)?;
        let mut world = World::new();
        schedule.run_startup(&mut StartCtx::new(&mut world))?;

        Ok(Self {
            platform,
            renderer,
            audio,
            asset_root: asset_root.into(),
            assets,
            input_registry,
            maps,
            prefabs,
            start_map,
            schedule,
            active_map,
            timestep: FixedTimestep::new(config.sim_hz),
            camera: Camera2D::new(glam::Vec2::ZERO, 1.0),
            world,
            pending_pressed: HashSet::new(),
            last_lag_warning: None,
        })
    }

    /// Runs exactly one complete runtime frame with a caller-provided elapsed
    /// duration. This is the deterministic entry point for headless tests.
    pub fn step_frame(&mut self, elapsed: Duration) -> Result<RenderOutcome> {
        self.platform.pump_events();
        if self.platform.should_quit() {
            return Ok(RenderOutcome::Skipped);
        }

        let drawable_size = self.platform.drawable_size();
        if drawable_size.x == 0 || drawable_size.y == 0 {
            self.timestep.reset_after_pause();
            return Ok(RenderOutcome::Skipped);
        }

        if self.platform.take_stable_resize_request() {
            self.renderer.request_resize();
        }

        if let Some(audio) = &self.audio {
            audio.poll_diagnostics();
        }

        self.pending_pressed.extend(Input::pressed_this_frame(
            &self.input_registry,
            self.platform.input(),
        ));
        let frame_input = Input::evaluate_continuous(&self.input_registry, self.platform.input());
        let mut frame = RenderFrame::new(self.camera);
        let mut audio_commands = AudioCommands::default();

        self.timestep.advance_by(elapsed);
        let mut steps = 0;
        while steps < FixedTimestep::MAX_STEPS_PER_FRAME {
            let Some(dt) = self.timestep.consume_step() else {
                break;
            };
            let mut input = frame_input.clone();
            if steps == 0 {
                input.set_pressed(std::mem::take(&mut self.pending_pressed));
            }

            {
                let mut ctx = Ctx {
                    world: &mut self.world,
                    map: &self.active_map.data.tilemap,
                    nav: &self.active_map.data.nav,
                    input: &input,
                    camera: &mut self.camera,
                    gfx: Gfx::new(&mut frame),
                    audio: Audio::new(&mut audio_commands),
                };
                self.schedule.run_fixed(&mut ctx, dt);
            }
            if process_core_commands(
                &mut self.world,
                &self.prefabs,
                &mut self.maps,
                self.start_map,
                &mut self.active_map,
                &mut audio_commands,
            ) {
                self.platform.request_quit();
            }
            self.reload_assets_if_requested();
            steps += 1;
        }

        {
            let mut ctx = Ctx {
                world: &mut self.world,
                map: &self.active_map.data.tilemap,
                nav: &self.active_map.data.nav,
                input: &frame_input,
                camera: &mut self.camera,
                gfx: Gfx::new(&mut frame),
                audio: Audio::new(&mut audio_commands),
            };
            let frame_dt = elapsed.as_secs_f32();
            self.schedule.run_update(&mut ctx, frame_dt);
            self.schedule.run_render_extract(&mut ctx, frame_dt);
            self.schedule.run_ui(&mut ctx, frame_dt);
        }
        if process_core_commands(
            &mut self.world,
            &self.prefabs,
            &mut self.maps,
            self.start_map,
            &mut self.active_map,
            &mut audio_commands,
        ) {
            self.platform.request_quit();
        }
        self.reload_assets_if_requested();

        if self.timestep.step_ready() {
            let now = Instant::now();
            if self
                .last_lag_warning
                .is_none_or(|last| now.duration_since(last) >= LAG_WARNING_INTERVAL)
            {
                log::warn!(
                    "fixed timestep hit {} steps in one frame; discarding accumulated lag",
                    FixedTimestep::MAX_STEPS_PER_FRAME
                );
                self.last_lag_warning = Some(now);
            }
            self.timestep.discard_lag();
        }

        frame.camera = self.camera;
        extract_tilemap_sprites(&self.active_map.data, &mut frame);
        extract_entity_sprites(&self.world, &mut frame);
        if let Some(audio) = &self.audio {
            for command in audio_commands.drain() {
                audio.submit(command);
            }
        }

        self.renderer.render(drawable_size, frame)
    }

    /// Runs the interactive loop until the platform requests exit (or the
    /// development smoke-frame limit is reached).
    pub fn run(&mut self) -> Result<()> {
        let smoke_frames = parse_smoke_frames()?;
        if smoke_frames == Some(0) {
            log::info!("GAME_SMOKE_FRAMES=0 requested; initialized and exiting before rendering");
            return Ok(());
        }

        let mut previous_frame = Instant::now();
        let mut rendered_frames = 0_u64;
        while !self.platform.should_quit() {
            let now = Instant::now();
            let elapsed = now.saturating_duration_since(previous_frame);
            previous_frame = now;
            let outcome = self.step_frame(elapsed)?;

            if self.platform.drawable_size() == glam::UVec2::ZERO {
                std::thread::sleep(RESIZE_IDLE_SLEEP);
            }
            if outcome == RenderOutcome::Presented {
                rendered_frames += 1;
            }
            if let Some(limit) = smoke_frames
                && rendered_frames >= limit
            {
                log::info!("GAME_SMOKE_FRAMES={limit} reached; exiting cleanly");
                self.platform.request_quit();
            }
        }
        Ok(())
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    pub fn platform(&self) -> &P {
        &self.platform
    }

    pub fn platform_mut(&mut self) -> &mut P {
        &mut self.platform
    }

    pub fn renderer(&self) -> &R {
        &self.renderer
    }

    pub fn renderer_mut(&mut self) -> &mut R {
        &mut self.renderer
    }

    pub fn audio(&self) -> Option<&A> {
        self.audio.as_ref()
    }

    fn reload_assets_if_requested(&mut self) {
        if self.world.remove_resource::<AssetReloadRequest>().is_none() {
            return;
        }

        match self.renderer.reload_textures(&self.assets.texture_loads()) {
            Ok(texture_count) => {
                let sound_count = match self.audio.as_mut() {
                    Some(audio) => {
                        match audio.reload_sounds(&self.asset_root, &self.assets.sound_loads()) {
                            Ok(count) => count,
                            Err(error) => {
                                let message = first_error_line(&error);
                                log::error!("sound reload failed: {error:#}");
                                self.world
                                    .insert_resource(AssetReloadStatus::failed(message));
                                return;
                            }
                        }
                    }
                    None => 0,
                };
                let message = format!("ok ({texture_count} texture(s), {sound_count} sound(s))");
                log::info!("asset reload: {message}");
                self.world
                    .insert_resource(AssetReloadStatus::succeeded(message));
            }
            Err(error) => {
                let message = first_error_line(&error);
                log::error!("asset reload failed: {error:#}");
                self.world
                    .insert_resource(AssetReloadStatus::failed(message));
            }
        }
    }
}

fn first_error_line(error: &anyhow::Error) -> String {
    error
        .to_string()
        .lines()
        .next()
        .unwrap_or("unknown error")
        .to_owned()
}

/// Runs a content plugin with the production SDL/Vulkan/audio backends.
#[cfg(feature = "real-backends")]
pub fn run<P: GamePlugin>(config: RuntimeConfig, plugin: P) -> Result<()> {
    let mut builder = GameBuilder::new();
    plugin.build(&mut builder)?;
    validate_builder(&builder)?;

    let root = asset_root().context("failed to resolve asset root for runtime")?;
    let platform = Platform::new(&config.title, config.width, config.height)?;
    let renderer = VulkanContext::new(&platform.window, builder.assets().texture_loads())?;
    let audio = match AudioSystem::new(&platform.sdl, &root, builder.assets().sound_loads()) {
        Ok(audio) => Some(audio),
        Err(error) => {
            log::warn!("audio disabled: {error}");
            None
        }
    };
    let mut runner = Runner::new(config, builder, root, platform, renderer, audio)?;
    runner.run()
}

/// Provides a clear error when somebody disables production backends but still
/// calls the windowed entry point. Headless users construct [`Runner`] directly.
#[cfg(not(feature = "real-backends"))]
pub fn run<P: GamePlugin>(_config: RuntimeConfig, _plugin: P) -> Result<()> {
    anyhow::bail!(
        "game-runtime was built without the `real-backends` feature; construct Runner with headless backends instead"
    )
}

/// Validates host-owned content before production backends are created.
#[cfg(feature = "real-backends")]
fn validate_builder(builder: &GameBuilder) -> Result<()> {
    let start_map_ready = builder
        .start_map()
        .is_some_and(|id| builder.maps().get(id).is_some());
    ScheduleValidator::new(builder.schedule())
        .start_map_set(start_map_ready)
        .builtin_render_extract()
        .validate()
        .context("schedule validation failed")?;

    let root = asset_root().context("failed to resolve asset root for validation")?;
    AssetValidator::new(builder.assets())
        .root(&root)
        .validate()
        .context("asset validation failed")?;
    validate_builtin_assets(&root).context("renderer built-in asset validation failed")?;
    Ok(())
}

struct ActiveMap {
    id: MapId,
    data: game_core::app::MapData,
}

impl ActiveMap {
    fn load(maps: &MapRegistry, id: MapId) -> Result<Self> {
        let map = maps
            .get(id)
            .ok_or_else(|| anyhow::anyhow!("map {:?} is not registered", id))?;
        Ok(Self {
            id,
            data: map.data.clone(),
        })
    }

    fn switch_to(&mut self, maps: &MapRegistry, id: MapId) -> Result<()> {
        *self = Self::load(maps, id)?;
        Ok(())
    }
}

fn process_core_commands(
    world: &mut World,
    prefabs: &PrefabRegistry,
    maps: &mut MapRegistry,
    start_map: MapId,
    active_map: &mut ActiveMap,
    audio_commands: &mut AudioCommands,
) -> bool {
    let commands = world
        .get_resource_mut::<CommandQueue>()
        .map(|queue| queue.drain().collect::<Vec<_>>())
        .unwrap_or_default();

    let mut quit = false;
    for command in commands {
        match command {
            Command::Despawn(entity) => world.despawn(entity),
            Command::PlaySound(sound) => {
                audio_commands.push(game_core::backend::AudioCommand::Play {
                    sound,
                    volume: 0.8,
                    looping: false,
                    bus: None,
                })
            }
            Command::SpawnPrefab {
                prefab,
                position,
                properties,
            } => {
                if let Err(error) = prefabs.spawn(prefab, world, position, &properties) {
                    log::error!("failed to spawn prefab command {:?}: {error:?}", prefab);
                }
            }
            Command::ChangeMap(map) => {
                if let Err(error) = active_map.switch_to(maps, map) {
                    log::error!("failed to change active map to {:?}: {error:?}", map);
                }
            }
            Command::Quit => quit = true,
            Command::ReloadMap(map) => {
                let reload = world.remove_resource::<MapReload>();
                match reload {
                    Some(reload) if reload.map == map => {
                        if let Err(error) =
                            maps.replace(map, reload.data.tilemap, reload.data.theme)
                        {
                            log::error!("failed to replace reloaded map {:?}: {error:?}", map);
                        } else if let Err(error) = active_map.switch_to(maps, map) {
                            log::error!("failed to activate reloaded map {:?}: {error:?}", map);
                        }
                    }
                    Some(reload) => log::error!(
                        "discarding reload data for {:?}; command requested {:?}",
                        reload.map,
                        map
                    ),
                    None => log::error!("map reload for {:?} had no replacement data", map),
                }
            }
            Command::ReloadAssets => {
                world.insert_resource(AssetReloadRequest);
                world.insert_resource(AssetReloadStatus::queued());
            }
            Command::RestartMap => {
                let map = active_map.id;
                if let Err(error) = active_map.switch_to(maps, map) {
                    log::error!("failed to restart active map {:?}: {error:?}", map);
                }
            }
            Command::RestartStartMap => {
                if let Err(error) = active_map.switch_to(maps, start_map) {
                    log::error!("failed to restart start map {:?}: {error:?}", start_map);
                }
            }
        }
    }
    quit
}

fn parse_smoke_frames() -> anyhow::Result<Option<u64>> {
    let Ok(raw) = std::env::var("GAME_SMOKE_FRAMES") else {
        return Ok(None);
    };

    raw.trim()
        .parse::<u64>()
        .map(Some)
        .map_err(|_| anyhow::anyhow!("GAME_SMOKE_FRAMES must be a non-negative integer"))
}

#[cfg(test)]
mod tests {
    use super::parse_smoke_frames;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
    }

    #[test]
    fn smoke_frames_unset_means_interactive_run() {
        let _guard = env_lock();
        unsafe { std::env::remove_var("GAME_SMOKE_FRAMES") };
        assert_eq!(parse_smoke_frames().unwrap(), None);
    }

    #[test]
    fn smoke_frames_accepts_zero_and_positive_counts() {
        let _guard = env_lock();
        unsafe { std::env::set_var("GAME_SMOKE_FRAMES", "0") };
        assert_eq!(parse_smoke_frames().unwrap(), Some(0));
        unsafe { std::env::set_var("GAME_SMOKE_FRAMES", "120") };
        assert_eq!(parse_smoke_frames().unwrap(), Some(120));
        unsafe { std::env::remove_var("GAME_SMOKE_FRAMES") };
    }

    #[test]
    fn smoke_frames_trims_whitespace() {
        let _guard = env_lock();
        unsafe { std::env::set_var("GAME_SMOKE_FRAMES", " 120 ") };
        assert_eq!(parse_smoke_frames().unwrap(), Some(120));
        unsafe { std::env::remove_var("GAME_SMOKE_FRAMES") };
    }

    #[test]
    fn smoke_frames_rejects_invalid_values() {
        let _guard = env_lock();
        for value in ["", "-1", "abc", "1.5"] {
            unsafe { std::env::set_var("GAME_SMOKE_FRAMES", value) };
            assert!(parse_smoke_frames().is_err(), "accepted {value:?}");
        }
        unsafe { std::env::remove_var("GAME_SMOKE_FRAMES") };
    }
}

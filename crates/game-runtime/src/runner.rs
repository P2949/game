use std::collections::HashSet;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use game_audio::AudioSystem;
use game_core::app::{Ctx, RenderFrame, StartCtx, extract_entity_sprites, extract_tilemap_sprites};
use game_core::assets::AssetValidator;
use game_core::audio::{Audio, AudioCommands};
use game_core::backend::AudioCommand;
use game_core::builder::{GameBuilder, RuntimeContent};
use game_core::camera::Camera2D;
use game_core::commands::{Command, CommandQueue};
use game_core::gfx::Gfx;
use game_core::input::{ActionId, Input};
use game_core::plugin::GamePlugin;
use game_core::schedule::ScheduleValidator;
use game_core::world::World;
use game_platform_sdl::window::Platform;
use game_renderer_vulkan::assets::{asset_root, validate_builtin_assets};
use game_renderer_vulkan::context::{RenderOutcome as VulkanRenderOutcome, VulkanContext};

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

pub fn run<P: GamePlugin>(config: RuntimeConfig, plugin: P) -> Result<()> {
    let mut builder = GameBuilder::new();
    plugin.build(&mut builder)?;
    validate_builder(&builder)?;
    run_game_inner(config, builder)
}

/// Validates host-owned content (schedule wiring and on-disk assets) before any
/// backend is created or the main loop starts (Phase 11.3/11.4). Map and prefab
/// validation runs earlier, inside `GamePlugin::build`.
fn validate_builder(builder: &GameBuilder) -> Result<()> {
    let start_map_ready = builder
        .start_map()
        .is_some_and(|id| builder.maps().get(id).is_some());
    ScheduleValidator::new(builder.schedule())
        .start_map_set(start_map_ready)
        // The runtime extracts tilemap/entity sprites into every frame itself, so
        // content is not required to register its own render_extract system.
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

fn run_game_inner(config: RuntimeConfig, builder: GameBuilder) -> Result<()> {
    let RuntimeContent {
        assets,
        input: input_registry,
        maps,
        prefabs,
        start_map,
        mut schedule,
    } = builder.into_parts()?;

    let map = maps
        .get(start_map)
        .ok_or_else(|| anyhow::anyhow!("start map {:?} is not registered", start_map))?
        .data
        .clone();
    // Runtime map switching is not wired yet: the loop owns this fixed MapData
    // and its derived nav/render state for the duration of the run.
    let _runtime_maps = maps;
    let _runtime_prefabs = prefabs;

    let smoke_frames = parse_smoke_frames()?;

    let mut platform = Platform::new(&config.title, config.width, config.height)?;
    let mut renderer = VulkanContext::new(&platform.window, assets.texture_loads())?;
    let audio = match AudioSystem::new(&platform.sdl) {
        Ok(audio) => Some(audio),
        Err(err) => {
            log::warn!("audio disabled: {err}");
            None
        }
    };
    let mut timestep = FixedTimestep::new(config.sim_hz);
    let mut camera = Camera2D::new(glam::Vec2::ZERO, 1.0);
    let mut world = World::new();

    {
        let mut start_ctx = StartCtx::new(&mut world);
        schedule.run_startup(&mut start_ctx)?;
    }

    if smoke_frames == Some(0) {
        log::info!("GAME_SMOKE_FRAMES=0 requested; initialized and exiting before rendering");
        return Ok(());
    }

    // Edge-pressed actions accumulate across frames so a key press during a
    // frame that consumed zero fixed steps is delivered to the next step instead
    // of being lost.
    let mut pending_pressed: HashSet<ActionId> = HashSet::new();
    let mut previous_frame = Instant::now();
    let mut last_lag_warning: Option<Instant> = None;
    let mut rendered_frames: u64 = 0;

    while !platform.should_quit {
        platform.pump_events();

        let (width, height) = platform.drawable_size();
        if width == 0 || height == 0 {
            previous_frame = Instant::now();
            timestep.reset_after_pause();
            std::thread::sleep(RESIZE_IDLE_SLEEP);
            continue;
        }

        if platform.take_stable_resize_request() {
            renderer.request_swapchain_recreate();
        }

        let now = Instant::now();
        let frame_ms = (now - previous_frame).as_secs_f32() * 1000.0;
        previous_frame = now;

        if let Some(audio) = &audio {
            audio.poll_dropped_frames();
            audio.poll_dropped_voices();
        }

        // Resolve this frame's input through the content-defined bindings rather
        // than hardcoded keys: continuous state (held actions + axes) plus the
        // edges newly pressed this frame.
        pending_pressed.extend(Input::pressed_this_frame(&input_registry, &platform.input));
        let frame_input = Input::evaluate_continuous(&input_registry, &platform.input);

        let mut frame = RenderFrame::new(camera);
        let mut audio_commands = AudioCommands::default();

        timestep.begin_frame();
        let mut steps = 0;
        while steps < FixedTimestep::MAX_STEPS_PER_FRAME {
            let Some(dt) = timestep.consume_step() else {
                break;
            };
            // Deliver accumulated edge presses to exactly one step; later steps in
            // the same frame see held state and axes but no fresh edges.
            let mut input = frame_input.clone();
            if steps == 0 {
                input.set_pressed(std::mem::take(&mut pending_pressed));
            }

            {
                let mut ctx = Ctx {
                    world: &mut world,
                    map: &map.tilemap,
                    nav: &map.nav,
                    input: &input,
                    camera: &mut camera,
                    gfx: Gfx::new(&mut frame),
                    audio: Audio::new(&mut audio_commands),
                };
                schedule.run_fixed(&mut ctx, dt);
            }
            process_core_commands(&mut world, &mut audio_commands);

            steps += 1;
        }

        // Frame-rate-paced stages run once per rendered frame (not once per fixed
        // step), so UI text and camera follow neither duplicate when several
        // steps run nor stall when none do.
        let frame_dt = frame_ms / 1000.0;
        {
            let mut ctx = Ctx {
                world: &mut world,
                map: &map.tilemap,
                nav: &map.nav,
                input: &frame_input,
                camera: &mut camera,
                gfx: Gfx::new(&mut frame),
                audio: Audio::new(&mut audio_commands),
            };
            schedule.run_update(&mut ctx, frame_dt);
            schedule.run_render_extract(&mut ctx, frame_dt);
            schedule.run_ui(&mut ctx, frame_dt);
        }
        process_core_commands(&mut world, &mut audio_commands);

        if timestep.step_ready() {
            let now = Instant::now();
            if last_lag_warning.is_none_or(|last| now.duration_since(last) >= LAG_WARNING_INTERVAL)
            {
                log::warn!(
                    "fixed timestep hit {} steps in one frame; discarding accumulated lag",
                    FixedTimestep::MAX_STEPS_PER_FRAME
                );
                last_lag_warning = Some(now);
            }
            timestep.discard_lag();
        }

        frame.camera = camera;
        extract_tilemap_sprites(&map, &mut frame);
        extract_entity_sprites(&world, &mut frame);
        if let Some(audio) = &audio {
            for command in audio_commands.drain() {
                submit_audio_command(audio, command);
            }
        }

        if renderer.render(&platform.window, frame)? == VulkanRenderOutcome::Presented {
            rendered_frames += 1;
        }
        if let Some(limit) = smoke_frames {
            if rendered_frames >= limit {
                log::info!("GAME_SMOKE_FRAMES={limit} reached; exiting cleanly");
                platform.should_quit = true;
            }
        }
    }

    Ok(())
}

fn process_core_commands(world: &mut World, audio_commands: &mut AudioCommands) {
    let commands = world
        .get_resource_mut::<CommandQueue>()
        .map(|queue| queue.drain().collect::<Vec<_>>())
        .unwrap_or_default();

    for command in commands {
        match command {
            Command::Despawn(entity) => world.despawn(entity),
            Command::PlaySound(sound) => audio_commands.push(AudioCommand::Play {
                sound,
                volume: 0.8,
                looping: false,
            }),
        }
    }
}

fn submit_audio_command(audio: &AudioSystem, command: AudioCommand) {
    match command {
        AudioCommand::Play { .. } => audio.play_blip(),
    }
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
        unsafe {
            std::env::remove_var("GAME_SMOKE_FRAMES");
        }

        assert_eq!(parse_smoke_frames().unwrap(), None);
    }

    #[test]
    fn smoke_frames_accepts_zero_and_positive_counts() {
        let _guard = env_lock();
        unsafe {
            std::env::set_var("GAME_SMOKE_FRAMES", "0");
        }
        assert_eq!(parse_smoke_frames().unwrap(), Some(0));

        unsafe {
            std::env::set_var("GAME_SMOKE_FRAMES", "120");
        }
        assert_eq!(parse_smoke_frames().unwrap(), Some(120));

        unsafe {
            std::env::remove_var("GAME_SMOKE_FRAMES");
        }
    }

    #[test]
    fn smoke_frames_trims_whitespace() {
        let _guard = env_lock();
        unsafe {
            std::env::set_var("GAME_SMOKE_FRAMES", " 120 ");
        }

        assert_eq!(parse_smoke_frames().unwrap(), Some(120));

        unsafe {
            std::env::remove_var("GAME_SMOKE_FRAMES");
        }
    }

    #[test]
    fn smoke_frames_rejects_invalid_values() {
        let _guard = env_lock();
        for value in ["", "-1", "abc", "1.5"] {
            unsafe {
                std::env::set_var("GAME_SMOKE_FRAMES", value);
            }
            assert!(parse_smoke_frames().is_err(), "accepted {value:?}");
        }

        unsafe {
            std::env::remove_var("GAME_SMOKE_FRAMES");
        }
    }
}

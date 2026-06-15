use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use game_audio::AudioSystem;
use game_core::app::{
    Ctx, Game, MapData, RenderFrame, StartCtx, extract_entity_sprites, extract_tilemap_sprites,
};
use game_core::assets::AssetValidator;
use game_core::audio::{Audio, AudioCommands};
use game_core::backend::AudioCommand;
use game_core::builder::GameBuilder;
use game_core::camera::Camera2D;
use game_core::commands::{Command, CommandQueue};
use game_core::gfx::Gfx;
use game_core::input::{FrameActions, Input, InputState, Key};
use game_core::plugin::GamePlugin;
use game_core::schedule::{Schedule, ScheduleValidator};
use game_core::world::World;
use game_platform_sdl::window::Platform;
use game_renderer_vulkan::assets::asset_root;
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

pub fn run_legacy<G: Game>(game: G, title: &str) -> Result<()> {
    run_game(game, RuntimeConfig::default().title(title))
}

pub fn run<P: GamePlugin>(config: RuntimeConfig, plugin: P) -> Result<()> {
    let mut builder = GameBuilder::new();
    let game = plugin.build(&mut builder)?;
    validate_builder(&builder)?;
    run_game_with_schedule(game, config, builder.into_schedule())
}

/// Validates host-owned content (schedule wiring and on-disk assets) before any
/// backend is created or the main loop starts (Phase 11.3/11.4). Map and prefab
/// validation runs earlier, inside `GamePlugin::build`.
fn validate_builder(builder: &GameBuilder) -> Result<()> {
    ScheduleValidator::new(builder.schedule())
        .start_map_set(builder.start_map().is_some())
        // The runtime extracts tilemap/entity sprites into every frame itself, so
        // content is not required to register its own render_extract system.
        .builtin_render_extract()
        .validate()
        .context("schedule validation failed")?;

    let root = asset_root().context("failed to resolve asset root for validation")?;
    AssetValidator::new(builder.assets())
        .root(root)
        // Sounds are currently synthesized at runtime rather than loaded from disk.
        .allow_generated_sounds()
        .validate()
        .context("asset validation failed")?;

    Ok(())
}

pub fn run_game<G: Game>(mut game: G, config: RuntimeConfig) -> Result<()> {
    run_game_inner(&mut game, config, Schedule::new())
}

fn run_game_with_schedule<G: Game>(
    mut game: G,
    config: RuntimeConfig,
    schedule: Schedule,
) -> Result<()> {
    run_game_inner(&mut game, config, schedule)
}

fn run_game_inner<G: Game>(
    game: &mut G,
    config: RuntimeConfig,
    mut schedule: Schedule,
) -> Result<()> {
    let smoke_frames = parse_smoke_frames()?;

    let mut platform = Platform::new(&config.title, config.width, config.height)?;
    let mut renderer = VulkanContext::new(&platform.window)?;
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
    let mut map_slot: Option<MapData> = None;

    {
        let mut start_ctx = StartCtx::new(&mut world, &mut map_slot);
        if schedule.has_startup_systems() {
            schedule.run_startup(&mut start_ctx)?;
        } else {
            game.start(&mut start_ctx)?;
        }
    }
    let map = map_slot.expect("Game::start must call ctx.set_map(...)");

    if smoke_frames == Some(0) {
        log::info!("GAME_SMOKE_FRAMES=0 requested; initialized and exiting before rendering");
        return Ok(());
    }

    let mut pending_actions = FrameActions::default();
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
        game.record_frame_time(frame_ms);

        let frame_actions = frame_actions(&platform.input);
        if let Some(audio) = &audio {
            audio.poll_dropped_frames();
            audio.poll_dropped_voices();
        }
        pending_actions.merge(frame_actions);

        let mut frame = RenderFrame::new(camera);
        let mut audio_commands = AudioCommands::default();

        timestep.begin_frame();
        let mut steps = 0;
        while steps < FixedTimestep::MAX_STEPS_PER_FRAME {
            let Some(dt) = timestep.consume_step() else {
                break;
            };
            let actions = if steps == 0 {
                std::mem::take(&mut pending_actions)
            } else {
                FrameActions::default()
            };
            let input = Input::new(
                movement_axis(&platform.input),
                zoom_axis(&platform.input),
                actions,
            );

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
                if schedule.has_frame_systems() {
                    schedule.run_frame(&mut ctx, dt);
                } else {
                    game.update(&mut ctx, dt);
                }
            }
            process_core_commands(&mut world, &mut audio_commands);

            steps += 1;
        }

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
            Command::Spawn(_, _) | Command::SetMap(_) | Command::EmitEvent(_) => {}
        }
    }
}

fn submit_audio_command(audio: &AudioSystem, command: AudioCommand) {
    match command {
        AudioCommand::Play { .. } | AudioCommand::PlayMusic { .. } => audio.play_blip(),
        AudioCommand::StopMusic => {}
    }
}

fn frame_actions(input: &InputState) -> FrameActions {
    FrameActions {
        action_pressed: input.pressed(Key::Space) || input.pressed(Key::Enter),
        pause_pressed: input.pressed(Key::P),
        reset_pressed: input.pressed(Key::R),
        debug_die_pressed: input.pressed(Key::K),
    }
}

fn movement_axis(input: &InputState) -> glam::Vec2 {
    let x = axis(
        input.down(Key::A) || input.down(Key::Left),
        input.down(Key::D) || input.down(Key::Right),
    );
    let y = axis(
        input.down(Key::W) || input.down(Key::Up),
        input.down(Key::S) || input.down(Key::Down),
    );
    let value = glam::vec2(x, y);
    if value.length_squared() > 1.0 {
        value.normalize()
    } else {
        value
    }
}

fn zoom_axis(input: &InputState) -> f32 {
    axis(input.down(Key::Minus), input.down(Key::Plus))
}

fn axis(negative: bool, positive: bool) -> f32 {
    match (negative, positive) {
        (true, false) => -1.0,
        (false, true) => 1.0,
        _ => 0.0,
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

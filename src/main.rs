mod audio;
mod game;
mod platform;
mod renderer;

use std::time::{Duration, Instant};

use platform::input::FrameActions;
use platform::window::Platform;

// While the window is being actively resized (or is zero-sized/minimized) we
// skip rendering and idle briefly so we neither spin the CPU nor fight the
// compositor mid-resize.
const RESIZE_IDLE_SLEEP: Duration = Duration::from_millis(16);

fn main() -> anyhow::Result<()> {
    // Default to `info` so startup diagnostics and warnings are visible without
    // requiring RUST_LOG; an explicit RUST_LOG still overrides this.
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let mut platform = Platform::new("SDL3 + ash demo", 1280, 720)?;
    let mut vk = renderer::context::VulkanContext::new(&platform.window)?;
    let audio = match audio::AudioSystem::new(&platform.sdl) {
        Ok(audio) => Some(audio),
        Err(err) => {
            log::warn!("audio disabled: {err}");
            None
        }
    };
    let mut timestep = platform::time::FixedTimestep::new(120.0);
    let mut game = game::state::Game::new();
    let mut pending_actions = FrameActions::default();
    let mut previous_frame = Instant::now();

    while !platform.should_quit {
        platform.pump_events();

        let (width, height) = platform.drawable_size();
        if width == 0 || height == 0 || platform.resize_pending() {
            // Don't accumulate simulated time across the pause; treat the resize
            // gap as a single reset so physics doesn't lurch when we resume.
            previous_frame = Instant::now();
            timestep.reset_after_pause();
            std::thread::sleep(RESIZE_IDLE_SLEEP);
            continue;
        }

        if platform.take_stable_resize_request() {
            vk.request_swapchain_recreate();
        }

        let now = Instant::now();
        let frame_ms = (now - previous_frame).as_secs_f32() * 1000.0;
        previous_frame = now;
        game.record_frame_time(frame_ms);

        let frame_actions = platform.input.take_frame_actions();
        if frame_actions.action_pressed
            && let Some(audio) = &audio
        {
            audio.play_blip();
        }
        pending_actions.merge(frame_actions);

        timestep.begin_frame();
        let mut steps = 0;
        while timestep.step_ready() && steps < platform::time::FixedTimestep::MAX_STEPS_PER_FRAME {
            let dt = timestep.consume_step();
            // Edge-triggered actions are consumed by the first simulation step
            // only, so a key press is never applied twice in one render frame.
            let actions = if steps == 0 {
                std::mem::take(&mut pending_actions)
            } else {
                FrameActions::default()
            };
            game.update(dt, platform.input, actions);
            steps += 1;
        }
        if timestep.step_ready() {
            log::warn!(
                "fixed timestep hit {} steps in one frame; discarding accumulated lag",
                platform::time::FixedTimestep::MAX_STEPS_PER_FRAME
            );
            timestep.discard_lag();
        }

        let alpha = timestep.alpha();
        game.render(alpha, &mut vk);
        vk.render(&platform.window, game.camera())?;
    }

    Ok(())
}

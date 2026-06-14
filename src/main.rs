mod audio;
mod game;
mod platform;
mod renderer;

use std::time::{Duration, Instant};

use platform::input::FrameActions;
use platform::window::Platform;

// When the window is zero-sized/minimized there is no surface to render to, so
// we skip rendering and idle briefly instead of spinning the CPU. During an
// ordinary (nonzero) live resize we keep rendering with the current swapchain and
// let the debounced resize policy plus SUBOPTIMAL/OUT_OF_DATE handling drive
// recreation, so the window shows frames instead of freezing.
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

    // Optional smoke-test hook: when GAME_SMOKE_FRAMES=N is set, render N frames
    // then quit cleanly (running normal teardown). Lets CI / a headless run
    // exercise startup, rendering, and shutdown — including validation-layer
    // resource-leak checks at Drop — without a human closing the window.
    let smoke_frames: Option<u64> = std::env::var("GAME_SMOKE_FRAMES")
        .ok()
        .and_then(|value| value.parse().ok());
    let mut rendered_frames: u64 = 0;

    while !platform.should_quit {
        platform.pump_events();

        let (width, height) = platform.drawable_size();
        if width == 0 || height == 0 {
            // Minimized / zero-sized window: nothing to draw to. Don't accumulate
            // simulated time across the pause — treat the gap as a single reset so
            // physics doesn't lurch when the window comes back.
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

        rendered_frames += 1;
        if let Some(limit) = smoke_frames
            && rendered_frames >= limit
        {
            log::info!("GAME_SMOKE_FRAMES={limit} reached; exiting cleanly");
            platform.should_quit = true;
        }
    }

    Ok(())
}

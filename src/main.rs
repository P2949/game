mod audio;
mod game;
mod platform;
mod renderer;

use std::time::{Duration, Instant};

use platform::input::FrameActions;
use platform::window::Platform;
use renderer::context::RenderOutcome;

// When the window is zero-sized/minimized there is no surface to render to, so
// we skip rendering and idle briefly instead of spinning the CPU. During an
// ordinary (nonzero) live resize we keep rendering with the current swapchain and
// let the debounced resize policy plus SUBOPTIMAL/OUT_OF_DATE handling drive
// recreation, so the window shows frames instead of freezing.
const RESIZE_IDLE_SLEEP: Duration = Duration::from_millis(16);

// Minimum spacing between fixed-timestep "discarding lag" warnings. Under a
// sustained stall (e.g. a long resize burst) the cap is hit every frame, so
// rate-limit the warning to keep it from flooding the log.
const LAG_WARNING_INTERVAL: Duration = Duration::from_secs(1);

fn main() -> anyhow::Result<()> {
    // Default to `info` so startup diagnostics and warnings are visible without
    // requiring RUST_LOG; an explicit RUST_LOG still overrides this.
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let smoke_frames = parse_smoke_frames()?;

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
    let mut last_lag_warning: Option<Instant> = None;

    if smoke_frames == Some(0) {
        log::info!("GAME_SMOKE_FRAMES=0 requested; initialized and exiting before rendering");
        return Ok(());
    }

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
        if let Some(audio) = &audio {
            if frame_actions.action_pressed {
                audio.play_blip();
            }
            // Surface any audio-output drops and voice-cap drops from the realtime
            // callback on this (main) thread, where logging is safe.
            audio.poll_dropped_frames();
            audio.poll_dropped_voices();
        }
        pending_actions.merge(frame_actions);

        timestep.begin_frame();
        let mut steps = 0;
        while steps < platform::time::FixedTimestep::MAX_STEPS_PER_FRAME {
            let Some(dt) = timestep.consume_step() else {
                break;
            };
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
            let now = Instant::now();
            if last_lag_warning.is_none_or(|last| now.duration_since(last) >= LAG_WARNING_INTERVAL)
            {
                log::warn!(
                    "fixed timestep hit {} steps in one frame; discarding accumulated lag",
                    platform::time::FixedTimestep::MAX_STEPS_PER_FRAME
                );
                last_lag_warning = Some(now);
            }
            timestep.discard_lag();
        }

        let alpha = timestep.alpha();
        game.render(alpha, &mut vk);
        if vk.render(&platform.window, game.camera())? == RenderOutcome::Presented {
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

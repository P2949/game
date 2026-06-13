mod audio;
mod game;
mod platform;
mod renderer;

use platform::input::FrameActions;
use platform::window::Platform;

fn main() -> anyhow::Result<()> {
    env_logger::init();

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
    let start = std::time::Instant::now();
    let mut previous_frame = start;

    while !platform.should_quit {
        let now = std::time::Instant::now();
        let frame_ms = (now - previous_frame).as_secs_f32() * 1000.0;
        previous_frame = now;
        game.record_frame_time(frame_ms);

        platform.pump_events();
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

        if platform.framebuffer_resized {
            vk.request_swapchain_recreate();
        }

        let (width, height) = platform.drawable_size();
        if width == 0 || height == 0 {
            std::thread::sleep(std::time::Duration::from_millis(16));
            continue;
        }

        let alpha = timestep.alpha();
        game.render(alpha, &mut vk);

        vk.render(
            &platform.window,
            game.camera(),
            start.elapsed().as_secs_f32(),
        )?;
    }

    Ok(())
}

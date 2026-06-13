mod audio;
mod game;
mod platform;
mod renderer;

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
    let start = std::time::Instant::now();

    while !platform.should_quit {
        platform.pump_events();
        if platform.input.action_pressed
            && let Some(audio) = &audio
        {
            audio.play_blip();
        }

        timestep.begin_frame();

        while timestep.step_ready() {
            let dt = timestep.consume_step();
            game.update(dt, &platform);
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

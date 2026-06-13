mod audio;
mod game;
mod platform;
mod renderer;

use platform::window::Platform;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let mut platform = Platform::new("SDL3 + ash demo", 1280, 720)?;
    let mut vk = renderer::context::VulkanContext::new(&platform.window)?;
    let start = std::time::Instant::now();

    while !platform.should_quit {
        platform.pump_events();
        vk.render(start.elapsed().as_secs_f32())?;
    }

    Ok(())
}

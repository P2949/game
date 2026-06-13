mod audio;
mod game;
mod platform;
mod renderer;

use platform::window::Platform;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let mut platform = Platform::new("SDL3 + ash demo", 1280, 720)?;

    while !platform.should_quit {
        platform.pump_events();
        std::thread::sleep(std::time::Duration::from_millis(8));
    }

    Ok(())
}

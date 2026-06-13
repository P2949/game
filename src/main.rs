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

        if platform.framebuffer_resized {
            vk.request_swapchain_recreate();
        }

        let (width, height) = platform.drawable_size();
        if width == 0 || height == 0 {
            std::thread::sleep(std::time::Duration::from_millis(16));
            continue;
        }

        for y in 0..10 {
            for x in 0..10 {
                vk.draw_sprite(renderer::SpriteDraw {
                    texture: renderer::TEST_TEXTURE_ID,
                    position: glam::vec2(x as f32 * 40.0, y as f32 * 40.0),
                    size: glam::vec2(32.0, 32.0),
                    uv_min: glam::Vec2::ZERO,
                    uv_max: glam::Vec2::ONE,
                    color: glam::Vec4::ONE,
                });
            }
        }

        vk.render(&platform.window, start.elapsed().as_secs_f32())?;
    }

    Ok(())
}

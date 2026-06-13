mod audio;
mod game;
mod platform;
mod renderer;

use platform::window::Platform;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let mut platform = Platform::new("SDL3 + ash demo", 1280, 720)?;
    let mut vk = renderer::context::VulkanContext::new(&platform.window)?;
    let mut camera = game::camera::Camera2D {
        center: glam::vec2(200.0, 200.0),
        zoom: 1.0,
    };
    let start = std::time::Instant::now();
    let mut previous_frame = start;

    while !platform.should_quit {
        platform.pump_events();
        let now = std::time::Instant::now();
        let dt = (now - previous_frame).as_secs_f32();
        previous_frame = now;
        update_camera(&mut camera, platform.input, dt);

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
        vk.draw_text(
            "FPS: 240\nSprites: 100",
            glam::vec2(16.0, 16.0),
            glam::vec4(1.0, 0.95, 0.75, 1.0),
        );

        vk.render(&platform.window, camera, start.elapsed().as_secs_f32())?;
    }

    Ok(())
}

fn update_camera(camera: &mut game::camera::Camera2D, input: platform::input::InputState, dt: f32) {
    let mut movement = glam::Vec2::ZERO;

    if input.move_left {
        movement.x -= 1.0;
    }
    if input.move_right {
        movement.x += 1.0;
    }
    if input.move_up {
        movement.y -= 1.0;
    }
    if input.move_down {
        movement.y += 1.0;
    }

    if movement.length_squared() > 0.0 {
        camera.center += movement.normalize() * (360.0 * dt / camera.zoom);
    }

    let zoom_step = 1.0 + 2.0 * dt;
    if input.zoom_in {
        camera.zoom *= zoom_step;
    }
    if input.zoom_out {
        camera.zoom /= zoom_step;
    }
    camera.zoom = camera.zoom.clamp(0.25, 6.0);
}

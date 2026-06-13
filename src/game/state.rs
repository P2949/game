use crate::game::camera::Camera2D;
use crate::platform::window::Platform;
use crate::renderer::{self, SpriteDraw};

pub struct Game {
    camera: Camera2D,
    moving_prev_pos: glam::Vec2,
    moving_pos: glam::Vec2,
    moving_vel: glam::Vec2,
    log_timer: f32,
}

impl Game {
    pub fn new() -> Self {
        Self {
            camera: Camera2D {
                center: glam::vec2(200.0, 200.0),
                zoom: 1.0,
            },
            moving_prev_pos: glam::vec2(420.0, 120.0),
            moving_pos: glam::vec2(420.0, 120.0),
            moving_vel: glam::vec2(80.0, 0.0),
            log_timer: 0.0,
        }
    }

    pub fn camera(&self) -> Camera2D {
        self.camera
    }

    pub fn update(&mut self, dt: f32, platform: &Platform) {
        self.update_camera(dt, platform.input);

        self.moving_prev_pos = self.moving_pos;
        self.moving_pos += self.moving_vel * dt;

        if self.moving_pos.x < 420.0 || self.moving_pos.x > 760.0 {
            self.moving_vel.x = -self.moving_vel.x;
            self.moving_pos.x = self.moving_pos.x.clamp(420.0, 760.0);
        }

        self.log_timer += dt;
        if self.log_timer >= 1.0 {
            self.log_timer -= 1.0;
            log::info!("fixed update moving object at {:?}", self.moving_pos);
        }
    }

    pub fn render(&self, alpha: f32, renderer: &mut crate::renderer::context::VulkanContext) {
        for y in 0..10 {
            for x in 0..10 {
                renderer.draw_sprite(SpriteDraw {
                    texture: renderer::TEST_TEXTURE_ID,
                    position: glam::vec2(x as f32 * 40.0, y as f32 * 40.0),
                    size: glam::vec2(32.0, 32.0),
                    uv_min: glam::Vec2::ZERO,
                    uv_max: glam::Vec2::ONE,
                    color: glam::Vec4::ONE,
                });
            }
        }

        let moving_pos = self.moving_prev_pos.lerp(self.moving_pos, alpha);
        renderer.draw_sprite(SpriteDraw {
            texture: renderer::TEST_TEXTURE_ID,
            position: moving_pos,
            size: glam::vec2(48.0, 48.0),
            uv_min: glam::Vec2::ZERO,
            uv_max: glam::Vec2::ONE,
            color: glam::vec4(1.0, 0.35, 0.25, 1.0),
        });

        renderer.draw_text(
            "FPS: 240\nSprites: 101",
            glam::vec2(16.0, 16.0),
            glam::vec4(1.0, 0.95, 0.75, 1.0),
        );
    }

    fn update_camera(&mut self, dt: f32, input: crate::platform::input::InputState) {
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
            self.camera.center += movement.normalize() * (360.0 * dt / self.camera.zoom);
        }

        let zoom_step = 1.0 + 2.0 * dt;
        if input.zoom_in {
            self.camera.zoom *= zoom_step;
        }
        if input.zoom_out {
            self.camera.zoom /= zoom_step;
        }
        self.camera.zoom = self.camera.zoom.clamp(0.25, 6.0);
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

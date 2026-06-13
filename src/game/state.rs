use crate::game::camera::Camera2D;
use crate::platform::window::Platform;
use crate::renderer::{self, SpriteDraw};

pub struct Game {
    camera: Camera2D,
    player_prev_pos: glam::Vec2,
    player_pos: glam::Vec2,
    player_vel: glam::Vec2,
    player_size: glam::Vec2,
    log_timer: f32,
}

impl Game {
    pub fn new() -> Self {
        Self {
            camera: Camera2D {
                center: glam::vec2(200.0, 200.0),
                zoom: 1.0,
            },
            player_prev_pos: glam::vec2(420.0, 120.0),
            player_pos: glam::vec2(420.0, 120.0),
            player_vel: glam::Vec2::ZERO,
            player_size: glam::vec2(48.0, 48.0),
            log_timer: 0.0,
        }
    }

    pub fn camera(&self) -> Camera2D {
        self.camera
    }

    pub fn update(&mut self, dt: f32, platform: &Platform) {
        let speed = 220.0;
        self.player_prev_pos = self.player_pos;
        self.player_vel = platform.input.movement() * speed;
        self.player_pos += self.player_vel * dt;

        self.update_camera_zoom(dt, platform.input);
        self.camera.center = self.player_pos + self.player_size * 0.5;

        self.log_timer += dt;
        if self.log_timer >= 1.0 {
            self.log_timer -= 1.0;
            log::info!("fixed update player at {:?}", self.player_pos);
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

        let player_pos = self.player_prev_pos.lerp(self.player_pos, alpha);
        renderer.draw_sprite(SpriteDraw {
            texture: renderer::TEST_TEXTURE_ID,
            position: player_pos,
            size: self.player_size,
            uv_min: glam::Vec2::ZERO,
            uv_max: glam::Vec2::ONE,
            color: glam::vec4(1.0, 0.35, 0.25, 1.0),
        });

        renderer.draw_text(
            "FPS: 240\nSprites: 101",
            player_pos + glam::vec2(-404.0, -104.0),
            glam::vec4(1.0, 0.95, 0.75, 1.0),
        );
    }

    fn update_camera_zoom(&mut self, dt: f32, input: crate::platform::input::InputState) {
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

use crate::game::camera::Camera2D;
use crate::game::collision::{Aabb, move_with_collision};
use crate::game::world::Entity;
use crate::platform::window::Platform;
use crate::renderer::{self, SpriteDraw};

pub struct Game {
    camera: Camera2D,
    player: Entity,
    solids: Vec<Aabb>,
    log_timer: f32,
}

impl Game {
    pub fn new() -> Self {
        Self {
            camera: Camera2D {
                center: glam::vec2(200.0, 200.0),
                zoom: 1.0,
            },
            player: Entity {
                pos: glam::vec2(420.0, 120.0),
                prev_pos: glam::vec2(420.0, 120.0),
                vel: glam::Vec2::ZERO,
                size: glam::vec2(48.0, 48.0),
                sprite: renderer::TEST_TEXTURE_ID,
                solid: false,
            },
            solids: test_room_solids(),
            log_timer: 0.0,
        }
    }

    pub fn camera(&self) -> Camera2D {
        self.camera
    }

    pub fn update(&mut self, dt: f32, platform: &Platform) {
        let speed = 220.0;
        self.player.vel = platform.input.movement() * speed;
        move_with_collision(&mut self.player, &self.solids, dt);

        self.update_camera_zoom(dt, platform.input);
        self.camera.center = self.player.pos + self.player.size * 0.5;

        self.log_timer += dt;
        if self.log_timer >= 1.0 {
            self.log_timer -= 1.0;
            log::info!("fixed update player at {:?}", self.player.pos);
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

        for solid in &self.solids {
            renderer.draw_sprite(SpriteDraw {
                texture: renderer::TEST_TEXTURE_ID,
                position: solid.min,
                size: solid.max - solid.min,
                uv_min: glam::Vec2::ZERO,
                uv_max: glam::Vec2::ONE,
                color: glam::vec4(0.35, 0.45, 0.75, 1.0),
            });
        }

        let player_pos = self.player.interpolated_pos(alpha);
        renderer.draw_sprite(SpriteDraw {
            texture: self.player.sprite,
            position: player_pos,
            size: self.player.size,
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

fn test_room_solids() -> Vec<Aabb> {
    vec![
        Aabb::from_pos_size(glam::vec2(-160.0, -120.0), glam::vec2(920.0, 24.0)),
        Aabb::from_pos_size(glam::vec2(-160.0, 520.0), glam::vec2(920.0, 24.0)),
        Aabb::from_pos_size(glam::vec2(-160.0, -120.0), glam::vec2(24.0, 664.0)),
        Aabb::from_pos_size(glam::vec2(736.0, -120.0), glam::vec2(24.0, 664.0)),
        Aabb::from_pos_size(glam::vec2(280.0, 200.0), glam::vec2(96.0, 96.0)),
    ]
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

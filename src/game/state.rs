use crate::game::camera::Camera2D;
use crate::game::collision::{Aabb, move_with_collision};
use crate::game::world::Entity;
use crate::platform::input::{FrameActions, InputState};
use crate::renderer::{self, DrawCommands, SpriteDraw};

// Side length of the decorative floor-tile grid drawn in the world. Kept as a
// named constant so the HUD sprite count stays in sync with what's rendered.
const FLOOR_GRID: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    MainMenu,
    Playing,
    Paused,
    Dead,
}

pub struct Game {
    pub mode: GameMode,
    camera: Camera2D,
    player: Entity,
    solids: Vec<Aabb>,
    shake: Shake,
    effect_time: f32,
    frame_graph: FrameGraph,
}

impl Game {
    pub fn new() -> Self {
        let (camera, player, solids) = new_world();
        Self {
            mode: GameMode::MainMenu,
            camera,
            player,
            solids,
            shake: Shake::default(),
            effect_time: 0.0,
            frame_graph: FrameGraph::default(),
        }
    }

    pub fn camera(&self) -> Camera2D {
        self.camera
    }

    pub fn record_frame_time(&mut self, ms: f32) {
        self.frame_graph.push(ms);
    }

    pub fn update(&mut self, dt: f32, input: InputState, actions: FrameActions) {
        // Wrap the effect clock at TAU so it stays bounded over long sessions.
        // The shake waveform uses sin(t * 71) / cos(t * 53); since 71 and 53 are
        // integers, advancing t by TAU is a whole number of cycles for both, so
        // wrapping is seamless while keeping f32 precision high indefinitely.
        self.effect_time = (self.effect_time + dt).rem_euclid(std::f32::consts::TAU);
        self.shake.update(dt);

        match self.mode {
            GameMode::MainMenu => {
                if actions.action_pressed {
                    self.reset_world();
                    self.mode = GameMode::Playing;
                }
            }
            GameMode::Playing => {
                if actions.debug_die_pressed {
                    self.mode = GameMode::Dead;
                } else if actions.reset_pressed {
                    self.reset_world();
                } else if actions.pause_pressed {
                    self.mode = GameMode::Paused;
                } else {
                    self.update_playing(dt, input);
                }
            }
            GameMode::Paused => {
                if actions.pause_pressed {
                    self.mode = GameMode::Playing;
                }
            }
            GameMode::Dead => {
                if actions.action_pressed || actions.reset_pressed {
                    self.reset_world();
                    self.mode = GameMode::Playing;
                }
            }
        }
    }

    pub fn render(&self, alpha: f32, renderer: &mut impl DrawCommands) {
        if self.mode != GameMode::MainMenu {
            self.render_world(alpha, renderer);
        }

        self.render_ui(alpha, renderer);
    }

    fn update_playing(&mut self, dt: f32, input: InputState) {
        let speed = 220.0;
        let desired_vel = input.movement() * speed;
        self.player.vel = desired_vel;
        move_with_collision(&mut self.player, &self.solids, dt);
        if desired_vel.length_squared() > 0.0
            && self.player.vel.length_squared() < desired_vel.length_squared()
        {
            self.shake.add(0.18);
        }

        self.update_camera_zoom(dt, input);
        self.camera.center =
            self.player.pos + self.player.size * 0.5 + self.shake.offset(self.effect_time);
    }

    fn render_world(&self, alpha: f32, renderer: &mut impl DrawCommands) {
        for y in 0..FLOOR_GRID {
            for x in 0..FLOOR_GRID {
                renderer.draw_world_sprite(SpriteDraw {
                    texture: renderer::TEST_TEXTURE_ID,
                    layer: 0,
                    position: glam::vec2(x as f32 * 40.0, y as f32 * 40.0),
                    size: glam::vec2(32.0, 32.0),
                    uv_min: glam::Vec2::ZERO,
                    uv_max: glam::Vec2::ONE,
                    color: glam::Vec4::ONE,
                });
            }
        }

        for solid in &self.solids {
            renderer.draw_world_sprite(SpriteDraw {
                texture: renderer::TEST_TEXTURE_ID,
                layer: 5,
                position: solid.min,
                size: solid.max - solid.min,
                uv_min: glam::Vec2::ZERO,
                uv_max: glam::Vec2::ONE,
                color: glam::vec4(0.35, 0.45, 0.75, 1.0),
            });
        }

        let player_pos = self.player.interpolated_pos(alpha);
        renderer.draw_world_sprite(SpriteDraw {
            texture: self.player.sprite,
            layer: 10,
            position: player_pos,
            size: self.player.size,
            uv_min: glam::Vec2::ZERO,
            uv_max: glam::Vec2::ONE,
            color: glam::vec4(1.0, 0.35, 0.25, 1.0),
        });
    }

    fn render_ui(&self, _alpha: f32, renderer: &mut impl DrawCommands) {
        match self.mode {
            GameMode::MainMenu => {
                renderer.draw_ui_text(
                    "SDL3 + Vulkan Demo",
                    glam::vec2(80.0, 80.0),
                    glam::vec4(1.0, 0.95, 0.75, 1.0),
                );
                renderer.draw_ui_text(
                    "Press Space / Enter to start",
                    glam::vec2(80.0, 140.0),
                    glam::Vec4::ONE,
                );
            }
            GameMode::Paused => {
                renderer.draw_ui_text(
                    "Paused",
                    glam::vec2(80.0, 80.0),
                    glam::vec4(1.0, 0.95, 0.75, 1.0),
                );
                renderer.draw_ui_text(
                    "Press P to resume",
                    glam::vec2(80.0, 128.0),
                    glam::Vec4::ONE,
                );
            }
            GameMode::Dead => {
                renderer.draw_ui_text(
                    "You died",
                    glam::vec2(80.0, 80.0),
                    glam::vec4(1.0, 0.35, 0.25, 1.0),
                );
                renderer.draw_ui_text(
                    "Press Space / Enter to restart",
                    glam::vec2(80.0, 128.0),
                    glam::Vec4::ONE,
                );
            }
            GameMode::Playing => {}
        }

        if self.mode != GameMode::MainMenu {
            let avg_ms = self.frame_graph.average_recent(120).unwrap_or(0.0);
            let fps = if avg_ms > 0.001 { 1000.0 / avg_ms } else { 0.0 };
            let hud = format!(
                "FPS: {fps:.0}\nSprites: {}\nP pause  R reset  K test death",
                self.world_sprite_count()
            );
            renderer.draw_ui_text(
                &hud,
                glam::vec2(16.0, 16.0),
                glam::vec4(1.0, 0.95, 0.75, 1.0),
            );
            self.render_frame_graph(renderer, glam::vec2(16.0, 104.0));
        }
    }

    fn reset_world(&mut self) {
        let (camera, player, solids) = new_world();
        self.camera = camera;
        self.player = player;
        self.solids = solids;
        self.shake = Shake::default();
        self.effect_time = 0.0;
    }

    fn update_camera_zoom(&mut self, dt: f32, input: InputState) {
        let zoom_step = 1.0 + 2.0 * dt;
        if input.zoom_in {
            self.camera.zoom *= zoom_step;
        }
        if input.zoom_out {
            self.camera.zoom /= zoom_step;
        }
        self.camera.zoom = self.camera.zoom.clamp(0.25, 6.0);
    }

    fn render_frame_graph(&self, renderer: &mut impl DrawCommands, origin: glam::Vec2) {
        for (i, ms) in self.frame_graph.recent(120).enumerate() {
            let height = (ms * 2.0).clamp(1.0, 72.0);
            let color = if ms <= 16.7 {
                glam::vec4(0.25, 1.0, 0.45, 0.85)
            } else if ms <= 33.4 {
                glam::vec4(1.0, 0.85, 0.25, 0.85)
            } else {
                glam::vec4(1.0, 0.25, 0.25, 0.85)
            };

            renderer.draw_ui_sprite(SpriteDraw {
                texture: renderer::TEST_TEXTURE_ID,
                layer: 0,
                position: origin + glam::vec2(i as f32 * 2.0, 72.0 - height),
                size: glam::vec2(1.0, height),
                uv_min: glam::Vec2::ZERO,
                uv_max: glam::Vec2::ONE,
                color,
            });
        }
    }

    fn world_sprite_count(&self) -> usize {
        FLOOR_GRID * FLOOR_GRID + self.solids.len() + 1
    }
}

#[derive(Default)]
pub struct Shake {
    pub trauma: f32,
}

impl Shake {
    pub fn add(&mut self, amount: f32) {
        self.trauma = (self.trauma + amount).clamp(0.0, 1.0);
    }

    pub fn update(&mut self, dt: f32) {
        self.trauma = (self.trauma - dt * 1.8).max(0.0);
    }

    pub fn offset(&self, time: f32) -> glam::Vec2 {
        let amount = self.trauma * self.trauma;
        glam::vec2(
            (time * 71.0).sin() * 8.0 * amount,
            (time * 53.0).cos() * 8.0 * amount,
        )
    }
}

pub struct FrameGraph {
    samples_ms: [f32; 240],
    cursor: usize,
    filled: bool,
}

impl FrameGraph {
    pub fn push(&mut self, ms: f32) {
        self.samples_ms[self.cursor] = ms;
        self.cursor = (self.cursor + 1) % self.samples_ms.len();
        self.filled |= self.cursor == 0;
    }

    pub fn recent(&self, max: usize) -> impl Iterator<Item = f32> + '_ {
        let len = if self.filled {
            self.samples_ms.len()
        } else {
            self.cursor
        }
        .min(max);
        let start = (self.cursor + self.samples_ms.len() - len) % self.samples_ms.len();

        (0..len).map(move |i| self.samples_ms[(start + i) % self.samples_ms.len()])
    }

    pub fn average_recent(&self, max: usize) -> Option<f32> {
        let mut sum = 0.0;
        let mut count = 0;
        for ms in self.recent(max) {
            sum += ms;
            count += 1;
        }

        (count > 0).then(|| sum / count as f32)
    }
}

impl Default for FrameGraph {
    fn default() -> Self {
        Self {
            samples_ms: [0.0; 240],
            cursor: 0,
            filled: false,
        }
    }
}

fn new_world() -> (Camera2D, Entity, Vec<Aabb>) {
    let player = Entity {
        pos: glam::vec2(420.0, 120.0),
        prev_pos: glam::vec2(420.0, 120.0),
        vel: glam::Vec2::ZERO,
        size: glam::vec2(48.0, 48.0),
        sprite: renderer::TEST_TEXTURE_ID,
    };
    let camera = Camera2D {
        center: player.pos + player.size * 0.5,
        zoom: 1.0,
    };

    (camera, player, test_room_solids())
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

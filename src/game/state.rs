use crate::game::camera::Camera2D;
use crate::game::collision::{Aabb, depenetrate, move_with_swept_collision, validate_spawn};
use crate::game::world::Entity;
use crate::platform::input::{FrameActions, InputState};
use crate::renderer::{self, DrawCommands, SpriteDraw};

// Side length of the decorative floor-tile grid drawn in the world. Kept as a
// named constant so the HUD sprite count stays in sync with what's rendered.
const FLOOR_GRID: usize = 10;
const MAX_FRAME_GRAPH_MS: f32 = 10_000.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    MainMenu,
    Playing,
    Paused,
    Dead,
}

pub struct Game {
    mode: GameMode,
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

    #[allow(dead_code)]
    pub fn mode(&self) -> GameMode {
        self.mode
    }

    pub fn record_frame_time(&mut self, ms: f32) {
        // `FrameGraph::push` sanitizes the value, so non-finite/negative deltas
        // from any caller are handled there.
        self.frame_graph.push(ms);
    }

    pub fn update(&mut self, dt: f32, input: InputState, actions: FrameActions) {
        // `FixedTimestep` already supplies a sane fixed dt, but `update` is public:
        // guard against a non-finite or non-positive dt so a direct caller cannot
        // push simulation state (effect clock, physics) into NaN/garbage.
        let dt = if dt.is_finite() && dt > 0.0 { dt } else { 0.0 };

        // Cosmetic effects (the effect clock and camera shake) advance only while
        // actively playing — see `update_playing`. Pause, the menu, and death all
        // freeze them (roadmap 5.1, option A: pause freezes simulation *and*
        // effects), so resuming continues exactly where it left off.
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
        // Wrap the effect clock at TAU so it stays bounded over long sessions.
        // The shake waveform uses sin(t * 71) / cos(t * 53); since 71 and 53 are
        // integers, advancing t by TAU is a whole number of cycles for both, so
        // wrapping is seamless while keeping f32 precision high indefinitely.
        self.effect_time = (self.effect_time + dt).rem_euclid(std::f32::consts::TAU);
        self.shake.update(dt);

        let speed = 220.0;
        let desired_vel = input.movement() * speed;
        self.player.set_velocity(desired_vel);
        move_with_swept_collision(&mut self.player, &self.solids, dt);
        if desired_vel.length_squared() > 0.0
            && self.player.velocity().length_squared() < desired_vel.length_squared()
        {
            self.shake.add(0.18);
        }

        self.update_camera_zoom(dt, input);
        self.camera.set_center(
            self.player.position() + self.player.size() * 0.5 + self.shake.offset(self.effect_time),
        );
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
                position: solid.min(),
                size: solid.size(),
                uv_min: glam::Vec2::ZERO,
                uv_max: glam::Vec2::ONE,
                color: glam::vec4(0.35, 0.45, 0.75, 1.0),
            });
        }

        let player_pos = self.player.interpolated_pos(alpha);
        renderer.draw_world_sprite(SpriteDraw {
            texture: self.player.sprite(),
            layer: 10,
            position: player_pos,
            size: self.player.size(),
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
                "FPS: {fps:.0}\nWorld sprites: {}\nP pause  R reset  K test death",
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
        let mut zoom = self.camera.zoom();
        if input.zoom_in {
            zoom *= zoom_step;
        }
        if input.zoom_out {
            zoom /= zoom_step;
        }
        // Route through `set_zoom` so the gameplay clamp and the camera's own
        // validity guard are applied together.
        self.camera.set_zoom(zoom.clamp(0.25, 6.0));
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
    trauma: f32,
}

impl Shake {
    #[allow(dead_code)]
    pub fn trauma(&self) -> f32 {
        self.trauma
    }

    pub fn add(&mut self, amount: f32) {
        // `add` is public; ignore a non-finite amount so a stray NaN/Inf cannot
        // poison trauma (and through it the camera offset) for the rest of the run.
        if !amount.is_finite() {
            return;
        }
        self.trauma = (self.trauma + amount).clamp(0.0, 1.0);
    }

    pub fn update(&mut self, dt: f32) {
        if !dt.is_finite() || dt <= 0.0 {
            return;
        }

        self.trauma = (self.trauma - dt * 1.8).clamp(0.0, 1.0);
    }

    pub fn offset(&self, time: f32) -> glam::Vec2 {
        let trauma = if self.trauma.is_finite() {
            self.trauma.clamp(0.0, 1.0)
        } else {
            0.0
        };
        let time = if time.is_finite() { time } else { 0.0 };
        let amount = trauma * trauma;

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
        // Sanitize here so every caller (this is a public method) is protected: a
        // NaN/inf would poison every average derived from the graph, and a negative
        // delta is meaningless. Clamp to a bounded diagnostic range.
        let ms = if ms.is_finite() {
            ms.clamp(0.0, MAX_FRAME_GRAPH_MS)
        } else {
            0.0
        };
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
    let mut player = Entity::new_player(glam::vec2(420.0, 120.0));
    let solids = test_room_solids();

    // Nudge the player out of any solid it spawns inside. This is a no-op for the
    // hand-authored room (the spawn is clear), but keeps spawns robust once levels
    // are data-driven and a spawn point could land embedded.
    if let Some(push) = depenetrate(player.aabb(), &solids) {
        player.set_position(player.position() + push);
    }

    let camera = Camera2D::new(player.position() + player.size() * 0.5, 1.0);
    validate_spawn(&player, &solids).expect("test room player spawn must not overlap solids");

    (camera, player, solids)
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

#[cfg(test)]
mod tests {
    use super::{FrameGraph, Game, GameMode, Shake};
    use crate::platform::input::{FrameActions, InputState};

    const DT: f32 = 1.0 / 120.0;
    // Player start position + half-size, matching `new_world`.
    const START_CAMERA_CENTER: glam::Vec2 = glam::Vec2::new(444.0, 144.0);

    fn playing_game() -> Game {
        let mut game = Game::new();
        game.update(
            DT,
            InputState::default(),
            FrameActions {
                action_pressed: true,
                ..Default::default()
            },
        );
        assert_eq!(game.mode(), GameMode::Playing);
        game
    }

    #[test]
    fn main_menu_starts_playing_on_action() {
        let mut game = Game::new();
        assert_eq!(game.mode(), GameMode::MainMenu);

        game.update(DT, InputState::default(), FrameActions::default());
        assert_eq!(game.mode(), GameMode::MainMenu, "no action keeps the menu");

        game.update(
            DT,
            InputState::default(),
            FrameActions {
                action_pressed: true,
                ..Default::default()
            },
        );
        assert_eq!(game.mode(), GameMode::Playing);
    }

    #[test]
    fn playing_pauses_and_resumes() {
        let mut game = playing_game();

        game.update(
            DT,
            InputState::default(),
            FrameActions {
                pause_pressed: true,
                ..Default::default()
            },
        );
        assert_eq!(game.mode(), GameMode::Paused);

        game.update(
            DT,
            InputState::default(),
            FrameActions {
                pause_pressed: true,
                ..Default::default()
            },
        );
        assert_eq!(game.mode(), GameMode::Playing);
    }

    #[test]
    fn playing_advances_effect_time() {
        let mut game = playing_game();
        assert_eq!(game.effect_time, 0.0);

        game.update(DT, InputState::default(), FrameActions::default());

        assert!(
            game.effect_time > 0.0,
            "effect clock should advance while playing"
        );
    }

    #[test]
    fn pause_freezes_effect_time() {
        let mut game = playing_game();
        game.update(DT, InputState::default(), FrameActions::default());
        let before_pause = game.effect_time;
        assert!(before_pause > 0.0);

        game.update(
            DT,
            InputState::default(),
            FrameActions {
                pause_pressed: true,
                ..Default::default()
            },
        );
        assert_eq!(game.mode(), GameMode::Paused);

        for _ in 0..5 {
            game.update(DT, InputState::default(), FrameActions::default());
        }

        assert_eq!(
            game.effect_time, before_pause,
            "pause must freeze the effect clock"
        );
    }

    #[test]
    fn pause_freezes_shake_decay() {
        let mut game = playing_game();
        game.shake.add(0.8);

        game.update(
            DT,
            InputState::default(),
            FrameActions {
                pause_pressed: true,
                ..Default::default()
            },
        );
        assert_eq!(game.mode(), GameMode::Paused);
        let trauma = game.shake.trauma();
        assert!(trauma > 0.0);

        for _ in 0..10 {
            game.update(DT, InputState::default(), FrameActions::default());
        }

        assert_eq!(game.shake.trauma(), trauma, "pause must freeze shake decay");
    }

    #[test]
    fn playing_transitions_to_dead_then_restarts() {
        let mut game = playing_game();

        game.update(
            DT,
            InputState::default(),
            FrameActions {
                debug_die_pressed: true,
                ..Default::default()
            },
        );
        assert_eq!(game.mode(), GameMode::Dead);

        game.update(
            DT,
            InputState::default(),
            FrameActions {
                action_pressed: true,
                ..Default::default()
            },
        );
        assert_eq!(game.mode(), GameMode::Playing);
    }

    #[test]
    fn camera_follows_player_movement() {
        let mut game = playing_game();
        let start = game.camera().center();
        assert_eq!(start, START_CAMERA_CENTER);

        let mut input = InputState::default();
        input.set_move_x(1.0);
        game.update(DT, input, FrameActions::default());

        let after = game.camera().center();
        assert!(
            after.x > start.x,
            "camera should track the player moving right"
        );
        assert_eq!(after.y, start.y);
    }

    #[test]
    fn reset_returns_player_to_start() {
        let mut game = playing_game();

        let mut input = InputState::default();
        input.set_move_x(1.0);
        for _ in 0..10 {
            game.update(DT, input, FrameActions::default());
        }
        assert!(game.camera().center().x > START_CAMERA_CENTER.x);

        game.update(
            DT,
            InputState::default(),
            FrameActions {
                reset_pressed: true,
                ..Default::default()
            },
        );
        assert_eq!(game.mode(), GameMode::Playing);
        assert_eq!(game.camera().center(), START_CAMERA_CENTER);
    }

    #[test]
    fn shake_add_ignores_non_finite_amounts() {
        let mut shake = Shake::default();
        shake.add(0.5);
        shake.add(f32::NAN);
        shake.add(f32::INFINITY);
        shake.add(f32::NEG_INFINITY);
        assert_eq!(shake.trauma(), 0.5);
    }

    #[test]
    fn shake_update_ignores_non_finite_dt() {
        let mut shake = Shake::default();
        shake.add(0.5);
        shake.update(f32::NAN);
        shake.update(f32::INFINITY);

        assert_eq!(shake.trauma(), 0.5);
    }

    #[test]
    fn shake_update_ignores_negative_dt() {
        let mut shake = Shake::default();
        shake.add(0.5);
        shake.update(-1.0);

        assert_eq!(shake.trauma(), 0.5);
    }

    #[test]
    fn shake_update_clamps_trauma() {
        let mut shake = Shake::default();
        shake.add(1.0);
        shake.update(10.0);

        assert_eq!(shake.trauma(), 0.0);
    }

    #[test]
    fn shake_offset_is_finite_for_non_finite_time() {
        let mut shake = Shake::default();
        shake.add(0.5);

        assert!(shake.offset(f32::NAN).is_finite());
        assert!(shake.offset(f32::INFINITY).is_finite());
    }

    #[test]
    fn frame_graph_orders_and_averages_recent_samples() {
        let mut graph = FrameGraph::default();
        assert_eq!(graph.average_recent(10), None);

        graph.push(10.0);
        graph.push(20.0);
        graph.push(30.0);

        let recent: Vec<f32> = graph.recent(10).collect();
        assert_eq!(recent, vec![10.0, 20.0, 30.0]);
        assert_eq!(graph.average_recent(10), Some(20.0));
    }

    #[test]
    fn frame_graph_recent_is_bounded_by_request_and_wraps() {
        let mut graph = FrameGraph::default();
        for i in 0..300 {
            graph.push(i as f32);
        }
        // Only the most recent 5 samples, in chronological order.
        let recent: Vec<f32> = graph.recent(5).collect();
        assert_eq!(recent, vec![295.0, 296.0, 297.0, 298.0, 299.0]);
    }

    #[test]
    fn frame_graph_clamps_extreme_samples() {
        let mut graph = FrameGraph::default();
        graph.push(f32::MAX);

        assert_eq!(graph.average_recent(1), Some(super::MAX_FRAME_GRAPH_MS));
    }

    #[test]
    fn frame_graph_rejects_non_finite_samples() {
        let mut graph = FrameGraph::default();
        graph.push(f32::INFINITY);
        graph.push(f32::NAN);

        assert_eq!(graph.average_recent(2), Some(0.0));
    }
}

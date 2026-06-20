//! Beginner default game bundles.

use game_core::backend::SoundHandle;
use game_core::input::{ActionId, Axis2dId};
use game_core::world::{EntityId, Velocity};
use glam::{vec2, vec4};

use crate::app::GameApp;
use crate::beginner::actors::{Enemy, Player, PlayerMovement, Speed};
use crate::beginner::animation::{Animation, AnimationSet};
use crate::beginner::combat::MeleeCombatConfig;
use crate::beginner::state::SimpleGameState;
use crate::context::{GameCtx, StartupGameCtx};
use crate::input::TopDownControls;

const MOVEMENT_ANIMATION_EPSILON_SQUARED: f32 = 0.0001;

pub struct TopDownGameAuthor<'a, 'app> {
    app: &'a mut GameApp<'app>,
    movement: Option<Axis2dId>,
    attack: Option<ActionId>,
    pause: Option<ActionId>,
    reset: Option<ActionId>,
    debug_kill: Option<ActionId>,
    debug_overlay: Option<ActionId>,
    debug_restart: Option<ActionId>,
    zoom: Option<(ActionId, ActionId)>,
    hit_sound: Option<SoundHandle>,
    melee_combat: bool,
    enemy_chase: bool,
    enemy_patrol: bool,
    collision: bool,
    camera_follow: bool,
    pause_death_ui: bool,
    player_animation_by_movement: bool,
    enemy_animation_by_movement: bool,
    player_directional_animation: bool,
    attack_animation: Option<&'static str>,
}

impl<'a, 'app> TopDownGameAuthor<'a, 'app> {
    pub(crate) fn new(app: &'a mut GameApp<'app>) -> Self {
        Self {
            app,
            movement: None,
            attack: None,
            pause: None,
            reset: None,
            debug_kill: None,
            debug_overlay: None,
            debug_restart: None,
            zoom: None,
            hit_sound: None,
            melee_combat: false,
            enemy_chase: false,
            enemy_patrol: false,
            collision: false,
            camera_follow: false,
            pause_death_ui: false,
            player_animation_by_movement: false,
            enemy_animation_by_movement: false,
            player_directional_animation: false,
            attack_animation: None,
        }
    }

    pub fn movement(mut self, movement: Axis2dId) -> Self {
        self.movement = Some(movement);
        self
    }

    pub fn controls(self, controls: TopDownControls) -> Self {
        self.movement(controls.movement)
            .attack(controls.attack)
            .pause(controls.pause)
            .reset(controls.reset)
            .debug_kill(controls.debug_die)
            .debug_toggle(controls.debug_overlay)
            .debug_restart(controls.reset)
            .zoom(controls.zoom_in, controls.zoom_out)
    }

    pub fn attack(mut self, attack: ActionId) -> Self {
        self.attack = Some(attack);
        self
    }

    pub fn pause(mut self, pause: ActionId) -> Self {
        self.pause = Some(pause);
        self
    }

    pub fn reset(mut self, reset: ActionId) -> Self {
        self.reset = Some(reset);
        self
    }

    pub fn debug_kill(mut self, debug_kill: ActionId) -> Self {
        self.debug_kill = Some(debug_kill);
        self
    }

    pub fn debug_toggle(mut self, debug_overlay: ActionId) -> Self {
        self.debug_overlay = Some(debug_overlay);
        self
    }

    pub fn debug_restart(mut self, debug_restart: ActionId) -> Self {
        self.debug_restart = Some(debug_restart);
        self
    }

    pub fn zoom(mut self, zoom_in: ActionId, zoom_out: ActionId) -> Self {
        self.zoom = Some((zoom_in, zoom_out));
        self
    }

    pub fn hit_sound(mut self, hit_sound: SoundHandle) -> Self {
        self.hit_sound = Some(hit_sound);
        self
    }

    pub fn combat_sound(self, hit_sound: SoundHandle) -> Self {
        self.hit_sound(hit_sound)
    }

    pub fn with_melee_combat(mut self) -> Self {
        self.melee_combat = true;
        self
    }

    pub fn with_enemy_chase(mut self) -> Self {
        self.enemy_chase = true;
        self
    }

    pub fn with_enemy_patrol(mut self) -> Self {
        self.enemy_patrol = true;
        self
    }

    pub fn with_collision(mut self) -> Self {
        self.collision = true;
        self
    }

    pub fn with_camera_follow(mut self) -> Self {
        self.camera_follow = true;
        self
    }

    pub fn with_pause_death_ui(mut self) -> Self {
        self.pause_death_ui = true;
        self
    }

    pub fn with_player_animation_by_movement(mut self) -> Self {
        self.player_animation_by_movement = true;
        self
    }

    pub fn with_enemy_animation_by_movement(mut self) -> Self {
        self.enemy_animation_by_movement = true;
        self
    }

    pub fn with_player_directional_animation(mut self) -> Self {
        self.player_directional_animation = true;
        self
    }

    pub fn with_attack_animation(mut self, name: &'static str) -> Self {
        self.attack_animation = Some(name);
        self
    }

    pub fn build(self) {
        let app = self.app;
        app.on_start(startup_simple_game);

        if self.debug_overlay.is_some() {
            app.configure_debug_overlay(|overlay| overlay.enabled = false);
        }

        let state_actions = StateActions {
            pause: self.pause,
            reset: self.reset,
            debug_kill: self.debug_kill,
            debug_overlay: self.debug_overlay,
            debug_restart: self.debug_restart,
            attack: self.attack,
        };
        app.every_tick(move |game: &mut GameCtx<'_, '_>, _dt| {
            state_input_system(game, state_actions);
        });

        if self.movement.is_some() {
            app.every_active_tick::<SimpleGameState>(|game: &mut GameCtx<'_, '_>, _dt| {
                game.drive_input::<PlayerMovement, Speed>();
            });
        }

        if self.enemy_chase {
            app.every_active_tick::<SimpleGameState>(|game: &mut GameCtx<'_, '_>, dt| {
                game.chase_first::<Player>(dt);
            });
        }

        if self.enemy_patrol {
            app.every_active_tick::<SimpleGameState>(|game: &mut GameCtx<'_, '_>, dt| {
                game.run_patrol(dt);
            });
        }

        if self.collision {
            app.every_active_tick::<SimpleGameState>(|game: &mut GameCtx<'_, '_>, dt| {
                game.move_and_collide(dt);
            });
        }

        if self.melee_combat {
            if let Some(attack) = self.attack {
                let config = MeleeCombatConfig {
                    attack,
                    hit_sound: self.hit_sound,
                    despawn_dead_enemies: true,
                    player_attack_animation: self.attack_animation,
                };
                app.every_active_tick::<SimpleGameState>(move |game: &mut GameCtx<'_, '_>, dt| {
                    game.run_melee_combat(config, dt);
                });
            } else {
                log::warn!("with_melee_combat() was enabled but no attack action was configured");
            }
        }

        if self.player_animation_by_movement {
            app.every_active_frame::<SimpleGameState>(player_animation_by_movement_system);
        }

        if self.enemy_animation_by_movement {
            app.every_active_frame::<SimpleGameState>(enemy_animation_by_movement_system);
        }

        if self.player_directional_animation {
            app.every_active_frame::<SimpleGameState>(player_directional_animation_system);
        }

        app.every_active_frame::<SimpleGameState>(|game: &mut GameCtx<'_, '_>, dt| {
            game.update_animations(dt);
        });

        app.every_tick(death_state_system);

        if self.zoom.is_some() || self.camera_follow {
            let zoom = self.zoom;
            let camera_follow = self.camera_follow;
            app.every_frame(move |game: &mut GameCtx<'_, '_>, dt| {
                if let Some((zoom_in, zoom_out)) = zoom {
                    game.zoom_camera_from_actions(zoom_in, zoom_out, dt);
                }
                if camera_follow {
                    game.camera_follow_first::<Player>();
                }
            });
        }

        if self.pause_death_ui {
            app.draw_ui(pause_death_ui_system);
        }

        app.every_frame(|game: &mut GameCtx<'_, '_>, dt| {
            game.update_camera_shake(dt);
        });

        app.fixed_systems_are_pause_guarded();
    }
}

#[derive(Clone, Copy)]
struct StateActions {
    pause: Option<ActionId>,
    reset: Option<ActionId>,
    debug_kill: Option<ActionId>,
    debug_overlay: Option<ActionId>,
    debug_restart: Option<ActionId>,
    attack: Option<ActionId>,
}

fn startup_simple_game(game: &mut StartupGameCtx<'_, '_>) -> anyhow::Result<()> {
    game.init_resource::<SimpleGameState>();
    game.spawn_start_map()
}

fn pressed(game: &GameCtx<'_, '_>, action: Option<ActionId>) -> bool {
    action.is_some_and(|action| game.pressed(action))
}

fn state_input_system(game: &mut GameCtx<'_, '_>, actions: StateActions) {
    let mut state = game
        .resource::<SimpleGameState>()
        .copied()
        .unwrap_or_default();

    if pressed(game, actions.pause) {
        state.paused = !state.paused;
    }

    if pressed(game, actions.reset) {
        game.reset_to_start_map_or_log();
        state = SimpleGameState::default();
    }

    if pressed(game, actions.debug_kill) {
        game.kill_player();
    }

    if pressed(game, actions.debug_overlay) {
        game.toggle_debug_overlay();
    }

    if pressed(game, actions.debug_restart) {
        game.restart_map_or_log();
        state = SimpleGameState::default();
    }

    state.player_dead = game.player_is_dead();
    if state.player_dead && (pressed(game, actions.attack) || pressed(game, actions.reset)) {
        game.reset_to_start_map_or_log();
        state = SimpleGameState::default();
    }

    if state.player_dead || state.paused {
        game.stop_all_velocity();
    }

    game.insert_resource(state);
}

fn death_state_system(game: &mut GameCtx<'_, '_>, _dt: f32) {
    let mut state = game
        .resource::<SimpleGameState>()
        .copied()
        .unwrap_or_default();
    state.player_dead = game.player_is_dead();
    game.insert_resource(state);
}

fn player_animation_by_movement_system(game: &mut GameCtx<'_, '_>, _dt: f32) {
    for id in game.entities_with::<Player>() {
        if one_shot_animation_is_active(game, id) {
            continue;
        }
        let moving = game.component::<Velocity>(id).is_some_and(|velocity| {
            velocity.0.length_squared() > MOVEMENT_ANIMATION_EPSILON_SQUARED
        });
        let animation = if moving { "walk" } else { "idle" };
        game.play_animation(id, animation);
    }
}

fn enemy_animation_by_movement_system(game: &mut GameCtx<'_, '_>, _dt: f32) {
    for id in game.entities_with::<Enemy>() {
        if one_shot_animation_is_active(game, id) || game.is_dead(id) {
            continue;
        }
        let moving = game.component::<Velocity>(id).is_some_and(|velocity| {
            velocity.0.length_squared() > MOVEMENT_ANIMATION_EPSILON_SQUARED
        });
        game.play_animation(id, if moving { "walk" } else { "idle" });
    }
}

fn player_directional_animation_system(game: &mut GameCtx<'_, '_>, _dt: f32) {
    for id in game.entities_with::<Player>() {
        if one_shot_animation_is_active(game, id) {
            continue;
        }
        let velocity = game
            .component::<Velocity>(id)
            .map(|velocity| velocity.0)
            .unwrap_or_default();
        if velocity.length_squared() <= MOVEMENT_ANIMATION_EPSILON_SQUARED {
            game.play_animation(id, "idle");
            continue;
        }
        let name = if velocity.x.abs() >= velocity.y.abs() {
            if velocity.x >= 0.0 {
                "walk_right"
            } else {
                "walk_left"
            }
        } else if velocity.y >= 0.0 {
            "walk_down"
        } else {
            "walk_up"
        };
        // A prefab may intentionally omit some directions. In that case, use
        // its ordinary walk clip rather than freezing on the prior direction.
        if !game.play_animation(id, name) {
            game.play_animation(id, "walk");
        }
    }
}

fn one_shot_animation_is_active(game: &GameCtx<'_, '_>, id: EntityId) -> bool {
    let Some(animation) = game.component::<Animation>(id) else {
        return false;
    };
    let Some(set) = game.component::<AnimationSet>(id) else {
        return false;
    };
    let Some(clip) = set.get(&animation.current) else {
        return false;
    };
    if clip.looping || clip.frames.is_empty() {
        return false;
    }

    let frame_seconds = 1.0 / clip.fps.max(0.001);
    animation.frame + 1 < clip.frames.len() || animation.timer < frame_seconds
}

fn pause_death_ui_system(game: &mut GameCtx<'_, '_>, _dt: f32) {
    let state = game
        .resource::<SimpleGameState>()
        .copied()
        .unwrap_or_default();
    if state.player_dead {
        game.text("You died", vec2(24.0, 24.0), vec4(1.0, 0.35, 0.25, 1.0));
    } else if state.paused {
        game.text("Paused", vec2(24.0, 24.0), vec4(1.0, 0.95, 0.75, 1.0));
    }
}

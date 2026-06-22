//! Beginner default game bundles.

use anyhow::Result;

use game_ai::ChaseTarget;
use game_core::backend::SoundHandle;
use game_core::input::{ActionId, Axis2dId};
use game_core::world::{EntityId, Velocity};
use glam::{vec2, vec4};

use crate::app::{GameApp, GamePlugin};
use crate::beginner::actors::{Enemy, FacingDirection, Player, PlayerMovement, Speed};
use crate::beginner::animation::{Animation, AnimationSet};
use crate::beginner::combat::MeleeCombatConfig;
use crate::beginner::state::SimpleGameState;
use crate::beginner::ui::{UiFocus, UiNavigation};
use crate::context::{GameCtx, StartupGameCtx};
use crate::input::TopDownControls;

const MOVEMENT_ANIMATION_EPSILON_SQUARED: f32 = 0.0001;

pub struct TopDownGameAuthor<'a, 'app> {
    app: &'a mut GameApp<'app>,
    movement: Option<Axis2dId>,
    attack: Option<ActionId>,
    pause: Option<ActionId>,
    reset: Option<ActionId>,
    reload: Option<ActionId>,
    menu_navigation: Option<(ActionId, ActionId, ActionId)>,
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
    enemy_directional_animation: bool,
    attack_animation: Option<&'static str>,
    directional_attack_animation: bool,
}

impl<'a, 'app> TopDownGameAuthor<'a, 'app> {
    pub(crate) fn new(app: &'a mut GameApp<'app>) -> Self {
        Self {
            app,
            movement: None,
            attack: None,
            pause: None,
            reset: None,
            reload: None,
            menu_navigation: None,
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
            enemy_directional_animation: false,
            attack_animation: None,
            directional_attack_animation: false,
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
            .reload(controls.reload)
            .menu_navigation(controls.menu_up, controls.menu_down, controls.menu_accept)
            .debug_kill(controls.debug_die)
            .debug_toggle(controls.debug_overlay)
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

    /// Sets the action that reloads the active text map during development.
    pub fn reload(mut self, reload: ActionId) -> Self {
        self.reload = Some(reload);
        self
    }

    /// Sets the standard up/down/accept actions for focused beginner menus.
    pub fn menu_navigation(mut self, up: ActionId, down: ActionId, accept: ActionId) -> Self {
        self.menu_navigation = Some((up, down, accept));
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

    /// Uses a sound registered through `game.asset_bag()` for melee hits.
    ///
    /// This keeps first-game setup name based. Typed content crates can keep
    /// using [`Self::hit_sound`] with their stored handle instead.
    pub fn hit_sound_named(mut self, key: &str) -> Self {
        self.hit_sound = Some(
            self.app
                .resolve_sound(key)
                .unwrap_or_else(|error| panic!("{error}")),
        );
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

    /// Switches enemy walk clips between `walk_up`, `walk_down`, `walk_left`,
    /// and `walk_right` based on their velocity.
    pub fn with_enemy_directional_animation(mut self) -> Self {
        self.enemy_directional_animation = true;
        self
    }

    pub fn with_attack_animation(mut self, name: &'static str) -> Self {
        self.attack_animation = Some(name);
        self
    }

    /// Plays `attack_up`, `attack_down`, `attack_left`, or `attack_right` on
    /// the player when the configured attack action is pressed. If a directional
    /// clip is absent, the ordinary `attack` clip remains a safe fallback.
    pub fn with_directional_attack_animation(mut self) -> Self {
        self.directional_attack_animation = true;
        self
    }

    pub fn build(self) {
        let app = self.app;
        app.use_behavior(SimpleGameStartupBehavior {
            menu_navigation: self.menu_navigation,
        })
        .expect("simple game startup behavior should register");

        if self.debug_overlay.is_some() {
            app.configure_debug_overlay(|overlay| overlay.enabled = false);
        }

        let state_actions = StateActions {
            pause: self.pause,
            reset: self.reset,
            reload: self.reload,
            debug_kill: self.debug_kill,
            debug_overlay: self.debug_overlay,
            debug_restart: self.debug_restart,
            attack: self.attack,
        };
        app.use_behavior(StateInputBehavior {
            actions: state_actions,
        })
        .expect("state input behavior should register");

        if self.movement.is_some() {
            app.use_behavior(MovementBehavior)
                .expect("movement behavior should register");
        }

        if self.player_directional_animation || self.directional_attack_animation {
            app.use_behavior(PlayerFacingBehavior)
                .expect("player-facing behavior should register");
        }

        if self.enemy_chase {
            app.use_behavior(EnemyChaseBehavior::default())
                .expect("enemy chase behavior should register");
        }

        if self.enemy_patrol {
            app.use_behavior(EnemyPatrolBehavior)
                .expect("enemy patrol behavior should register");
        }

        if self.collision {
            app.use_behavior(CollisionBehavior)
                .expect("collision behavior should register");
        }

        if self.melee_combat {
            if let Some(attack) = self.attack {
                let config = MeleeCombatConfig {
                    attack,
                    hit_sound: self.hit_sound,
                    despawn_dead_enemies: true,
                    player_attack_animation: self.attack_animation,
                    directional_player_attack_animation: self.directional_attack_animation,
                };
                app.use_behavior(MeleeCombatBehavior { config })
                    .expect("melee combat behavior should register");
            } else {
                log::warn!("with_melee_combat() was enabled but no attack action was configured");
            }
        }

        if self.directional_attack_animation && !self.melee_combat {
            if let Some(attack) = self.attack {
                app.use_behavior(DirectionalAttackBehavior {
                    attack,
                    fallback: self.attack_animation,
                })
                .expect("directional attack behavior should register");
            } else {
                log::warn!(
                    "with_directional_attack_animation() was enabled but no attack action was configured"
                );
            }
        }

        if self.player_animation_by_movement {
            app.use_behavior(PlayerAnimationByMovementBehavior)
                .expect("player animation behavior should register");
        }

        if self.enemy_animation_by_movement {
            app.use_behavior(EnemyAnimationByMovementBehavior)
                .expect("enemy animation behavior should register");
        }

        if self.player_directional_animation {
            app.use_behavior(PlayerDirectionalAnimationBehavior)
                .expect("player directional animation behavior should register");
        }

        if self.enemy_directional_animation {
            app.use_behavior(EnemyDirectionalAnimationBehavior)
                .expect("enemy directional animation behavior should register");
        }

        app.use_behavior(AnimationUpdateBehavior)
            .expect("animation update behavior should register");

        app.use_behavior(DeathStateBehavior)
            .expect("death state behavior should register");

        if let Some((zoom_in, zoom_out)) = self.zoom {
            app.use_behavior(CameraZoomBehavior { zoom_in, zoom_out })
                .expect("camera zoom behavior should register");
        }
        if self.camera_follow {
            app.use_behavior(CameraFollowBehavior)
                .expect("camera follow behavior should register");
        }

        if self.pause_death_ui {
            app.use_behavior(PauseDeathUiBehavior)
                .expect("pause and death UI behavior should register");
        }

        app.use_behavior(CameraShakeBehavior)
            .expect("camera shake behavior should register");

        app.fixed_systems_are_pause_guarded();
    }
}

/// Installs the beginner state and spawns the start map.
pub struct SimpleGameStartupBehavior {
    pub menu_navigation: Option<(ActionId, ActionId, ActionId)>,
}

impl GamePlugin for SimpleGameStartupBehavior {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        game.on_start(startup_simple_game);
        let menu_navigation = self.menu_navigation;
        game.on_start(move |game| {
            game.init_resource::<UiFocus>();
            if let Some((up, down, accept)) = menu_navigation {
                game.insert_resource(UiNavigation::new(up, down, accept));
            }
            Ok(())
        });
        Ok(())
    }
}

pub(crate) struct StateInputBehavior {
    actions: StateActions,
}

impl GamePlugin for StateInputBehavior {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        let actions = self.actions;
        game.every_tick(move |game, _dt| state_input_system(game, actions));
        Ok(())
    }
}

/// Drives a player movement axis while the simple game is active.
pub struct MovementBehavior;

impl GamePlugin for MovementBehavior {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        game.every_active_tick::<SimpleGameState>(|game, _dt| {
            game.drive_input::<PlayerMovement, Speed>();
        });
        Ok(())
    }
}

/// Makes enemies chase the player while the simple game is active.
///
/// Set `range` to override each chasing prefab's authored aggro radius; leave
/// it as `None` to preserve the prefab's own `.chase_range(...)` setting.
#[derive(Clone, Copy, Debug, Default)]
pub struct EnemyChaseBehavior {
    pub range: Option<f32>,
}

impl GamePlugin for EnemyChaseBehavior {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        let range = self.range.map(|range| range.max(0.0));
        game.every_active_tick::<SimpleGameState>(move |game, dt| {
            if let Some(range) = range {
                for id in game.entities_with::<ChaseTarget>() {
                    if let Some(chase) = game.component_mut::<ChaseTarget>(id) {
                        chase.aggro_radius = range;
                    }
                }
            }
            game.chase_first::<Player>(dt);
        });
        Ok(())
    }
}

/// Advances authored enemy patrol paths while the simple game is active.
pub struct EnemyPatrolBehavior;

impl GamePlugin for EnemyPatrolBehavior {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        game.every_active_tick::<SimpleGameState>(|game, dt| game.run_patrol(dt));
        Ok(())
    }
}

/// Applies collision-aware movement while the simple game is active.
pub struct CollisionBehavior;

impl GamePlugin for CollisionBehavior {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        game.every_active_tick::<SimpleGameState>(|game, dt| game.move_and_collide(dt));
        Ok(())
    }
}

/// Runs the standard player/enemy melee rules while the simple game is active.
pub struct MeleeCombatBehavior {
    pub config: MeleeCombatConfig,
}

impl GamePlugin for MeleeCombatBehavior {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        let config = self.config;
        game.every_active_tick::<SimpleGameState>(move |game, dt| {
            game.run_melee_combat(config, dt);
        });
        Ok(())
    }
}

/// Tracks the latest movement direction for directional player animations.
pub struct PlayerFacingBehavior;

impl GamePlugin for PlayerFacingBehavior {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        game.every_active_tick::<SimpleGameState>(update_player_facing_direction_system);
        Ok(())
    }
}

/// Plays directional player attack clips when the configured action is pressed.
pub struct DirectionalAttackBehavior {
    pub attack: ActionId,
    pub fallback: Option<&'static str>,
}

impl GamePlugin for DirectionalAttackBehavior {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        let attack = self.attack;
        let fallback = self.fallback;
        game.every_active_tick::<SimpleGameState>(move |game, _dt| {
            if game.pressed(attack) {
                game.play_player_attack_animation(true, fallback);
            }
        });
        Ok(())
    }
}

/// Updates ordinary player walk and idle animations every frame.
pub struct PlayerAnimationByMovementBehavior;

impl GamePlugin for PlayerAnimationByMovementBehavior {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        game.every_active_frame::<SimpleGameState>(player_animation_by_movement_system);
        Ok(())
    }
}

/// Updates ordinary enemy walk and idle animations every frame.
pub struct EnemyAnimationByMovementBehavior;

impl GamePlugin for EnemyAnimationByMovementBehavior {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        game.every_active_frame::<SimpleGameState>(enemy_animation_by_movement_system);
        Ok(())
    }
}

/// Updates player directional walk animations every frame.
pub struct PlayerDirectionalAnimationBehavior;

impl GamePlugin for PlayerDirectionalAnimationBehavior {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        game.every_active_frame::<SimpleGameState>(player_directional_animation_system);
        Ok(())
    }
}

/// Updates enemy directional walk animations every frame.
pub struct EnemyDirectionalAnimationBehavior;

impl GamePlugin for EnemyDirectionalAnimationBehavior {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        game.every_active_frame::<SimpleGameState>(enemy_directional_animation_system);
        Ok(())
    }
}

/// Advances all configured sprite animations while the simple game is active.
pub struct AnimationUpdateBehavior;

impl GamePlugin for AnimationUpdateBehavior {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        game.every_active_frame::<SimpleGameState>(|game, dt| game.update_animations(dt));
        Ok(())
    }
}

/// Keeps the camera centered on the player each frame.
pub struct CameraFollowBehavior;

impl GamePlugin for CameraFollowBehavior {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        game.every_frame(|game, _dt| game.camera_follow_first::<Player>());
        Ok(())
    }
}

/// Applies zoom actions each frame.
pub struct CameraZoomBehavior {
    pub zoom_in: ActionId,
    pub zoom_out: ActionId,
}

impl GamePlugin for CameraZoomBehavior {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        let zoom_in = self.zoom_in;
        let zoom_out = self.zoom_out;
        game.every_frame(move |game, dt| game.zoom_camera_from_actions(zoom_in, zoom_out, dt));
        Ok(())
    }
}

/// Draws the simple game's pause and death notices.
pub struct PauseDeathUiBehavior;

impl GamePlugin for PauseDeathUiBehavior {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        game.draw_ui(pause_death_ui_system);
        Ok(())
    }
}

/// Keeps the `player_dead` state field synchronized with player health.
pub struct DeathStateBehavior;

impl GamePlugin for DeathStateBehavior {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        game.every_tick(death_state_system);
        Ok(())
    }
}

/// Advances camera-shake effects every frame.
pub struct CameraShakeBehavior;

impl GamePlugin for CameraShakeBehavior {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        game.every_frame(|game, dt| game.update_camera_shake(dt));
        Ok(())
    }
}

#[derive(Clone, Copy)]
struct StateActions {
    pause: Option<ActionId>,
    reset: Option<ActionId>,
    reload: Option<ActionId>,
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

    if development_reload_enabled() && pressed(game, actions.reload) {
        game.reload_tuning_if_configured_or_log();
        game.reload_current_map_or_log();
        game.reload_assets();
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

fn development_reload_enabled() -> bool {
    cfg!(debug_assertions)
        || matches!(
            std::env::var("GAME_DEV_RELOAD")
                .ok()
                .as_deref()
                .map(str::trim),
            Some("1" | "true" | "yes" | "on")
        )
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

fn update_player_facing_direction_system(game: &mut GameCtx<'_, '_>, _dt: f32) {
    for id in game.entities_with::<Player>() {
        let direction = game
            .component::<Velocity>(id)
            .and_then(|velocity| FacingDirection::from_motion(velocity.0));
        if let Some(direction) = direction {
            if let Some(facing) = game.component_mut::<FacingDirection>(id) {
                *facing = direction;
            }
        }
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

pub(crate) fn player_directional_animation_system(game: &mut GameCtx<'_, '_>, _dt: f32) {
    for id in game.entities_with::<Player>() {
        animate_directionally(game, id);
    }
}

pub(crate) fn enemy_directional_animation_system(game: &mut GameCtx<'_, '_>, _dt: f32) {
    for id in game.entities_with::<Enemy>() {
        if game.is_dead(id) {
            continue;
        }
        animate_directionally(game, id);
    }
}

fn animate_directionally(game: &mut GameCtx<'_, '_>, id: EntityId) {
    if one_shot_animation_is_active(game, id) {
        return;
    }
    let velocity = game
        .component::<Velocity>(id)
        .map(|velocity| velocity.0)
        .unwrap_or_default();
    if velocity.length_squared() <= MOVEMENT_ANIMATION_EPSILON_SQUARED {
        game.play_animation(id, "idle");
        return;
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
    // A prefab may intentionally omit some directions. In that case, use its
    // ordinary walk clip rather than freezing on the prior direction.
    if !game.play_animation(id, name) {
        game.play_animation(id, "walk");
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

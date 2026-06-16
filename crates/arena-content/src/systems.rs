use game_kit::prelude::*;

use crate::actor::{MoveSpeed, PlayerController};
use crate::assets::ArenaAssets;
use crate::combat;
use crate::input::ArenaActions;
use crate::state::GameState;

pub fn register(game: &mut GameApp, assets: ArenaAssets, actions: ArenaActions) {
    game.startup(startup_system);

    game.fixed(move |ctx: &mut GameCtx, _dt| state_input_system(ctx, actions));
    game.fixed_active::<GameState>(player_control_system);
    game.fixed_active::<GameState>(chase_player_system);
    game.fixed_active::<GameState>(physics_system);

    let hit_sound = assets.hit;
    game.fixed_active::<GameState>(move |ctx: &mut GameCtx, dt| {
        combat_system(ctx, actions.attack, hit_sound, dt);
    });
    game.fixed(death_state_system);

    game.update(move |ctx: &mut GameCtx, dt| camera_follow_player_system(ctx, actions, dt));
    game.ui(pause_death_ui_system);

    game.fixed_systems_are_pause_guarded();
}

pub fn startup_system(game: &mut StartupGameCtx<'_, '_>) -> anyhow::Result<()> {
    game.init_resource::<GameState>();
    game.spawn_start_map()
}

pub fn state_input_system(game: &mut GameCtx<'_, '_>, actions: ArenaActions) {
    let mut state = game.resource::<GameState>().copied().unwrap_or_default();

    if game.pressed(actions.pause) {
        state.paused = !state.paused;
    }

    if game.pressed(actions.reset) {
        game.reset_to_start_map_or_log();
        state = GameState::default();
    }

    if game.pressed(actions.debug_die) {
        combat::kill_player(game);
    }

    state.player_dead = combat::player_is_dead(game);
    if state.player_dead && (game.pressed(actions.attack) || game.pressed(actions.reset)) {
        game.reset_to_start_map_or_log();
        state = GameState::default();
    }

    if state.player_dead || state.paused {
        game.stop_all_velocity();
    }

    game.insert_resource(state);
}

pub fn player_control_system(game: &mut GameCtx<'_, '_>, _dt: f32) {
    game.drive_input::<PlayerController, MoveSpeed>();
}

pub fn chase_player_system(game: &mut GameCtx<'_, '_>, dt: f32) {
    game.chase_first::<PlayerController>(dt);
}

pub fn physics_system(game: &mut GameCtx<'_, '_>, dt: f32) {
    game.move_and_collide(dt);
}

pub fn combat_system(
    game: &mut GameCtx<'_, '_>,
    attack: ActionId,
    hit_sound: SoundHandle,
    dt: f32,
) {
    combat::tick_commands(game, attack, hit_sound, dt);
}

pub fn death_state_system(game: &mut GameCtx<'_, '_>, _dt: f32) {
    let mut state = game.resource::<GameState>().copied().unwrap_or_default();
    state.player_dead = combat::player_is_dead(game);
    game.insert_resource(state);
}

pub fn camera_follow_player_system(game: &mut GameCtx<'_, '_>, actions: ArenaActions, dt: f32) {
    game.zoom_camera_from_actions(actions.zoom_in, actions.zoom_out, dt);
    game.camera_follow_first::<PlayerController>();
}

pub fn pause_death_ui_system(game: &mut GameCtx<'_, '_>, _dt: f32) {
    let state = game.resource::<GameState>().copied().unwrap_or_default();
    if state.player_dead {
        game.text(
            "You died",
            glam::vec2(24.0, 24.0),
            glam::vec4(1.0, 0.35, 0.25, 1.0),
        );
    } else if state.paused {
        game.text(
            "Paused",
            glam::vec2(24.0, 24.0),
            glam::vec4(1.0, 0.95, 0.75, 1.0),
        );
    }
}

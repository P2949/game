use game_kit::prelude::*;

use crate::ai;
use crate::assets::ArenaAssets;
use crate::combat;
use crate::input::ArenaActions;
use crate::state::GameState;

pub fn register(game: &mut GameApp, assets: ArenaAssets, actions: ArenaActions) {
    game.startup(startup_system);

    game.fixed(move |ctx: &mut GameCtx, _dt| state_input_system(ctx, actions));
    game.fixed(player_control_system);
    game.fixed(chase_player_system);
    game.fixed(physics_system);

    let hit_sound = assets.hit;
    game.fixed(move |ctx: &mut GameCtx, dt| combat_system(ctx, actions.attack, hit_sound, dt));
    game.fixed(death_state_system);

    game.update(move |ctx: &mut GameCtx, dt| camera_follow_player_system(ctx, actions, dt));
    game.ui(pause_death_ui_system);

    game.fixed_systems_are_pause_guarded();
}

pub fn startup_system(game: &mut StartupGameCtx<'_, '_>) -> anyhow::Result<()> {
    game.resource_or_insert_with(GameState::default);
    game.spawn_start_map()
}

pub fn state_input_system(game: &mut GameCtx<'_, '_>, actions: ArenaActions) {
    let mut state = game.resource::<GameState>().copied().unwrap_or_default();

    if game.input().pressed(actions.pause) {
        state.paused = !state.paused;
    }

    if game.input().pressed(actions.reset) {
        game.reset_to_start_map()
            .expect("arena map objects reference registered prefabs");
        state = GameState::default();
    }

    if game.input().pressed(actions.debug_die) {
        combat::kill_player(game.world_mut());
    }

    state.player_dead = combat::player_is_dead(game.world());
    if state.player_dead
        && (game.input().pressed(actions.attack) || game.input().pressed(actions.reset))
    {
        game.reset_to_start_map()
            .expect("arena map objects reference registered prefabs");
        state = GameState::default();
    }

    if state.player_dead || state.paused {
        stop_all_velocity(game.world_mut());
    }

    game.insert_resource(state);
}

pub fn player_control_system(game: &mut GameCtx<'_, '_>, _dt: f32) {
    let state = game.resource::<GameState>().copied().unwrap_or_default();
    if state.active() {
        let (world, input) = game.world_and_input();
        ai::drive_player(world, input);
    }
}

pub fn chase_player_system(game: &mut GameCtx<'_, '_>, dt: f32) {
    let state = game.resource::<GameState>().copied().unwrap_or_default();
    if state.active() {
        let (world, nav) = game.world_and_nav();
        ai::chase_player(world, nav, dt);
    }
}

pub fn physics_system(game: &mut GameCtx<'_, '_>, dt: f32) {
    let state = game.resource::<GameState>().copied().unwrap_or_default();
    if state.active() {
        let (world, map) = game.world_and_map();
        movement_system(world, map, dt);
    }
}

pub fn combat_system(
    game: &mut GameCtx<'_, '_>,
    attack: ActionId,
    hit_sound: SoundHandle,
    dt: f32,
) {
    let state = game.resource::<GameState>().copied().unwrap_or_default();
    if state.active() {
        combat::tick_commands(game, attack, hit_sound, dt);
    }
}

pub fn death_state_system(game: &mut GameCtx<'_, '_>, _dt: f32) {
    let mut state = game.resource::<GameState>().copied().unwrap_or_default();
    state.player_dead = combat::player_is_dead(game.world());
    game.insert_resource(state);
}

pub fn camera_follow_player_system(game: &mut GameCtx<'_, '_>, actions: ArenaActions, dt: f32) {
    let zoom_in = game.input().down(actions.zoom_in);
    let zoom_out = game.input().down(actions.zoom_out);
    if zoom_in != zoom_out {
        let zoom_step = 1.0 + 2.0 * dt;
        let mut zoom = game.camera().zoom();
        if zoom_in {
            zoom *= zoom_step;
        } else {
            zoom /= zoom_step;
        }
        game.camera_mut().set_zoom(zoom.clamp(0.25, 6.0));
    }

    if let Some(pos) = ai::player_pos(game.world()) {
        game.camera_mut().set_center(pos);
    }
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

use game_kit::prelude::*;

use crate::assets::TestbedAssets;
use crate::combat;
use crate::input::TestbedActions;
use crate::state::GameState;

pub fn register(game: &mut GameApp, assets: TestbedAssets, actions: TestbedActions) {
    game.startup(startup_system);

    game.fixed(move |ctx: &mut GameCtx, _dt| state_input_system(ctx, actions));
    game.fixed(player_control_system);
    game.fixed(chase_system);
    game.fixed(patrol_system);
    game.fixed(physics_system);

    let hit_sound = assets.hit;
    game.fixed(move |ctx: &mut GameCtx, dt| combat_system(ctx, actions.attack, hit_sound, dt));
    game.fixed(death_state_system);

    game.update(move |ctx: &mut GameCtx, dt| camera_follow_player_system(ctx, actions, dt));
    game.ui(testbed_ui_system);

    game.fixed_systems_are_pause_guarded();
}

pub fn startup_system(game: &mut StartupGameCtx<'_, '_>) -> anyhow::Result<()> {
    game.resource_or_insert_with(GameState::default);
    game.spawn_start_map()
}

pub fn state_input_system(game: &mut GameCtx<'_, '_>, actions: TestbedActions) {
    let mut state = game.resource::<GameState>().copied().unwrap_or_default();

    if game.input().pressed(actions.pause) {
        state.paused = !state.paused;
    }

    if game.input().pressed(actions.reset) {
        game.reset_to_start_map()
            .expect("testbed map objects reference registered prefabs");
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
            .expect("testbed map objects reference registered prefabs");
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
        crate::ai::drive_player(world, input);
    }
}

pub fn chase_system(game: &mut GameCtx<'_, '_>, dt: f32) {
    let state = game.resource::<GameState>().copied().unwrap_or_default();
    if state.active() {
        let target = crate::ai::player_pos(game.world());
        let (world, nav) = game.world_and_nav();
        game_kit::prelude::chase_system(world, nav, target, dt);
    }
}

pub fn patrol_system(game: &mut GameCtx<'_, '_>, dt: f32) {
    let state = game.resource::<GameState>().copied().unwrap_or_default();
    if state.active() {
        game_kit::prelude::patrol_system(game.world_mut(), dt);
    }
}

pub fn physics_system(game: &mut GameCtx<'_, '_>, dt: f32) {
    let state = game.resource::<GameState>().copied().unwrap_or_default();
    if state.active() {
        let (world, map) = game.world_and_map();
        game_kit::prelude::movement_system(world, map, dt);
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

pub fn camera_follow_player_system(game: &mut GameCtx<'_, '_>, actions: TestbedActions, dt: f32) {
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

    camera_follow_first::<crate::actor::PlayerController>(game);
}

pub fn testbed_ui_system(game: &mut GameCtx<'_, '_>, _dt: f32) {
    let state = game.resource::<GameState>().copied().unwrap_or_default();
    let label = if state.player_dead {
        "TESTBED - you died"
    } else if state.paused {
        "TESTBED - paused"
    } else {
        "TESTBED"
    };
    game.text(label, vec2(24.0, 24.0), vec4(0.7, 1.0, 0.8, 1.0));
}

#[cfg(test)]
mod tests {
    use game_kit::prelude::*;

    use crate::TestbedPlugin;
    use crate::state::GameState;

    #[test]
    fn startup_spawns_player_and_two_enemies() {
        let game = GameTestHarness::from_plugin(TestbedPlugin).unwrap();

        assert!(game.world().get_resource::<GameState>().is_some());
        assert_eq!(game.world().ids().count(), 3);
        assert_eq!(game.world().ids_with::<Patrol>().len(), 1);
    }

    #[test]
    fn patrol_enemy_moves_when_simulation_active() {
        let mut game = GameTestHarness::from_plugin(TestbedPlugin).unwrap();

        game.fixed_step(1.0 / 120.0);

        let patroller = game.world().ids_with::<Patrol>()[0];
        let velocity = game.world().get::<Velocity>(patroller).unwrap().0;
        assert!(velocity.length() > 0.0, "patroller should be moving");
    }

    #[test]
    fn ui_renders_distinct_testbed_label() {
        let mut game = GameTestHarness::from_plugin(TestbedPlugin).unwrap();
        game.frame(1.0 / 120.0);
        assert_eq!(game.ui_text(), vec!["TESTBED".to_owned()]);
    }
}

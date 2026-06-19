use game_kit::advanced::testing::prelude::*;
use testbed_content::TestbedPlugin;
use testbed_content::state::GameState;

#[test]
fn startup_spawns_player_and_two_enemies() {
    let game = GameTestHarness::from_plugin(TestbedPlugin).unwrap();

    assert!(game.has_resource::<GameState>());
    assert_eq!(game.entity_count(), 3);
    assert_eq!(game.count::<Patrol>(), 1);
}

#[test]
fn patrol_enemy_moves_when_simulation_active() {
    let mut game = GameTestHarness::from_plugin(TestbedPlugin).unwrap();

    game.step();

    let patroller = game.world().ids_with::<Patrol>()[0];
    let velocity = game.world().get::<Velocity>(patroller).unwrap().0;
    assert!(velocity.length() > 0.0, "patroller should be moving");
}

#[test]
fn ui_renders_distinct_testbed_label() {
    let mut game = GameTestHarness::from_plugin(TestbedPlugin).unwrap();
    game.frame(1.0 / 120.0);
    game.assert_ui_contains("TESTBED");
}

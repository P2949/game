use game_kit::testing::prelude::*;
use testbed_content::TestbedPlugin;
use testbed_content::state::GameState;

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

use game_kit::testing::prelude::*;
use testbed_content::TestbedPlugin;

fn move_first_enemy_next_to_player(game: &mut GameTestHarness) {
    game.move_enemy_next_to_player(0);
    game.set_enemy_health(0, 25);
}

#[test]
fn player_attack_records_dead_enemy_despawn_effect() {
    let mut game = GameTestHarness::from_plugin(TestbedPlugin).unwrap();
    move_first_enemy_next_to_player(&mut game);

    game.tap_action("attack");

    assert_eq!(game.enemy_count(), 1);
    assert_eq!(game.sound_count(), 1);
}

#[test]
fn enemy_in_range_damages_player() {
    let mut game = GameTestHarness::from_plugin(TestbedPlugin).unwrap();
    move_first_enemy_next_to_player(&mut game);

    game.step();

    assert!(game.player().health() < game.player().max_health());
    assert_eq!(game.sound_count(), 1);
}

#[test]
fn debug_kill_marks_player_dead() {
    let mut game = GameTestHarness::from_plugin(TestbedPlugin).unwrap();

    game.tap_action("debug_die");

    assert!(game.player().is_dead());
}

#[test]
fn no_combat_events_when_idle_and_out_of_range() {
    let mut game = GameTestHarness::from_plugin(TestbedPlugin).unwrap();

    game.step();

    assert_eq!(game.sound_count(), 0);
}

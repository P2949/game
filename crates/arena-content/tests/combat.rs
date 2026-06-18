use arena_content::ArenaPlugin;
use game_kit::testing::prelude::*;

fn move_enemy_next_to_player(game: &mut GameTestHarness) {
    game.move_enemy_next_to_player(0);
    game.set_enemy_health(0, 25);
}

#[test]
fn player_attack_damages_and_despawns_dead_enemy() {
    let mut game = GameTestHarness::from_plugin(ArenaPlugin).unwrap();
    move_enemy_next_to_player(&mut game);

    game.tap_action("attack");

    assert_eq!(game.enemy_count(), 0);
    assert_eq!(game.sound_count(), 1);
}

#[test]
fn enemy_attack_damages_player() {
    let mut game = GameTestHarness::from_plugin(ArenaPlugin).unwrap();
    move_enemy_next_to_player(&mut game);

    game.step();

    assert_eq!(game.player().health(), 94);
    assert_eq!(game.sound_count(), 1);
}

#[test]
fn debug_kill_marks_player_dead() {
    let mut game = GameTestHarness::from_plugin(ArenaPlugin).unwrap();

    game.tap_action("debug_die");

    assert!(game.player().is_dead());
}

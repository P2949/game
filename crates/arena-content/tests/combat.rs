use arena_content::ArenaPlugin;
use arena_content::actor::{EnemyTag, PlayerController};
use game_kit::testing::prelude::*;

fn player_id(game: &GameTestHarness) -> EntityId {
    game.world().ids_with::<PlayerController>()[0]
}

fn enemy_id(game: &GameTestHarness) -> EntityId {
    game.world().ids_with::<EnemyTag>()[0]
}

fn move_enemy_next_to_player(game: &mut GameTestHarness) {
    let player = player_id(game);
    let enemy = enemy_id(game);
    let player_pos = game.world().get::<Transform>(player).unwrap().pos;
    game.world_mut().get_mut::<Transform>(enemy).unwrap().pos = player_pos + vec2(10.0, 0.0);
    game.world_mut().get_mut::<Health>(enemy).unwrap().current = 25;
}

#[test]
fn player_attack_damages_and_despawns_dead_enemy() {
    let mut game = GameTestHarness::from_plugin(ArenaPlugin)
        .unwrap()
        .press_action("attack");
    move_enemy_next_to_player(&mut game);

    game.fixed_step(1.0 / 120.0);

    assert!(game.world().ids_with::<EnemyTag>().is_empty());
    assert_eq!(game.audio_commands().len(), 1);
}

#[test]
fn enemy_attack_damages_player() {
    let mut game = GameTestHarness::from_plugin(ArenaPlugin).unwrap();
    move_enemy_next_to_player(&mut game);

    game.fixed_step(1.0 / 120.0);

    let player = player_id(&game);
    assert_eq!(game.world().get::<Health>(player).unwrap().current, 94);
    assert_eq!(game.audio_commands().len(), 1);
}

#[test]
fn debug_kill_marks_player_dead() {
    let mut game = GameTestHarness::from_plugin(ArenaPlugin)
        .unwrap()
        .press_action("debug_die");

    game.fixed_step(1.0 / 120.0);

    let player = player_id(&game);
    assert!(game.world().get::<Health>(player).unwrap().is_dead());
}

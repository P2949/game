use arena_content::ArenaPlugin;
use game_kit::beginner::testing::prelude::*;

#[test]
fn arena_plugin_builds_and_spawns_start_map() {
    let game = GameTestHarness::from_plugin(ArenaPlugin).unwrap();

    assert_eq!(game.player_count(), 1);
    game.assert_enemy_count(1);
}

use arena_content::ArenaPlugin;
use arena_content::actor::{EnemyTag, PlayerController};
use game_kit::testing::prelude::*;

#[test]
fn arena_plugin_builds_and_spawns_start_map() {
    let game = GameTestHarness::from_plugin(ArenaPlugin).unwrap();

    assert_eq!(game.world().ids_with::<PlayerController>().len(), 1);
    assert_eq!(game.world().ids_with::<EnemyTag>().len(), 1);
}

use game_kit::testing::prelude::*;
use testbed_content::TestbedPlugin;

#[test]
fn testbed_plugin_builds_and_spawns_start_map() {
    let game = GameTestHarness::from_plugin(TestbedPlugin).unwrap();

    assert_eq!(game.player_count(), 1);
    assert_eq!(game.enemy_count(), 2);
    assert_eq!(game.count::<Faction>(), 3);
}

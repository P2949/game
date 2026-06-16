use game_kit::testing::prelude::*;
use testbed_content::TestbedPlugin;
use testbed_content::actor::PlayerController;

#[test]
fn testbed_plugin_builds_and_spawns_start_map() {
    let game = GameTestHarness::from_plugin(TestbedPlugin).unwrap();

    assert_eq!(game.world().ids_with::<PlayerController>().len(), 1);
    assert_eq!(game.world().ids_with::<Faction>().len(), 3);
}

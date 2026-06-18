use game_kit::testing::prelude::*;
use simple_content::SimplePlugin;

#[test]
fn simple_plugin_builds_and_spawns_start_map() {
    let game = GameTestHarness::from_plugin(SimplePlugin).unwrap();

    assert_eq!(game.current_map_name(), Some("simple".to_owned()));
}

#[test]
fn debug_overlay_hotkey_shows_basic_debug_text() {
    let mut game = GameTestHarness::from_plugin(SimplePlugin).unwrap();

    game.tap_action("debug_overlay");
    game.frame(1.0 / 60.0);

    game.assert_ui_contains("map: simple");
    game.assert_ui_contains("player hp:");
}

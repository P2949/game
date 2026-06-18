use arena_content::ArenaPlugin;
use game_kit::testing::prelude::*;

#[test]
fn reset_respawns_start_map_objects_and_reinserts_runtime() {
    let mut game = GameTestHarness::from_plugin(ArenaPlugin).unwrap();
    let initial_count = game.entity_count();
    assert_eq!(game.current_map_name(), Some("arena".to_owned()));

    game.reset_to_start_map().unwrap();

    assert_eq!(game.entity_count(), initial_count);
    assert_eq!(game.current_map_name(), Some("arena".to_owned()));
}

#[test]
fn reset_clears_queued_commands_before_respawned_world_steps() {
    let mut game = GameTestHarness::from_plugin(ArenaPlugin).unwrap();
    let enemy = game.enemy(0);

    game.queue_despawn_entity(enemy);
    game.queue_play_sound(SoundHandle(99));
    game.reset_to_start_map().unwrap();
    game.step();

    assert_eq!(game.enemy_count(), 1);
    assert_eq!(game.sound_count(), 0);
}

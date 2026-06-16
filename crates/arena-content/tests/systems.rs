use arena_content::ArenaPlugin;
use arena_content::actor::EnemyTag;
use game_kit::testing::prelude::*;

#[test]
fn reset_respawns_start_map_objects_and_reinserts_runtime() {
    let mut game = GameTestHarness::from_plugin(ArenaPlugin).unwrap();
    let initial_count = game.world().ids().count();
    assert_eq!(game.current_map_name(), Some("arena".to_owned()));

    game.reset_to_start_map().unwrap();

    assert_eq!(game.world().ids().count(), initial_count);
    assert_eq!(game.current_map_name(), Some("arena".to_owned()));
}

#[test]
fn reset_clears_queued_commands_before_respawned_world_steps() {
    let mut game = GameTestHarness::from_plugin(ArenaPlugin).unwrap();
    let enemy = game.world().ids_with::<EnemyTag>()[0];

    game.queue_despawn(enemy);
    game.queue_play_sound(SoundHandle(99));
    game.reset_to_start_map().unwrap();
    game.fixed_step(1.0 / 120.0);

    assert_eq!(game.world().ids_with::<EnemyTag>().len(), 1);
    assert!(game.audio_commands().is_empty());
}

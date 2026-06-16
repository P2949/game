use game_kit::testing::prelude::*;
use testbed_content::TestbedPlugin;
use testbed_content::actor::PlayerController;

fn player_id(game: &GameTestHarness) -> EntityId {
    game.world().ids_with::<PlayerController>()[0]
}

fn enemy_ids(game: &GameTestHarness) -> Vec<EntityId> {
    game.world()
        .ids_with::<Faction>()
        .into_iter()
        .filter(|id| game.world().get::<Faction>(*id).unwrap().0 == FactionId::Enemy)
        .collect()
}

fn move_first_enemy_next_to_player(game: &mut GameTestHarness) -> EntityId {
    let player = player_id(game);
    let enemy = enemy_ids(game)[0];
    let player_pos = game.world().get::<Transform>(player).unwrap().pos;
    game.world_mut().get_mut::<Transform>(enemy).unwrap().pos = player_pos + vec2(10.0, 0.0);
    game.world_mut().get_mut::<Health>(enemy).unwrap().current = 25;
    enemy
}

#[test]
fn player_attack_records_dead_enemy_despawn_effect() {
    let mut game = GameTestHarness::from_plugin(TestbedPlugin)
        .unwrap()
        .press_action("attack");
    let enemy = move_first_enemy_next_to_player(&mut game);

    game.fixed_step(1.0 / 120.0);

    assert!(game.world().get::<Health>(enemy).is_none());
    assert_eq!(game.audio_commands().len(), 1);
}

#[test]
fn enemy_in_range_damages_player() {
    let mut game = GameTestHarness::from_plugin(TestbedPlugin).unwrap();
    move_first_enemy_next_to_player(&mut game);

    game.fixed_step(1.0 / 120.0);

    let player = player_id(&game);
    assert!(game.world().get::<Health>(player).unwrap().current < 120);
    assert_eq!(game.audio_commands().len(), 1);
}

#[test]
fn debug_kill_marks_player_dead() {
    let mut game = GameTestHarness::from_plugin(TestbedPlugin)
        .unwrap()
        .press_action("debug_die");

    game.fixed_step(1.0 / 120.0);

    let player = player_id(&game);
    assert!(game.world().get::<Health>(player).unwrap().is_dead());
}

#[test]
fn no_combat_events_when_idle_and_out_of_range() {
    let mut game = GameTestHarness::from_plugin(TestbedPlugin).unwrap();

    game.fixed_step(1.0 / 120.0);

    assert!(game.audio_commands().is_empty());
}

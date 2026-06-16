use crate::actor::PlayerController;
use game_kit::prelude::*;

#[derive(Default)]
struct CombatEffects {
    hit_sounds: u32,
    despawns: Vec<EntityId>,
}

/// Resolves a melee combat tick into engine commands: queued hit sounds and
/// despawns of dead enemies. Enemies are identified by [`Faction`] (Enemy) rather
/// than a content-specific tag, demonstrating reuse of `game-combat`.
pub fn tick_commands(
    game: &mut GameCtx<'_, '_>,
    attack: ActionId,
    hit_sound: SoundHandle,
    dt: f32,
) {
    let effects = tick_effects(game, attack, dt);
    let mut commands = game.commands();
    for _ in 0..effects.hit_sounds {
        commands.play_sound(hit_sound);
    }
    for id in effects.despawns {
        commands.despawn(id);
    }
}

fn tick_effects(game: &mut GameCtx<'_, '_>, attack: ActionId, dt: f32) -> CombatEffects {
    let mut effects = CombatEffects::default();
    let Some((player_id, player_pos, player_range, player_damage)) = player_snapshot(game) else {
        return effects;
    };

    if game.pressed(attack) {
        if let Some(target) = nearest_enemy_in_range(game, player_pos, player_range) {
            if game.damage(target, player_damage) {
                effects.hit_sounds += 1;
            }
        }
    }

    let mut player_damage_taken = 0;
    for id in enemy_ids(game) {
        if game.is_dead(id) {
            continue;
        }
        let Some(enemy_pos) = game.position(id) else {
            continue;
        };

        let Some(attack) = game.component_mut::<MeleeAttack>(id) else {
            continue;
        };
        attack.timer = (attack.timer - dt).max(0.0);
        if attack.timer == 0.0 && enemy_pos.distance(player_pos) <= attack.range {
            attack.timer = attack.cooldown;
            player_damage_taken += attack.damage;
        }
    }

    if player_damage_taken > 0 && game.damage(player_id, player_damage_taken) {
        effects.hit_sounds += 1;
    }

    effects.despawns = enemy_ids(game)
        .into_iter()
        .filter(|id| game.is_dead(*id))
        .collect();
    effects
}

pub fn kill_player(game: &mut GameCtx<'_, '_>) {
    for id in game.entities_with::<PlayerController>() {
        if let Some(health) = game.component_mut::<Health>(id) {
            health.damage(health.current);
        }
        if let Some(velocity) = game.component_mut::<Velocity>(id) {
            velocity.0 = Vec2::ZERO;
        }
    }
}

pub fn player_is_dead(game: &GameCtx<'_, '_>) -> bool {
    game.entities_with::<PlayerController>()
        .into_iter()
        .any(|id| game.is_dead(id))
}

fn is_enemy(game: &GameCtx<'_, '_>, id: EntityId) -> bool {
    game.faction(id) == Some(FactionId::Enemy)
}

fn enemy_ids(game: &GameCtx<'_, '_>) -> Vec<EntityId> {
    game.entities_with::<Faction>()
        .into_iter()
        .filter(|id| is_enemy(game, *id))
        .collect()
}

fn player_snapshot(game: &GameCtx<'_, '_>) -> Option<(EntityId, Vec2, f32, i32)> {
    game.entities_with::<PlayerController>()
        .into_iter()
        .find_map(|id| {
            let transform = game.component::<Transform>(id)?;
            let attack = game.component::<MeleeAttack>(id)?;
            Some((id, transform.pos, attack.range, attack.damage))
        })
}

fn nearest_enemy_in_range(
    game: &GameCtx<'_, '_>,
    player_pos: Vec2,
    range: f32,
) -> Option<EntityId> {
    enemy_ids(game)
        .into_iter()
        .filter_map(|id| {
            if game.is_dead(id) {
                return None;
            }
            let transform = game.component::<Transform>(id)?;
            let dist_sq = transform.pos.distance_squared(player_pos);
            (dist_sq <= range * range).then_some((id, dist_sq))
        })
        .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(id, _)| id)
}

#[cfg(test)]
mod tests {
    use game_kit::testing::prelude::*;

    use crate::TestbedPlugin;
    use crate::actor::PlayerController;

    const ATTACK: ActionId = ActionId(0);
    const DEBUG_DIE: ActionId = ActionId(3);

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
            .press(ATTACK);
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
            .press(DEBUG_DIE);

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
}

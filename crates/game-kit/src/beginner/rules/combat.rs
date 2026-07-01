use super::shared::*;

fn enemy_drops_system(game: &mut GameCtx<'_, '_>, _dt: f32) {
    let drops = game
        .entities_with::<Enemy>()
        .into_iter()
        .filter(|id| game.is_dead(*id) && !game.has::<DropSpawned>(*id))
        .filter_map(|id| {
            let drop = game.component::<DropsPrefab>(id)?.clone();
            let position = game.position(id)?;
            Some((id, position, drop))
        })
        .collect::<Vec<_>>();

    for (enemy, position, drop) in drops {
        if drop.prefab.is_empty() || drop.chance <= 0.0 {
            continue;
        }
        // A stable position-derived roll keeps examples deterministic without
        // adding an RNG dependency. The common `.drops(...)` path always uses
        // chance 1.0.
        let roll = ((position.x.to_bits() ^ position.y.to_bits()) % 10_000) as f32 / 10_000.0;
        if drop.chance >= 1.0 || roll < drop.chance {
            game.spawn_prefab_or_log(&drop.prefab, position);
        }
        game.insert_component(enemy, DropSpawned);
    }
}

/// Queues removal of defeated enemies every fixed tick.
pub struct DeadEnemiesDespawnBehavior;

impl GamePlugin for DeadEnemiesDespawnBehavior {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        game.fixed(|game: &mut GameCtx<'_, '_>, _dt| {
            game.enemies().dead().despawn();
        });
        Ok(())
    }
}

/// Spawns configured drops from defeated enemies.
pub struct EnemyDropsBehavior;

impl GamePlugin for EnemyDropsBehavior {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        game.fixed(enemy_drops_system);
        Ok(())
    }
}

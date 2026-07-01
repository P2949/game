use super::shared::*;

struct SpawnRequest {
    prefab: String,
    placement: SpawnPlacement,
    at_spawner: Vec2,
}

fn spawners_spawn_prefabs_system(game: &mut GameCtx<'_, '_>, dt: f32) {
    let spawners = game
        .entities_with::<Spawner>()
        .into_iter()
        .filter_map(|id| {
            let spawner = game.component::<Spawner>(id)?.clone();
            let position = game.position(id)?;
            Some((id, spawner, position))
        })
        .collect::<Vec<_>>();
    let mut pending_by_prefab: HashMap<String, usize> = HashMap::new();
    let mut requests = Vec::new();

    for (id, snapshot, position) in spawners {
        let alive = count_alive_prefab(game, &snapshot.prefab);
        let already_pending = pending_by_prefab
            .get(&snapshot.prefab)
            .copied()
            .unwrap_or_default();
        let mut spawn_count = 0usize;

        if let Some(spawner) = game.component_mut::<Spawner>(id) {
            spawner.timer += dt.max(0.0);
            while spawner.timer >= spawner.every_seconds
                && spawner
                    .max_alive
                    .is_none_or(|max| alive + already_pending + spawn_count < max)
            {
                spawner.timer -= spawner.every_seconds;
                spawn_count += 1;
            }
        }

        if spawn_count > 0 {
            *pending_by_prefab
                .entry(snapshot.prefab.clone())
                .or_default() += spawn_count;
            for _ in 0..spawn_count {
                requests.push(SpawnRequest {
                    prefab: snapshot.prefab.clone(),
                    placement: snapshot.placement.clone(),
                    at_spawner: position,
                });
            }
        }
    }

    for request in requests {
        let position = match request.placement {
            SpawnPlacement::AtSpawner => Some(request.at_spawner),
            SpawnPlacement::NearPlayer { radius } => game
                .player_position()
                .map(|player| player + Vec2::new(radius, 0.0)),
            SpawnPlacement::AtFirstFloor => game.first_floor_center(),
        };
        if let Some(position) = position {
            game.spawn(request.prefab).at_world(position);
        }
    }
}

fn count_alive_prefab(game: &GameCtx<'_, '_>, prefab: &str) -> usize {
    game.entities_with::<PrefabName>()
        .into_iter()
        .filter(|id| {
            game.component::<PrefabName>(*id)
                .is_some_and(|name| name.matches(prefab))
                && !game.is_dead(*id)
        })
        .count()
}

/// Advances author-configured spawners every fixed tick.
#[derive(Clone, Copy)]
pub struct SpawnerBehavior;

impl GamePlugin for SpawnerBehavior {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        game.fixed(spawners_spawn_prefabs_system);
        Ok(())
    }
}

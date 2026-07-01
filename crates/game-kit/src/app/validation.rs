use game_core::world::{EntityId, Transform};
use game_physics::Collider;
use glam::Vec2;

use crate::beginner::actors::PrefabName;
use crate::context::GameCtx;

pub(super) fn prefab_matches(game: &GameCtx<'_, '_>, entity: EntityId, expected: &str) -> bool {
    game.component::<PrefabName>(entity)
        .is_some_and(|name| name.matches(expected))
}

pub(super) fn matching_overlaps(
    game: &GameCtx<'_, '_>,
    a_prefab: &str,
    b_prefab: &str,
) -> Vec<(EntityId, EntityId)> {
    let a_entities = game
        .entities_with::<PrefabName>()
        .into_iter()
        .filter(|entity| prefab_matches(game, *entity, a_prefab))
        .collect::<Vec<_>>();
    let b_entities = game
        .entities_with::<PrefabName>()
        .into_iter()
        .filter(|entity| prefab_matches(game, *entity, b_prefab))
        .collect::<Vec<_>>();
    let mut overlaps = Vec::new();

    for a in a_entities {
        for &b in &b_entities {
            if a != b && colliders_overlap(game, a, b) {
                overlaps.push((a, b));
            }
        }
    }
    overlaps
}

pub(super) fn matching_nearby_prefabs(
    game: &GameCtx<'_, '_>,
    a_prefab: &str,
    b_prefab: &str,
    range: f32,
) -> Vec<(EntityId, EntityId, Vec2)> {
    let range = range.max(0.0);
    let range_squared = range * range;
    let a_entities = game
        .entities_with::<PrefabName>()
        .into_iter()
        .filter(|entity| prefab_matches(game, *entity, a_prefab))
        .filter_map(|entity| game.position(entity).map(|position| (entity, position)))
        .collect::<Vec<_>>();
    let b_entities = game
        .entities_with::<PrefabName>()
        .into_iter()
        .filter(|entity| prefab_matches(game, *entity, b_prefab))
        .filter_map(|entity| game.position(entity).map(|position| (entity, position)))
        .collect::<Vec<_>>();
    let mut matches = Vec::new();

    for (a, a_position) in a_entities {
        for &(b, b_position) in &b_entities {
            if a != b && a_position.distance_squared(b_position) <= range_squared {
                matches.push((a, b, a_position));
            }
        }
    }
    matches
}

fn colliders_overlap(game: &GameCtx<'_, '_>, a: EntityId, b: EntityId) -> bool {
    let Some(a_transform) = game.component::<Transform>(a) else {
        return false;
    };
    let Some(a_collider) = game.component::<Collider>(a) else {
        return false;
    };
    let Some(b_transform) = game.component::<Transform>(b) else {
        return false;
    };
    let Some(b_collider) = game.component::<Collider>(b) else {
        return false;
    };

    let delta = a_transform.pos - b_transform.pos;
    delta.x.abs() < a_collider.half_extents.x + b_collider.half_extents.x
        && delta.y.abs() < a_collider.half_extents.y + b_collider.half_extents.y
}

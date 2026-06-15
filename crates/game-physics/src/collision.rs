use game_core::world::{EntityId, Transform, World};
use game_map::tilemap::TileMap;

use crate::collider::{Collider, Solid, Trigger};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CollisionPair {
    pub a: EntityId,
    pub b: EntityId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TriggerOverlap {
    pub trigger: EntityId,
    pub other: EntityId,
}

pub fn overlaps_wall(map: &TileMap, center: glam::Vec2, half: glam::Vec2) -> bool {
    let tile_size = map.tile_size();
    let min = center - half;
    let max = center + half;
    let col_min = (min.x / tile_size).floor() as i32;
    let col_max = ((max.x / tile_size) - 1e-4).floor() as i32;
    let row_min = (min.y / tile_size).floor() as i32;
    let row_max = ((max.y / tile_size) - 1e-4).floor() as i32;

    for row in row_min..=row_max {
        for col in col_min..=col_max {
            if map.is_wall(col, row) {
                return true;
            }
        }
    }

    false
}

pub fn collision_system(world: &World) -> Vec<CollisionPair> {
    let ids = world
        .ids()
        .filter(|id| world.has::<Collider>(*id))
        .collect::<Vec<_>>();
    let mut pairs = Vec::new();

    for (left_index, a) in ids.iter().copied().enumerate() {
        if !world.has::<Solid>(a) {
            continue;
        }

        for b in ids.iter().copied().skip(left_index + 1) {
            if !world.has::<Solid>(b) {
                continue;
            }
            if colliders_overlap(world, a, b) {
                pairs.push(CollisionPair { a, b });
            }
        }
    }

    pairs
}

pub fn trigger_overlap_system(world: &World) -> Vec<TriggerOverlap> {
    let colliders = world
        .ids()
        .filter(|id| world.has::<Collider>(*id))
        .collect::<Vec<_>>();
    let triggers = world
        .ids()
        .filter(|id| world.has::<Trigger>(*id))
        .collect::<Vec<_>>();
    let mut overlaps = Vec::new();

    for trigger in triggers {
        for other in colliders.iter().copied().filter(|other| *other != trigger) {
            if colliders_overlap(world, trigger, other) {
                overlaps.push(TriggerOverlap { trigger, other });
            }
        }
    }

    overlaps
}

fn colliders_overlap(world: &World, a: EntityId, b: EntityId) -> bool {
    let Some(a_transform) = world.get::<Transform>(a) else {
        return false;
    };
    let Some(a_collider) = world.get::<Collider>(a) else {
        return false;
    };
    let Some(b_transform) = world.get::<Transform>(b) else {
        return false;
    };
    let Some(b_collider) = world.get::<Collider>(b) else {
        return false;
    };

    let delta = a_transform.pos - b_transform.pos;
    delta.x.abs() < a_collider.half_extents.x + b_collider.half_extents.x
        && delta.y.abs() < a_collider.half_extents.y + b_collider.half_extents.y
}

#[cfg(test)]
mod tests {
    use game_core::world::{Entity, World};

    use crate::collider::{Collider, Solid, Trigger};
    use crate::collision::{collision_system, trigger_overlap_system};

    #[test]
    fn collision_system_reports_overlapping_solid_pairs() {
        let mut world = World::new();
        let a = world.spawn(
            Entity::new(glam::Vec2::ZERO)
                .with(Collider::box_of(glam::Vec2::splat(10.0)))
                .with(Solid),
        );
        let b = world.spawn(
            Entity::new(glam::vec2(4.0, 0.0))
                .with(Collider::box_of(glam::Vec2::splat(10.0)))
                .with(Solid),
        );

        assert_eq!(
            collision_system(&world),
            vec![super::CollisionPair { a, b }]
        );
    }

    #[test]
    fn trigger_overlap_system_reports_trigger_against_colliders() {
        let mut world = World::new();
        let trigger = world.spawn(
            Entity::new(glam::Vec2::ZERO)
                .with(Collider::box_of(glam::Vec2::splat(10.0)))
                .with(Trigger),
        );
        let other = world.spawn(
            Entity::new(glam::vec2(4.0, 0.0)).with(Collider::box_of(glam::Vec2::splat(10.0))),
        );

        assert_eq!(
            trigger_overlap_system(&world),
            vec![super::TriggerOverlap { trigger, other }]
        );
    }
}

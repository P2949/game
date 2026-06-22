use game_core::world::{Transform, Velocity, World};
use game_map::tilemap::TileMap;

use crate::collider::Collider;
use crate::collision::overlaps_wall;

/// Result of moving an AABB through a tile map without skipping intervening
/// collision checks. The current beginner physics path resolves horizontal
/// movement first, then vertical movement, preserving simple wall sliding.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SweptAabbMove {
    pub position: glam::Vec2,
    pub blocked_x: bool,
    pub blocked_y: bool,
}

/// Sweeps an AABB through a tile map using short, bounded substeps rather than
/// testing only the destination. This prevents normal fast movement from
/// tunnelling through thin tile walls while retaining the familiar
/// axis-separated sliding behavior: a blocked axis keeps its starting value.
pub fn sweep_aabb(
    map: &TileMap,
    start: glam::Vec2,
    half: glam::Vec2,
    delta: glam::Vec2,
) -> SweptAabbMove {
    if !delta.is_finite() {
        return SweptAabbMove {
            position: start,
            blocked_x: false,
            blocked_y: false,
        };
    }

    let mut position = start;
    let mut blocked_x = false;
    let mut blocked_y = false;
    position.x = sweep_axis(map, position, half, delta.x, true, &mut blocked_x);
    position.y = sweep_axis(map, position, half, delta.y, false, &mut blocked_y);
    SweptAabbMove {
        position,
        blocked_x,
        blocked_y,
    }
}

fn sweep_axis(
    map: &TileMap,
    mut position: glam::Vec2,
    half: glam::Vec2,
    distance: f32,
    horizontal: bool,
    blocked: &mut bool,
) -> f32 {
    let start_coordinate = if horizontal { position.x } else { position.y };
    if distance == 0.0 {
        return start_coordinate;
    }

    // A quarter tile keeps the sampled AABB from jumping over a one-tile wall,
    // including when the collider is smaller than a cell. The cap guards
    // malformed/absurd velocities without affecting ordinary game movement.
    let max_step = (map.tile_size() * 0.25).max(1.0);
    let steps = (distance.abs() / max_step).ceil().clamp(1.0, 16_384.0) as usize;
    let step = distance / steps as f32;
    for _ in 0..steps {
        let candidate = if horizontal {
            glam::vec2(position.x + step, position.y)
        } else {
            glam::vec2(position.x, position.y + step)
        };
        if overlaps_wall(map, candidate, half) {
            *blocked = true;
            return start_coordinate;
        }
        position = candidate;
    }
    if horizontal { position.x } else { position.y }
}

pub fn movement_system(world: &mut World, map: &TileMap, dt: f32) {
    if !dt.is_finite() || dt <= 0.0 {
        return;
    }

    let ids = world.ids_with::<Collider>();
    for id in ids {
        let Some(half) = world
            .get::<Collider>(id)
            .map(|collider| collider.half_extents)
        else {
            continue;
        };
        let Some(start) = world.get::<Transform>(id).map(|transform| transform.pos) else {
            continue;
        };
        let delta = world
            .get::<Velocity>(id)
            .map(|velocity| velocity.0)
            .unwrap_or(glam::Vec2::ZERO)
            * dt;

        let swept = sweep_aabb(map, start, half, delta);
        if swept.blocked_x {
            if let Some(velocity) = world.get_mut::<Velocity>(id) {
                velocity.0.x = 0.0;
            }
        }
        if swept.blocked_y {
            if let Some(velocity) = world.get_mut::<Velocity>(id) {
                velocity.0.y = 0.0;
            }
        }

        if let Some(transform) = world.get_mut::<Transform>(id) {
            transform.pos = swept.position;
        }
    }
}

#[cfg(test)]
mod tests {
    use game_core::world::{Entity, Transform, Velocity, World};
    use game_map::tilemap::TileMap;

    use crate::collider::Collider;
    use crate::movement::movement_system;

    #[test]
    fn stops_against_wall_on_blocked_axis() {
        let map = TileMap::from_rows(&[".#"], 10.0);
        let mut world = World::new();
        let id = world.spawn(
            Entity::new(glam::vec2(5.0, 5.0)).with_collider(Collider::box_of(glam::vec2(8.0, 8.0))),
        );

        world.get_mut::<Velocity>(id).unwrap().0 = glam::vec2(20.0, 0.0);
        movement_system(&mut world, &map, 1.0);

        assert_eq!(
            world.get::<Transform>(id).unwrap().pos,
            glam::vec2(5.0, 5.0)
        );
        assert_eq!(world.get::<Velocity>(id).unwrap().0.x, 0.0);
    }

    #[test]
    fn allows_slide_on_free_axis() {
        let map = TileMap::from_rows(&[".#", ".."], 10.0);
        let mut world = World::new();
        let id = world.spawn(
            Entity::new(glam::vec2(5.0, 15.0))
                .with_collider(Collider::box_of(glam::vec2(8.0, 8.0))),
        );

        world.get_mut::<Velocity>(id).unwrap().0 = glam::vec2(20.0, -5.0);
        movement_system(&mut world, &map, 1.0);

        let transform = world.get::<Transform>(id).unwrap();
        assert_eq!(transform.pos.x, 5.0);
        assert!(transform.pos.y < 15.0);
    }

    #[test]
    fn swept_aabb_stops_fast_motion_before_a_thin_wall() {
        let map = TileMap::from_rows(&["..#.."], 10.0);
        let swept = super::sweep_aabb(
            &map,
            glam::vec2(5.0, 5.0),
            glam::vec2(2.0, 2.0),
            glam::vec2(100.0, 0.0),
        );

        assert!(swept.blocked_x);
        assert!(!swept.blocked_y);
        assert!(swept.position.x < 18.0);
    }
}

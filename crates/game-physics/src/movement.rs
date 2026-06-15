use game_core::world::{Transform, Velocity, World};
use game_map::tilemap::TileMap;

use crate::collider::Collider;
use crate::collision::overlaps_wall;

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

        let mut x = start.x + delta.x;
        let mut y = start.y;
        if overlaps_wall(map, glam::Vec2::new(x, y), half) {
            x = start.x;
            if let Some(velocity) = world.get_mut::<Velocity>(id) {
                velocity.0.x = 0.0;
            }
        }

        y += delta.y;
        if overlaps_wall(map, glam::Vec2::new(x, y), half) {
            y = start.y;
            if let Some(velocity) = world.get_mut::<Velocity>(id) {
                velocity.0.y = 0.0;
            }
        }

        if let Some(transform) = world.get_mut::<Transform>(id) {
            transform.pos = glam::Vec2::new(x, y);
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
}

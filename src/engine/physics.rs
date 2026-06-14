use glam::Vec2;

use crate::engine::tilemap::TileMap;
use crate::engine::world::World;

pub fn step<U>(world: &mut World<U>, map: &TileMap, dt: f32) {
    if !dt.is_finite() || dt <= 0.0 {
        return;
    }

    let ids = world.ids_where(|entity| entity.collider.is_some());
    for id in ids {
        let Some(entity) = world.get_mut(id) else {
            continue;
        };
        let half = entity
            .collider
            .expect("ids_where selected only collider holders")
            .half_extents;
        let start = entity.transform.pos;
        let delta = entity.velocity * dt;

        let mut x = start.x + delta.x;
        let mut y = start.y;
        if overlaps_wall(map, Vec2::new(x, y), half) {
            x = start.x;
            entity.velocity.x = 0.0;
        }

        y += delta.y;
        if overlaps_wall(map, Vec2::new(x, y), half) {
            y = start.y;
            entity.velocity.y = 0.0;
        }

        entity.transform.pos = Vec2::new(x, y);
    }
}

fn overlaps_wall(map: &TileMap, center: Vec2, half: Vec2) -> bool {
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

#[cfg(test)]
mod tests {
    use crate::engine::physics::step;
    use crate::engine::tilemap::TileMap;
    use crate::engine::world::{Collider, Entity, World};

    #[test]
    fn stops_against_wall_on_blocked_axis() {
        let map = TileMap::from_rows(&[".#"], 10.0);
        let mut world = World::new();
        let id = world.spawn(
            Entity::new(glam::vec2(5.0, 5.0), ())
                .with_collider(Collider::box_of(glam::vec2(8.0, 8.0))),
        );

        world.get_mut(id).unwrap().velocity = glam::vec2(20.0, 0.0);
        step(&mut world, &map, 1.0);

        let entity = world.get(id).unwrap();
        assert_eq!(entity.transform.pos, glam::vec2(5.0, 5.0));
        assert_eq!(entity.velocity.x, 0.0);
    }

    #[test]
    fn allows_slide_on_free_axis() {
        let map = TileMap::from_rows(&[".#", ".."], 10.0);
        let mut world = World::new();
        let id = world.spawn(
            Entity::new(glam::vec2(5.0, 15.0), ())
                .with_collider(Collider::box_of(glam::vec2(8.0, 8.0))),
        );

        world.get_mut(id).unwrap().velocity = glam::vec2(20.0, -5.0);
        step(&mut world, &map, 1.0);

        let entity = world.get(id).unwrap();
        assert_eq!(entity.transform.pos.x, 5.0);
        assert!(entity.transform.pos.y < 15.0);
    }
}

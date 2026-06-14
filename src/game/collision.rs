//! Simple discrete axis-aligned bounding-box (AABB) collision.
//!
//! Movement is resolved by integrating the X and Y axes separately and snapping
//! the entity out of any solid it ends a step overlapping. This is sufficient
//! and stable for the current slow, actor-style motion, but it is *discrete*:
//! it only checks the entity's final position each step, so a fast-moving entity
//! can tunnel straight through a thin solid within a single step. Before adding
//! high-speed movement (projectiles, dashes, knockback, fast enemies, moving
//! platforms), switch to a swept AABB or sub-stepped integration. The
//! `fast_entity_tunnels_through_thin_solid` test documents this limitation
//! intentionally rather than treating it as a bug.

use crate::game::world::Entity;

#[derive(Debug, Clone, Copy)]
pub struct Aabb {
    pub min: glam::Vec2,
    pub max: glam::Vec2,
}

impl Aabb {
    pub fn from_pos_size(pos: glam::Vec2, size: glam::Vec2) -> Self {
        Self {
            min: pos,
            max: pos + size,
        }
    }

    pub fn overlaps(self, other: Self) -> bool {
        self.min.x < other.max.x
            && self.max.x > other.min.x
            && self.min.y < other.max.y
            && self.max.y > other.min.y
    }
}

/// Integrates `entity` by one discrete step against `solids` (see the module
/// docs for the tunneling caveat). The X and Y axes are resolved independently
/// so an entity slides along walls instead of sticking on contact.
pub fn move_with_collision(entity: &mut Entity, solids: &[Aabb], dt: f32) {
    entity.prev_pos = entity.pos;

    entity.pos.x += entity.vel.x * dt;
    for solid in solids {
        let aabb = Aabb::from_pos_size(entity.pos, entity.size);
        if aabb.overlaps(*solid) {
            if entity.vel.x > 0.0 {
                entity.pos.x = solid.min.x - entity.size.x;
            } else if entity.vel.x < 0.0 {
                entity.pos.x = solid.max.x;
            }
            entity.vel.x = 0.0;
        }
    }

    entity.pos.y += entity.vel.y * dt;
    for solid in solids {
        let aabb = Aabb::from_pos_size(entity.pos, entity.size);
        if aabb.overlaps(*solid) {
            if entity.vel.y > 0.0 {
                entity.pos.y = solid.min.y - entity.size.y;
            } else if entity.vel.y < 0.0 {
                entity.pos.y = solid.max.y;
            }
            entity.vel.y = 0.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Aabb, move_with_collision};
    use crate::game::world::Entity;

    fn entity(pos: glam::Vec2, vel: glam::Vec2, size: glam::Vec2) -> Entity {
        Entity {
            pos,
            prev_pos: pos,
            vel,
            size,
            sprite: crate::renderer::TEST_TEXTURE_ID,
        }
    }

    #[test]
    fn overlapping_aabbs_intersect() {
        let a = Aabb::from_pos_size(glam::vec2(0.0, 0.0), glam::vec2(10.0, 10.0));
        let b = Aabb::from_pos_size(glam::vec2(5.0, 5.0), glam::vec2(10.0, 10.0));

        assert!(a.overlaps(b));
    }

    #[test]
    fn separated_aabbs_do_not_intersect() {
        let a = Aabb::from_pos_size(glam::vec2(0.0, 0.0), glam::vec2(10.0, 10.0));
        let b = Aabb::from_pos_size(glam::vec2(10.0, 0.0), glam::vec2(10.0, 10.0));

        assert!(!a.overlaps(b));
    }

    #[test]
    fn slow_entity_is_stopped_at_a_solid() {
        let wall = Aabb::from_pos_size(glam::vec2(50.0, -50.0), glam::vec2(10.0, 100.0));
        let mut e = entity(
            glam::vec2(0.0, 0.0),
            glam::Vec2::ZERO,
            glam::vec2(10.0, 10.0),
        );

        // Drive the entity rightward in small steps; collision zeroes velocity, so
        // re-apply it each step the way the game loop does.
        for _ in 0..100 {
            e.vel.x = 100.0;
            move_with_collision(&mut e, &[wall], 0.05);
        }

        // Snapped flush against the wall's left edge: solid.min.x - size.x.
        assert!((e.pos.x - 40.0).abs() < 1e-3, "got {}", e.pos.x);
    }

    #[test]
    fn entity_slides_along_a_wall_on_the_free_axis() {
        // A wall to the right should stop X motion but not Y motion.
        let wall = Aabb::from_pos_size(glam::vec2(50.0, -1000.0), glam::vec2(10.0, 2000.0));
        let mut e = entity(
            glam::vec2(45.0, 0.0),
            glam::vec2(100.0, 100.0),
            glam::vec2(10.0, 10.0),
        );

        move_with_collision(&mut e, &[wall], 0.1);

        assert!((e.pos.x - 40.0).abs() < 1e-3, "x blocked: {}", e.pos.x);
        assert!(e.pos.y > 0.0, "y should slide freely: {}", e.pos.y);
    }

    #[test]
    fn fast_entity_tunnels_through_thin_solid() {
        // Documents the known discrete-collision limitation: a single large step
        // jumps past a thin solid without ever overlapping it.
        let wall = Aabb::from_pos_size(glam::vec2(100.0, -50.0), glam::vec2(2.0, 100.0));
        let mut e = entity(
            glam::vec2(0.0, 0.0),
            glam::vec2(20_000.0, 0.0),
            glam::vec2(10.0, 10.0),
        );

        move_with_collision(&mut e, &[wall], 0.016);

        assert!(e.pos.x > 102.0, "expected tunneling, got {}", e.pos.x);
    }
}

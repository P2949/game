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
    min: glam::Vec2,
    max: glam::Vec2,
}

impl Aabb {
    pub fn new(pos: glam::Vec2, size: glam::Vec2) -> Option<Self> {
        if !pos.is_finite() || !size.is_finite() || size.x <= 0.0 || size.y <= 0.0 {
            return None;
        }

        let max = pos + size;
        if !max.is_finite() || max.x <= pos.x || max.y <= pos.y {
            return None;
        }

        Some(Self { min: pos, max })
    }

    pub fn from_pos_size(pos: glam::Vec2, size: glam::Vec2) -> Self {
        Self::try_from_pos_size(pos, size)
            .expect("AABB geometry must be finite, positive, and non-collapsing")
    }

    pub fn try_from_pos_size(pos: glam::Vec2, size: glam::Vec2) -> anyhow::Result<Self> {
        Self::new(pos, size).ok_or_else(|| {
            anyhow::anyhow!(
                "AABB geometry must be finite, positive, and non-collapsing, got pos={pos:?}, size={size:?}"
            )
        })
    }

    pub fn min(self) -> glam::Vec2 {
        self.min
    }

    pub fn max(self) -> glam::Vec2 {
        self.max
    }

    pub fn size(self) -> glam::Vec2 {
        self.max - self.min
    }

    pub fn overlaps(self, other: Self) -> bool {
        self.min.x < other.max.x
            && self.max.x > other.min.x
            && self.min.y < other.max.y
            && self.max.y > other.min.y
    }
}

pub fn validate_spawn(entity: &Entity, solids: &[Aabb]) -> anyhow::Result<()> {
    let entity_aabb = entity.aabb();
    if let Some(solid) = solids
        .iter()
        .copied()
        .find(|solid| entity_aabb.overlaps(*solid))
    {
        anyhow::bail!(
            "entity spawn {:?}..{:?} overlaps solid {:?}..{:?}",
            entity_aabb.min(),
            entity_aabb.max(),
            solid.min(),
            solid.max()
        );
    }

    Ok(())
}

/// Integrates `entity` by one discrete step against `solids` (see the module
/// docs for the tunneling caveat). The X and Y axes are resolved independently
/// so an entity slides along walls instead of sticking on contact.
pub fn move_with_collision(entity: &mut Entity, solids: &[Aabb], dt: f32) {
    entity.begin_step();

    if !dt.is_finite() || dt <= 0.0 {
        return;
    }

    let mut pos = entity.position();
    let mut vel = entity.velocity();

    pos.x += vel.x * dt;
    entity.set_position(pos);
    for solid in solids {
        let aabb = entity.aabb();
        if aabb.overlaps(*solid) {
            pos = entity.position();
            if vel.x > 0.0 {
                pos.x = solid.min().x - entity.size().x;
            } else if vel.x < 0.0 {
                pos.x = solid.max().x;
            }
            vel.x = 0.0;
            entity.set_position(pos);
            entity.set_velocity(vel);
        }
    }

    pos = entity.position();
    vel = entity.velocity();
    pos.y += vel.y * dt;
    entity.set_position(pos);
    for solid in solids {
        let aabb = entity.aabb();
        if aabb.overlaps(*solid) {
            pos = entity.position();
            if vel.y > 0.0 {
                pos.y = solid.min().y - entity.size().y;
            } else if vel.y < 0.0 {
                pos.y = solid.max().y;
            }
            vel.y = 0.0;
            entity.set_position(pos);
            entity.set_velocity(vel);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Aabb, move_with_collision, validate_spawn};
    use crate::game::world::Entity;

    fn entity(pos: glam::Vec2, vel: glam::Vec2, size: glam::Vec2) -> Entity {
        let mut entity = Entity::new(pos, size, crate::renderer::TEST_TEXTURE_ID);
        entity.set_velocity(vel);
        entity
    }

    #[test]
    fn aabb_new_rejects_invalid_geometry() {
        assert!(Aabb::new(glam::vec2(f32::NAN, 0.0), glam::Vec2::ONE).is_none());
        assert!(Aabb::new(glam::Vec2::ZERO, glam::vec2(0.0, 1.0)).is_none());
        assert!(Aabb::new(glam::Vec2::ZERO, glam::vec2(1.0, -1.0)).is_none());
        assert!(Aabb::new(glam::Vec2::ZERO, glam::Vec2::ONE).is_some());
    }

    #[test]
    fn aabb_new_rejects_overflowing_max_corner() {
        assert!(Aabb::new(glam::Vec2::splat(f32::MAX), glam::Vec2::splat(f32::MAX)).is_none());
    }

    #[test]
    fn aabb_new_rejects_collapsed_max_corner() {
        assert!(Aabb::new(glam::Vec2::splat(f32::MAX), glam::Vec2::ONE).is_none());
    }

    #[test]
    #[should_panic(expected = "AABB geometry must be finite, positive, and non-collapsing")]
    fn from_pos_size_panics_on_invalid_geometry() {
        Aabb::from_pos_size(glam::Vec2::ZERO, glam::vec2(0.0, 1.0));
    }

    #[test]
    fn try_from_pos_size_reports_invalid_geometry() {
        let err = Aabb::try_from_pos_size(glam::Vec2::ZERO, glam::vec2(0.0, 1.0))
            .expect_err("zero width must be rejected");

        assert!(
            err.to_string()
                .contains("AABB geometry must be finite, positive, and non-collapsing"),
            "{err}"
        );
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
            e.set_velocity(glam::vec2(100.0, 0.0));
            move_with_collision(&mut e, &[wall], 0.05);
        }

        // Snapped flush against the wall's left edge: solid.min.x - size.x.
        assert!(
            (e.position().x - 40.0).abs() < 1e-3,
            "got {}",
            e.position().x
        );
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

        assert!(
            (e.position().x - 40.0).abs() < 1e-3,
            "x blocked: {}",
            e.position().x
        );
        assert!(
            e.position().y > 0.0,
            "y should slide freely: {}",
            e.position().y
        );
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

        assert!(
            e.position().x > 102.0,
            "expected tunneling, got {}",
            e.position().x
        );
    }

    #[test]
    fn validate_spawn_rejects_embedded_start() {
        let wall = Aabb::from_pos_size(glam::vec2(0.0, 0.0), glam::vec2(50.0, 50.0));
        let player = entity(
            glam::vec2(10.0, 10.0),
            glam::Vec2::ZERO,
            glam::vec2(10.0, 10.0),
        );

        assert!(validate_spawn(&player, &[wall]).is_err());
    }

    #[test]
    fn zero_velocity_overlap_remains_documented_current_behavior() {
        let wall = Aabb::from_pos_size(glam::vec2(0.0, 0.0), glam::vec2(50.0, 50.0));
        let mut player = entity(
            glam::vec2(10.0, 10.0),
            glam::Vec2::ZERO,
            glam::vec2(10.0, 10.0),
        );

        move_with_collision(&mut player, &[wall], 0.016);

        assert!(player.aabb().overlaps(wall));
    }

    #[test]
    fn adjacent_solids_do_not_block_player_landing_between_them() {
        let left = Aabb::from_pos_size(glam::vec2(0.0, 0.0), glam::vec2(20.0, 20.0));
        let right = Aabb::from_pos_size(glam::vec2(30.0, 0.0), glam::vec2(20.0, 20.0));
        let mut player = entity(
            glam::vec2(20.0, 30.0),
            glam::vec2(0.0, -100.0),
            glam::vec2(10.0, 10.0),
        );

        move_with_collision(&mut player, &[left, right], 0.1);

        assert_eq!(player.position().x, 20.0);
        assert!(player.position().y >= 20.0);
    }

    #[test]
    fn move_with_collision_ignores_non_positive_dt_after_begin_step() {
        let mut e = entity(
            glam::vec2(1.0, 2.0),
            glam::vec2(100.0, 0.0),
            glam::vec2(10.0, 10.0),
        );

        move_with_collision(&mut e, &[], -1.0);

        assert_eq!(e.position(), glam::vec2(1.0, 2.0));
        assert_eq!(e.previous_position(), e.position());
    }

    #[test]
    fn move_with_collision_ignores_non_finite_dt() {
        let mut e = entity(
            glam::vec2(1.0, 2.0),
            glam::vec2(100.0, 0.0),
            glam::vec2(10.0, 10.0),
        );

        move_with_collision(&mut e, &[], f32::NAN);

        assert_eq!(e.position(), glam::vec2(1.0, 2.0));
    }
}

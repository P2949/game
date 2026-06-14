//! Axis-aligned bounding-box (AABB) collision.
//!
//! Two movement integrators are provided:
//!
//! * [`move_with_swept_collision`] — the integrator gameplay uses. It finds the
//!   earliest time-of-impact against every solid along the movement vector, moves
//!   up to it, removes the blocked component, and slides with the remainder. It
//!   does *not* tunnel through thin solids even at high speed, which is why it is
//!   the default before any fast movement (projectiles, dashes, knockback) lands.
//! * [`move_with_collision`] — the older *discrete* integrator. It resolves the X
//!   and Y axes separately and snaps the entity out of any solid it ends a step
//!   overlapping. It is simple and stable for slow motion but only checks the
//!   final position each step, so a fast entity can tunnel straight through a thin
//!   solid (documented intentionally by `fast_entity_tunnels_through_thin_solid`).
//!   Retained for comparison and regression tests.
//!
//! [`depenetrate`] resolves a pre-existing overlap (e.g. a spawn or teleport that
//! lands inside a solid) by pushing out along the shallowest axis.

use crate::game::world::Entity;

/// Iteration cap for [`depenetrate`], bounding work when an entity is wedged
/// between solids it cannot fully escape.
const MAX_DEPENETRATION_STEPS: usize = 8;
/// Small separation kept after a swept impact so the next sweep starts just clear
/// of the surface instead of exactly touching it.
const COLLISION_EPSILON: f32 = 0.001;
/// Cap on slide iterations per [`move_with_swept_collision`] step.
const MAX_COLLISION_ITERATIONS: usize = 4;

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
///
/// Retained for comparison and regression tests; gameplay uses
/// [`move_with_swept_collision`].
#[allow(dead_code)]
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

/// The minimum translation that pushes `a` out of `b` along the single shallowest
/// axis, or `None` when they do not overlap.
fn minimum_translation(a: Aabb, b: Aabb) -> Option<glam::Vec2> {
    if !a.overlaps(b) {
        return None;
    }

    // Per-axis resolution distances; pick the smaller-magnitude direction on each
    // axis, then push along whichever axis penetrates least.
    let push_left = b.min().x - a.max().x; // move a left (negative)
    let push_right = b.max().x - a.min().x; // move a right (positive)
    let dx = if push_right < -push_left {
        push_right
    } else {
        push_left
    };

    let push_down = b.min().y - a.max().y; // move a down (negative)
    let push_up = b.max().y - a.min().y; // move a up (positive)
    let dy = if push_up < -push_down {
        push_up
    } else {
        push_down
    };

    if dx.abs() <= dy.abs() {
        Some(glam::vec2(dx, 0.0))
    } else {
        Some(glam::vec2(0.0, dy))
    }
}

/// Computes a best-effort translation that attempts to move `aabb` clear of the
/// solids it overlaps, resolving the deepest overlap one shallowest-axis push at
/// a time. Returns `None` when `aabb` overlaps nothing (no correction needed).
/// Capped at [`MAX_DEPENETRATION_STEPS`]: an entity wedged between solids it
/// cannot fully escape still terminates, returning a push that may leave some
/// residual overlap rather than looping forever.
pub fn depenetrate(aabb: Aabb, solids: &[Aabb]) -> Option<glam::Vec2> {
    let mut total = glam::Vec2::ZERO;
    let mut current = aabb;

    for _ in 0..MAX_DEPENETRATION_STEPS {
        // Resolve the deepest-overlapping solid first; that ordering converges
        // fastest when several solids overlap at once.
        let mut deepest: Option<(f32, glam::Vec2)> = None;
        for solid in solids {
            if let Some(push) = minimum_translation(current, *solid) {
                let depth = push.length();
                if deepest.is_none_or(|(best, _)| depth > best) {
                    deepest = Some((depth, push));
                }
            }
        }

        let Some((_, push)) = deepest else {
            break;
        };

        total += push;
        current = Aabb::new(current.min() + push, current.size())?;
    }

    (total != glam::Vec2::ZERO).then_some(total)
}

/// A swept-collision result: the fraction `time` in `[0, 1]` of the movement
/// vector at which contact occurs, the surface `normal` to slide along, and the
/// index of the solid that was hit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SweepHit {
    pub time: f32,
    pub normal: glam::Vec2,
    pub solid_index: usize,
}

/// Sweeps `moving` along `delta` against the single static `solid`, returning the
/// earliest contact (if any) as a fraction of `delta`. `solid_index` is always 0
/// here; [`move_with_swept_collision`] fills in the real index.
pub fn swept_aabb(moving: Aabb, delta: glam::Vec2, solid: Aabb) -> Option<SweepHit> {
    // A zero-movement axis imposes no time constraint, but it must still require
    // the boxes to already overlap on that axis — otherwise pure motion on the
    // other axis would falsely "collide" with a box it is not aligned with.
    if delta.x == 0.0 && (moving.max().x <= solid.min().x || moving.min().x >= solid.max().x) {
        return None;
    }
    if delta.y == 0.0 && (moving.max().y <= solid.min().y || moving.min().y >= solid.max().y) {
        return None;
    }

    // Distance to begin/finish overlapping on each axis, signed by movement dir.
    let (inv_entry_x, inv_exit_x) = if delta.x > 0.0 {
        (
            solid.min().x - moving.max().x,
            solid.max().x - moving.min().x,
        )
    } else {
        (
            solid.max().x - moving.min().x,
            solid.min().x - moving.max().x,
        )
    };
    let (inv_entry_y, inv_exit_y) = if delta.y > 0.0 {
        (
            solid.min().y - moving.max().y,
            solid.max().y - moving.min().y,
        )
    } else {
        (
            solid.max().y - moving.min().y,
            solid.min().y - moving.max().y,
        )
    };

    let (entry_x, exit_x) = if delta.x == 0.0 {
        (f32::NEG_INFINITY, f32::INFINITY)
    } else {
        (inv_entry_x / delta.x, inv_exit_x / delta.x)
    };
    let (entry_y, exit_y) = if delta.y == 0.0 {
        (f32::NEG_INFINITY, f32::INFINITY)
    } else {
        (inv_entry_y / delta.y, inv_exit_y / delta.y)
    };

    let entry_time = entry_x.max(entry_y);
    let exit_time = exit_x.min(exit_y);

    // No hit: the overlap windows don't intersect, the contact is before the
    // start (already-overlapping case, left to depenetration), or it is past the
    // end of this movement.
    if entry_time > exit_time || !(0.0..=1.0).contains(&entry_time) {
        return None;
    }

    let normal = if entry_x > entry_y {
        if delta.x > 0.0 {
            glam::vec2(-1.0, 0.0)
        } else {
            glam::vec2(1.0, 0.0)
        }
    } else if delta.y > 0.0 {
        glam::vec2(0.0, -1.0)
    } else {
        glam::vec2(0.0, 1.0)
    };

    Some(SweepHit {
        time: entry_time,
        normal,
        solid_index: 0,
    })
}

/// Integrates `entity` by one step against `solids` using swept collision: it
/// repeatedly advances to the earliest time-of-impact, stops the velocity
/// component into the surface, and slides along it with the remaining movement.
/// Unlike [`move_with_collision`] this cannot tunnel through thin solids at high
/// speed. Capped at [`MAX_COLLISION_ITERATIONS`] slides per step.
///
/// This resolves movement *into* solids, not an already-embedded start: a caller
/// whose entity may begin a step overlapping a solid (a fresh spawn, teleport, or
/// loaded position) should run [`depenetrate`] first. Gameplay does this at spawn
/// in `new_world`.
pub fn move_with_swept_collision(entity: &mut Entity, solids: &[Aabb], dt: f32) {
    entity.begin_step();

    if !dt.is_finite() || dt <= 0.0 {
        return;
    }

    let mut remaining = entity.velocity() * dt;
    let mut velocity = entity.velocity();

    for _ in 0..MAX_COLLISION_ITERATIONS {
        if remaining == glam::Vec2::ZERO {
            break;
        }

        let aabb = entity.aabb();
        let mut earliest: Option<SweepHit> = None;
        for (index, solid) in solids.iter().enumerate() {
            if let Some(mut hit) = swept_aabb(aabb, remaining, *solid) {
                hit.solid_index = index;
                if earliest.is_none_or(|current| hit.time < current.time) {
                    earliest = Some(hit);
                }
            }
        }

        let Some(hit) = earliest else {
            // Nothing in the way: consume the whole remaining movement.
            entity.set_position(entity.position() + remaining);
            break;
        };

        // Advance up to the impact, keeping a small epsilon of separation.
        let travel = remaining * hit.time + hit.normal * COLLISION_EPSILON;
        entity.set_position(entity.position() + travel);

        // Slide: drop the component of the leftover movement and of the velocity
        // that points into the surface.
        let leftover = remaining * (1.0 - hit.time);
        remaining = leftover - hit.normal * leftover.dot(hit.normal);
        velocity -= hit.normal * velocity.dot(hit.normal);
        entity.set_velocity(velocity);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Aabb, depenetrate, move_with_collision, move_with_swept_collision, swept_aabb,
        validate_spawn,
    };
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

    #[test]
    fn depenetration_returns_none_when_not_overlapping() {
        let wall = Aabb::from_pos_size(glam::vec2(100.0, 100.0), glam::vec2(50.0, 50.0));
        let aabb = Aabb::from_pos_size(glam::vec2(0.0, 0.0), glam::vec2(10.0, 10.0));

        assert_eq!(depenetrate(aabb, &[wall]), None);
    }

    #[test]
    fn depenetrates_entity_from_single_wall() {
        let wall = Aabb::from_pos_size(glam::vec2(0.0, 0.0), glam::vec2(50.0, 50.0));
        // Entity overlaps the wall's bottom-left, shallowest escape is downward.
        let aabb = Aabb::from_pos_size(glam::vec2(10.0, 44.0), glam::vec2(10.0, 10.0));

        let push = depenetrate(aabb, &[wall]).expect("overlap must resolve");
        let resolved = Aabb::from_pos_size(aabb.min() + push, aabb.size());
        assert!(
            !resolved.overlaps(wall),
            "still overlapping after push: {push:?}"
        );
    }

    #[test]
    fn depenetrates_using_smallest_axis() {
        let wall = Aabb::from_pos_size(glam::vec2(0.0, 0.0), glam::vec2(100.0, 100.0));
        // Entity pokes 4px into the wall from the right edge; the shallowest exit
        // is a +x push of 4, not a vertical one.
        let aabb = Aabb::from_pos_size(glam::vec2(96.0, 40.0), glam::vec2(10.0, 10.0));

        let push = depenetrate(aabb, &[wall]).expect("overlap must resolve");
        assert_eq!(push, glam::vec2(4.0, 0.0));
    }

    #[test]
    fn depenetration_stops_after_iteration_cap() {
        // An entity wider than the gap between two walls can never fully escape;
        // depenetration must still terminate (return) rather than loop forever.
        let left = Aabb::from_pos_size(glam::vec2(0.0, 0.0), glam::vec2(20.0, 100.0));
        let right = Aabb::from_pos_size(glam::vec2(25.0, 0.0), glam::vec2(20.0, 100.0));
        let aabb = Aabb::from_pos_size(glam::vec2(15.0, 40.0), glam::vec2(15.0, 10.0));

        // The assertion that matters is that this call returns at all.
        let result = depenetrate(aabb, &[left, right]);
        assert!(result.is_some());
    }

    #[test]
    fn swept_collision_handles_zero_velocity() {
        let wall = Aabb::from_pos_size(glam::vec2(50.0, -50.0), glam::vec2(10.0, 100.0));
        let mut e = entity(
            glam::vec2(0.0, 0.0),
            glam::Vec2::ZERO,
            glam::vec2(10.0, 10.0),
        );

        move_with_swept_collision(&mut e, &[wall], 0.016);

        assert_eq!(e.position(), glam::vec2(0.0, 0.0));
    }

    #[test]
    fn swept_collision_ignores_non_colliding_solid() {
        // A solid far off the movement line must not affect the move.
        let off_path = Aabb::from_pos_size(glam::vec2(0.0, 500.0), glam::vec2(10.0, 10.0));
        let mut e = entity(
            glam::vec2(0.0, 0.0),
            glam::vec2(100.0, 0.0),
            glam::vec2(10.0, 10.0),
        );

        move_with_swept_collision(&mut e, &[off_path], 0.1);

        assert!(
            (e.position().x - 10.0).abs() < 1e-3,
            "got {}",
            e.position().x
        );
        assert_eq!(e.position().y, 0.0);
    }

    #[test]
    fn fast_entity_hits_thin_wall() {
        // The discrete integrator tunnels through this (see
        // `fast_entity_tunnels_through_thin_solid`); swept must stop short of it.
        let wall = Aabb::from_pos_size(glam::vec2(100.0, -50.0), glam::vec2(2.0, 100.0));
        let mut e = entity(
            glam::vec2(0.0, 0.0),
            glam::vec2(20_000.0, 0.0),
            glam::vec2(10.0, 10.0),
        );

        move_with_swept_collision(&mut e, &[wall], 0.016);

        // Stopped flush against the wall's left face (within epsilon), never past.
        assert!(
            e.position().x <= 90.0 + 1e-2,
            "expected to stop at the wall, got {}",
            e.position().x
        );
        assert!(!e.aabb().overlaps(wall), "must not end inside the wall");
    }

    #[test]
    fn fast_entity_slides_along_wall() {
        // Moving diagonally into a vertical wall should stop X but keep Y motion.
        let wall = Aabb::from_pos_size(glam::vec2(50.0, -1000.0), glam::vec2(10.0, 2000.0));
        let mut e = entity(
            glam::vec2(0.0, 0.0),
            glam::vec2(20_000.0, 1_000.0),
            glam::vec2(10.0, 10.0),
        );

        move_with_swept_collision(&mut e, &[wall], 0.016);

        assert!(
            e.position().x <= 40.0 + 1e-2,
            "x should be blocked by the wall: {}",
            e.position().x
        );
        assert!(e.position().y > 0.0, "y should slide: {}", e.position().y);
        assert!(!e.aabb().overlaps(wall));
    }

    #[test]
    fn fast_entity_stops_at_corner() {
        // Boxed in on the right and below; a fast down-right move must end inside
        // neither solid.
        let right = Aabb::from_pos_size(glam::vec2(50.0, -1000.0), glam::vec2(10.0, 2000.0));
        let below = Aabb::from_pos_size(glam::vec2(-1000.0, 50.0), glam::vec2(2000.0, 10.0));
        let mut e = entity(
            glam::vec2(0.0, 0.0),
            glam::vec2(20_000.0, 20_000.0),
            glam::vec2(10.0, 10.0),
        );

        move_with_swept_collision(&mut e, &[right, below], 0.016);

        assert!(!e.aabb().overlaps(right), "ended inside right wall");
        assert!(!e.aabb().overlaps(below), "ended inside floor");
        assert!(e.position().x <= 40.0 + 1e-2);
        assert!(e.position().y <= 40.0 + 1e-2);
    }

    #[test]
    fn swept_slow_entity_is_stopped_at_a_solid() {
        // Parity with the discrete integrator for slow motion.
        let wall = Aabb::from_pos_size(glam::vec2(50.0, -50.0), glam::vec2(10.0, 100.0));
        let mut e = entity(
            glam::vec2(0.0, 0.0),
            glam::Vec2::ZERO,
            glam::vec2(10.0, 10.0),
        );

        for _ in 0..100 {
            e.set_velocity(glam::vec2(100.0, 0.0));
            move_with_swept_collision(&mut e, &[wall], 0.05);
        }

        assert!(
            (e.position().x - 40.0).abs() < 0.05,
            "got {}",
            e.position().x
        );
        assert!(!e.aabb().overlaps(wall));
    }

    #[test]
    fn swept_collision_iteration_cap_prevents_infinite_loop() {
        // Surrounded on all four sides; the slide loop must terminate at the cap.
        let solids = [
            Aabb::from_pos_size(glam::vec2(-100.0, -10.0), glam::vec2(100.0, 200.0)),
            Aabb::from_pos_size(glam::vec2(60.0, -10.0), glam::vec2(100.0, 200.0)),
            Aabb::from_pos_size(glam::vec2(-10.0, -100.0), glam::vec2(200.0, 100.0)),
            Aabb::from_pos_size(glam::vec2(-10.0, 60.0), glam::vec2(200.0, 100.0)),
        ];
        let mut e = entity(
            glam::vec2(20.0, 20.0),
            glam::vec2(5_000.0, 5_000.0),
            glam::vec2(10.0, 10.0),
        );

        // Must return (not hang). Position stays finite and within the box.
        move_with_swept_collision(&mut e, &solids, 0.1);
        assert!(e.position().is_finite());
    }

    #[test]
    fn swept_aabb_reports_axis_normal() {
        // Moving +x into a wall yields a -x normal at the contact fraction.
        let wall = Aabb::from_pos_size(glam::vec2(50.0, -50.0), glam::vec2(10.0, 100.0));
        let moving = Aabb::from_pos_size(glam::vec2(0.0, 0.0), glam::vec2(10.0, 10.0));

        let hit = swept_aabb(moving, glam::vec2(100.0, 0.0), wall).expect("should hit");
        assert_eq!(hit.normal, glam::vec2(-1.0, 0.0));
        // Contact when the right edge (x=10) reaches the wall's left edge (x=50):
        // 40 of 100 units traveled.
        assert!((hit.time - 0.4).abs() < 1e-4, "got {}", hit.time);
    }
}

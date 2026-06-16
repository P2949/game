//! Small reusable gameplay helpers (Phase 10).
//!
//! These reduce duplication between content crates without forcing a game design:
//! content keeps its own state/components and opts into the helpers it wants.

use game_core::input::Axis2dId;
use game_core::world::Component;

use crate::context::GameCtx;

/// A pause/death-style state. Implement it on a content `GameState` to get the
/// derived [`SimulationState::active`] guard without re-deriving it everywhere.
pub trait SimulationState {
    fn paused(&self) -> bool;
    fn dead(&self) -> bool;
    /// True when the simulation should advance (not paused and not dead).
    fn active(&self) -> bool {
        !self.paused() && !self.dead()
    }
}

/// Content components implement this to tell [`GameCtx::drive_input`] which
/// logical 2D axis drives an entity.
pub trait InputDriven {
    fn movement_axis(&self) -> Axis2dId;
}

/// Content speed components implement this to expose movement speed without
/// forcing a shared component type.
pub trait MovementSpeed {
    fn units_per_second(&self) -> f32;
}

/// Centers the camera on the first live entity carrying component `T` (typically a
/// player marker). No-op if none exists.
pub fn camera_follow_first<T: Component>(game: &mut GameCtx<'_, '_>) {
    game.camera_follow_first::<T>();
}

/// Zeroes every entity's velocity (used when pausing or on death).
pub fn stop_all_velocity(game: &mut GameCtx<'_, '_>) {
    game.stop_all_velocity();
}

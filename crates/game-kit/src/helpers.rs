//! Small reusable gameplay helpers (Phase 10).
//!
//! These reduce duplication between content crates without forcing a game design:
//! content keeps its own state/components and opts into the helpers it wants.

use game_core::world::{Component, Transform, Velocity, World};
use glam::Vec2;

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

/// Centers the camera on the first live entity carrying component `T` (typically a
/// player marker). No-op if none exists.
pub fn camera_follow_first<T: Component>(game: &mut GameCtx<'_, '_>) {
    let position = game
        .world()
        .ids_with::<T>()
        .into_iter()
        .find_map(|id| game.world().get::<Transform>(id).map(|t| t.pos));
    if let Some(position) = position {
        game.camera_mut().set_center(position);
    }
}

/// Zeroes every entity's velocity (used when pausing or on death).
pub fn stop_all_velocity(world: &mut World) {
    for id in world.ids_with::<Velocity>() {
        if let Some(velocity) = world.get_mut::<Velocity>(id) {
            velocity.0 = Vec2::ZERO;
        }
    }
}

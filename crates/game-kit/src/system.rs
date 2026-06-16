//! System adapters (Phase 9).
//!
//! These traits let content register plain `fn(&mut GameCtx, f32)` /
//! `fn(&mut StartupGameCtx) -> Result<()>` systems through [`crate::GameApp`],
//! which wraps the engine's raw `Ctx`/`StartCtx` for them. Content never imports
//! `Schedule` or the engine `System` types.

use crate::context::{GameCtx, StartupGameCtx};

/// A fixed/update/render/ui system: `fn(&mut GameCtx, f32)` or any matching
/// closure. Frame systems are intentionally infallible; runtime operations that
/// should not fail after validation expose infallible/logging helpers instead of
/// making every gameplay system return `Result`.
///
/// The trait erases the context lifetimes so a bare function item works.
pub trait GameSystem: 'static {
    fn run(&mut self, game: &mut GameCtx<'_, '_>, dt: f32);
}

impl<F> GameSystem for F
where
    F: FnMut(&mut GameCtx<'_, '_>, f32) + 'static,
{
    fn run(&mut self, game: &mut GameCtx<'_, '_>, dt: f32) {
        self(game, dt)
    }
}

/// A startup system: `fn(&mut StartupGameCtx) -> anyhow::Result<()>` or any
/// matching closure. Startup is fallible because content initialization,
/// validation-time spawning, and resource setup can fail before the game loop
/// starts.
pub trait StartupSystem: 'static {
    fn run(&mut self, game: &mut StartupGameCtx<'_, '_>) -> anyhow::Result<()>;
}

impl<F> StartupSystem for F
where
    F: FnMut(&mut StartupGameCtx<'_, '_>) -> anyhow::Result<()> + 'static,
{
    fn run(&mut self, game: &mut StartupGameCtx<'_, '_>) -> anyhow::Result<()> {
        self(game)
    }
}

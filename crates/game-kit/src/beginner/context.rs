//! Beginner-friendly context names.

pub use crate::context::{Commands, GameCtx, StartupGameCtx};

pub type Game<'a, 'w> = GameCtx<'a, 'w>;
pub type StartupGame<'a, 'w> = StartupGameCtx<'a, 'w>;
pub type Seconds = f32;

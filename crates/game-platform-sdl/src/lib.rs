pub mod input;
pub mod resize;
pub mod window;

// TEMP: compatibility module for Phase 2's mechanical move from `src/platform`.
pub mod platform {
    pub use crate::{input, resize, window};
}

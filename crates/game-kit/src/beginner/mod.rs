//! Beginner-facing authoring surface.
//!
//! The modules here are intentionally thin at first. They give beginner APIs a
//! named home while preserving the existing `game_kit::prelude::*` transition.

pub mod actors;
pub mod animation;
pub mod app;
pub mod audio;
pub mod camera;
pub mod collections;
pub mod combat;
pub mod context;
pub mod debug;
pub mod defaults;
pub mod events;
pub mod prefabs;
pub mod prelude;
pub mod rules;
pub mod scene;
pub mod spawn;
pub mod state;
pub mod testing;
pub mod time;
pub mod tuning;
pub mod ui;

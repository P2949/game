//! `game-kit` is the friendly content-authoring facade.
//!
//! Game content (the `arena-content`/`testbed-content` crates) imports
//! `game_kit::prelude::*` and expresses **assets, controls, prefabs, maps, and
//! systems** through [`GameApp`] and [`GameCtx`]. It never operates the engine's
//! `GameBuilder`, `Schedule`, `PrefabRegistry`, `MapRegistry`, validators, raw
//! `Ctx`, or `CommandQueue` directly, and it never sees the SDL/Vulkan/audio
//! backends.
//!
//! See `docs/content-authoring.md` for the author-facing guide.

pub mod app;
pub mod assets;
pub mod bundle;
pub mod context;
pub mod harness;
pub mod helpers;
pub mod input;
pub mod map;
pub mod prefab;
pub mod system;

pub use app::{GameApp, GamePlugin, Plugin, plugin};
pub use assets::AssetAuthor;
pub use bundle::{Bundle, vec2s};
pub use context::{Commands, GameCtx, StartupGameCtx};
pub use harness::GameTestHarness;
pub use helpers::{SimulationState, camera_follow_first, stop_all_velocity};
pub use input::{ActionAuthor, Axis2dAuthor, InputAuthor};
pub use map::{MapAuthor, TileTheme};
pub use prefab::PrefabAuthor;
pub use system::{GameSystem, StartupSystem};

/// The single import content crates need: `use game_kit::prelude::*;`.
pub mod prelude {
    pub use anyhow::{Context, Result};
    pub use glam::{Vec2, Vec4, vec2, vec4};

    // Engine-neutral gameplay types and behaviors, surfaced through one prelude so
    // content never reaches into `game_core`/`game_map`/`game_ai`/... directly.
    pub use game_ai::{AiController, ChaseTarget, PathFollow, Patrol, chase_system, patrol_system};
    pub use game_combat::{Faction, FactionId, Health, MeleeAttack, apply_damage};
    pub use game_core::backend::{FontHandle, SoundHandle, TextureHandle};
    pub use game_core::builder::PrefabId;
    pub use game_core::camera::Camera2D;
    pub use game_core::input::{ActionId, Axis2dId, Input, Key};
    pub use game_core::nav::NavGrid;
    pub use game_core::tilemap::TileMap;
    pub use game_core::world::{Component, Entity, EntityId, Sprite, Transform, Velocity, World};
    pub use game_map::{MapCell, cell};
    pub use game_physics::{Collider, movement_system};

    // The authoring facade itself.
    pub use crate::app::{GameApp, GamePlugin, plugin};
    pub use crate::assets::AssetAuthor;
    pub use crate::bundle::{Bundle, vec2s};
    pub use crate::context::{Commands, GameCtx, StartupGameCtx};
    pub use crate::harness::GameTestHarness;
    pub use crate::helpers::{SimulationState, camera_follow_first, stop_all_velocity};
    pub use crate::input::{ActionAuthor, Axis2dAuthor, InputAuthor};
    pub use crate::map::{MapAuthor, TileTheme};
    pub use crate::prefab::PrefabAuthor;
}

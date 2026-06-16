//! `game-kit` is the friendly content-authoring facade.
//!
//! Game content (the `arena-content`/`testbed-content` crates) imports
//! `game_kit::prelude::*` and expresses **assets, controls, prefabs, maps, and
//! systems** through [`GameApp`] and [`GameCtx`]. It never operates the engine's
//! `GameBuilder`, `Schedule`, `PrefabRegistry`, `MapRegistry`, validators, raw
//! `Ctx`, or `CommandQueue` directly, and it never sees the SDL/Vulkan/audio
//! backends.
//!
//! A content crate is usually a small plugin that delegates registration to
//! assets/input/prefab/map/system modules:
//!
//! ```ignore
//! use game_kit::prelude::*;
//!
//! pub struct DemoPlugin;
//!
//! impl GamePlugin for DemoPlugin {
//!     fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
//!         let assets = game.assets(assets::register)?;
//!         let input = game.input(input::register)?;
//!         prefabs::register(game, &assets, &input)?;
//!         level::register(game, &assets)?;
//!         systems::register(game, &assets, &input);
//!         Ok(())
//!     }
//! }
//! ```
//!
//! Prefabs spawn tuple bundles and state their validation requirements through
//! the facade:
//!
//! ```ignore
//! use game_kit::prelude::*;
//!
//! game.prefab("demo/player", |prefab| {
//!     prefab
//!         .spawn(move |at| {
//!             (
//!                 Transform::at(at),
//!                 Velocity::default(),
//!                 Sprite::new(assets.player, vec2s(20.0)),
//!                 Collider::box_of(vec2s(20.0)),
//!             )
//!         })?
//!         .require::<Transform>()
//!         .require::<Sprite>();
//!     Ok(())
//! })?;
//! ```
//!
//! Systems receive [`GameCtx`] and use its helpers instead of raw world plumbing:
//!
//! ```ignore
//! use game_kit::prelude::*;
//!
//! fn player_control(game: &mut GameCtx<'_, '_>, _dt: f32) {
//!     game.drive_input::<PlayerController, MovementSpeed>();
//! }
//! ```
//!
//! See `docs/content-authoring.md` for the author-facing guide.

pub mod app;
pub mod assets;
pub mod bundle;
pub mod context;
mod harness;
pub mod helpers;
pub mod input;
pub mod map;
pub mod prefab;
pub mod system;

pub use app::{GameApp, GamePlugin, Plugin, plugin};
pub use assets::AssetAuthor;
pub use bundle::{Bundle, vec2s};
pub use context::{Commands, GameCtx, StartupGameCtx};
pub use helpers::{
    InputDriven, MovementSpeed, SimulationState, camera_follow_first, stop_all_velocity,
};
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
    pub use game_ai::{AiController, ChaseTarget, PathFollow, Patrol};
    pub use game_combat::{Faction, FactionId, Health, MeleeAttack};
    pub use game_core::backend::{FontHandle, SoundHandle, TextureHandle};
    pub use game_core::camera::Camera2D;
    pub use game_core::input::{ActionId, Axis2dId, Key};
    pub use game_core::world::{Component, EntityId, Sprite, Transform, Velocity};
    pub use game_map::{MapCell, cell};
    pub use game_physics::Collider;

    // The authoring facade itself.
    pub use crate::app::{GameApp, GamePlugin, plugin};
    pub use crate::assets::AssetAuthor;
    pub use crate::bundle::{Bundle, vec2s};
    pub use crate::context::{Commands, GameCtx, StartupGameCtx};
    pub use crate::helpers::{InputDriven, MovementSpeed, SimulationState};
    pub use crate::input::{ActionAuthor, Axis2dAuthor, InputAuthor};
    pub use crate::map::{MapAuthor, TileTheme};
    pub use crate::prefab::PrefabAuthor;
}

/// Test imports for content tests that need raw ECS/world inspection.
pub mod testing {
    pub use crate::harness::GameTestHarness;

    pub mod prelude {
        pub use anyhow::{Context, Result};
        pub use glam::{Vec2, Vec4, vec2, vec4};

        pub use game_ai::{
            AiController, ChaseTarget, PathFollow, Patrol, chase_system, patrol_system,
        };
        pub use game_combat::{Faction, FactionId, Health, MeleeAttack, apply_damage};
        pub use game_core::backend::{FontHandle, SoundHandle, TextureHandle};
        pub use game_core::builder::PrefabId;
        pub use game_core::camera::Camera2D;
        pub use game_core::input::{ActionId, Axis2dId, Input, Key};
        pub use game_core::nav::NavGrid;
        pub use game_core::tilemap::TileMap;
        pub use game_core::world::{
            Component, Entity, EntityId, Sprite, Transform, Velocity, World,
        };
        pub use game_map::{MapCell, cell};
        pub use game_physics::{Collider, movement_system};

        pub use crate::prelude::*;
        pub use crate::testing::GameTestHarness;
    }
}

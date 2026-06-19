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

pub mod advanced;
pub mod app;
pub mod assets;
pub mod beginner;
pub mod bundle;
pub mod context;
mod harness;
pub mod helpers;
pub mod input;
pub mod map;
pub mod prefab;
pub mod system;

pub use app::{DebugOverlayAuthor, FnGamePlugin, GameApp, GamePlugin, Plugin, plugin, plugin_fn};
pub use assets::{AssetAuthor, AssetBag, AssetBagAuthor};
pub use beginner::actors::{
    CollectSound, Collectible, DespawnOnCollect, DespawnOnHit, Door, DoorAction, DoorTarget, Enemy,
    ExitDoor, Lifetime, Name, Npc, Pickup, Player, PlayerMovement, Projectile, ProjectileDamage,
    ScoreValue, Solid, Spawner, Speed,
};
pub use beginner::animation::{
    Animation, AnimationClip, AnimationSet, SpriteSheet, attack_frames, die_frames, frames,
    idle_frames, walk_frames,
};
pub use beginner::camera::CameraShake;
pub use beginner::collections::{
    CameraOps, EnemyCollection, PickupCollection, PlayerActor, Score, ScoreOps,
};
pub use beginner::combat::MeleeCombatConfig;
pub use beginner::debug::DebugOverlay;
pub use beginner::defaults::TopDownGameAuthor;
pub use beginner::prefabs::{
    DoorPrefabAuthor, EnemyPrefabAuthor, PickupPrefabAuthor, PlayerPrefabAuthor,
    ProjectilePrefabAuthor, SpawnerPrefabAuthor,
};
pub use beginner::rules::RulesAuthor;
pub use beginner::scene::{SceneRegistry, SceneState, SimpleSceneFlowAuthor};
pub use beginner::spawn::SpawnAuthor;
pub use beginner::state::SimpleGameState;
pub use bundle::{Bundle, vec2s};
pub use context::{Commands, GameCtx, StartupGameCtx};
pub use helpers::{
    InputDriven, MovementSpeed, SimulationState, camera_follow_first, stop_all_velocity,
};
pub use input::{ActionAuthor, Axis2dAuthor, InputAuthor, TopDownControls};
pub use map::{MapAuthor, TileTheme};
pub use prefab::PrefabAuthor;
pub use system::{GameSystem, StartupSystem};

/// Compatibility prelude.
///
/// New beginner code should prefer `game_kit::beginner::prelude::*`; advanced
/// code should prefer `game_kit::advanced::prelude::*`.
pub mod prelude {
    pub use anyhow::{Context, Result};
    pub use glam::{Vec2, Vec4, vec2, vec4};

    // Engine-neutral gameplay types and behaviors, surfaced through one prelude so
    // content never reaches into `game_core`/`game_map`/`game_ai`/... directly.
    pub use game_ai::{AiController, ChaseTarget, PathFollow, Patrol};
    pub use game_combat::{Faction, FactionId, Health, MeleeAttack};
    pub use game_core::backend::{FontHandle, SoundHandle, TextureHandle};
    pub use game_core::camera::Camera2D;
    pub use game_core::input::{ActionId, Axis2dId, Key, MouseButton};
    pub use game_core::world::{Component, EntityId, Sprite, Transform, Velocity};
    pub use game_map::{MapCell, cell};
    pub use game_physics::Collider;

    // The authoring facade itself.
    pub use crate::app::{
        DebugOverlayAuthor, FnGamePlugin, GameApp, GamePlugin, plugin, plugin_fn,
    };
    pub use crate::assets::{AssetAuthor, AssetBag, AssetBagAuthor};
    pub use crate::beginner::actors::{
        CollectSound, Collectible, DespawnOnCollect, DespawnOnHit, Door, DoorAction, DoorTarget,
        Enemy, ExitDoor, Lifetime, Name, Npc, Pickup, Player, PlayerMovement, Projectile,
        ProjectileDamage, ScoreValue, Solid, Spawner, Speed,
    };
    pub use crate::beginner::animation::{
        Animation, AnimationClip, AnimationSet, SpriteSheet, attack_frames, die_frames, frames,
        idle_frames, walk_frames,
    };
    pub use crate::beginner::camera::CameraShake;
    pub use crate::beginner::collections::{
        CameraOps, EnemyCollection, PickupCollection, PlayerActor, Score, ScoreOps,
    };
    pub use crate::beginner::combat::MeleeCombatConfig;
    pub use crate::beginner::context::{Game, Seconds, StartupGame};
    pub use crate::beginner::debug::DebugOverlay;
    pub use crate::beginner::defaults::TopDownGameAuthor;
    pub use crate::beginner::prefabs::{
        DoorPrefabAuthor, EnemyPrefabAuthor, PickupPrefabAuthor, PlayerPrefabAuthor,
        ProjectilePrefabAuthor, SpawnerPrefabAuthor,
    };
    pub use crate::beginner::rules::RulesAuthor;
    pub use crate::beginner::scene::{SceneRegistry, SceneState, SimpleSceneFlowAuthor};
    pub use crate::beginner::spawn::SpawnAuthor;
    pub use crate::beginner::state::SimpleGameState;
    pub use crate::bundle::{Bundle, vec2s};
    pub use crate::context::{Commands, GameCtx, StartupGameCtx};
    pub use crate::helpers::{InputDriven, MovementSpeed, SimulationState};
    pub use crate::input::{ActionAuthor, Axis2dAuthor, InputAuthor, TopDownControls};
    pub use crate::map::{MapAuthor, TileTheme};
    pub use crate::prefab::PrefabAuthor;
}

pub mod advanced_prelude {
    pub use crate::advanced::prelude::*;
}

/// Test imports for content tests that need raw ECS/world inspection.
pub mod testing {
    pub use crate::harness::GameTestHarness;

    pub mod prelude {
        pub use crate::advanced::testing::prelude::*;
    }
}

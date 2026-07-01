//! `game-kit` is the content-authoring facade.
//!
//! Beginner content should import:
//!
//! ```ignore
//! use game_kit::beginner::prelude::*;
//! ```
//!
//! Standalone demos should usually import:
//!
//! ```ignore
//! use game_starter::prelude::*;
//! ```
//!
//! The beginner API provides players, enemies, pickups, doors, projectiles,
//! maps, assets, controls, scenes, rules, audio, and UI helpers.
//!
//! ## Stability
//!
//! Beginner APIs are stabilized first. When a beginner method is renamed, keep
//! the old method for one release with a deprecation note, changelog entry, and
//! migration note. Data-driven `assets/game.ron` files are versioned by their
//! `version` field. Advanced APIs are allowed to evolve faster.
//! Engine internals are unstable implementation details.
//!
//! ## First game
//!
//! ```ignore
//! game.asset_bag()
//!     .texture("player", "textures/player.png")?
//!     .texture("slime", "textures/slime.png")?
//!     .texture("floor", "textures/floor.png")?
//!     .texture("wall", "textures/wall.png")?
//!     .sound("hit", "sounds/hit.wav")?
//!     .build();
//!
//! let controls = game.input(|input| input.top_down_controls())?;
//!
//! game.player_prefab("player")
//!     .sprite("player")
//!     .moves_with(controls.movement, 130.0)
//!     .build()?;
//!
//! game.enemy_prefab("slime")
//!     .sprite("slime")
//!     .chases_player()
//!     .build()?;
//!
//! game.map("level_1")
//!     .tiles(["#####", "#P.E#", "#####"])
//!     .simple_theme("floor", "wall")
//!     .legend('P', "player")
//!     .legend('E', "slime")
//!     .start();
//!
//! game.rules()
//!     .top_down_controls(controls)
//!     .enemies_damage_player()
//!     .camera_follows_player()
//!     .show_player_health()
//!     .build();
//! ```
//!
//! ## Advanced authoring
//!
//! For custom ECS-style systems and tuple prefabs, import:
//!
//! ```ignore
//! use game_kit::advanced::prelude::*;
//! ```
//!
//! ```ignore
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
//! See `docs/content-authoring.md` for the author-facing guide.

pub mod advanced;
pub mod app;
pub mod assets;
pub mod beginner;
pub mod bundle;
pub mod context;
pub mod data;
mod diagnostics;
mod harness;
mod helpers;
pub mod input;
pub mod map;
mod paths;
pub mod prefab;
pub mod system;

/// Defines a content crate plugin without exposing trait or app boilerplate.
///
/// Beginner content crates can use this through
/// `game_kit::beginner::prelude::*`:
///
/// ```ignore
/// content_plugin!(MyContent, plugin, |game| {
///     game.asset_bag().texture("player", "textures/player.png")?.build();
/// });
/// ```
#[macro_export]
macro_rules! content_plugin {
    ($plugin_ty:ident, $plugin_fn:ident, |$game:ident| $body:block) => {
        pub struct $plugin_ty;

        pub fn $plugin_fn() -> $crate::app::Plugin<$plugin_ty> {
            $crate::app::plugin($plugin_ty)
        }

        impl $crate::app::GamePlugin for $plugin_ty {
            fn build(&self, $game: &mut $crate::app::GameApp<'_>) -> anyhow::Result<()> {
                $body
                Ok(())
            }
        }
    };
}

/// Deprecated root-export compatibility surface.
///
/// New content should import `game_kit::beginner::prelude::*`,
/// `game_kit::advanced::prelude::*`, or explicit module paths such as
/// `game_kit::app::GameApp`.
#[deprecated(note = "Use game_kit::beginner::prelude::* or game_kit::advanced::prelude::*")]
pub mod compat {
    pub use crate::app::{
        DebugOverlayAuthor, FnGamePlugin, GameApp, GamePlugin, Plugin, plugin, plugin_fn,
    };
    pub use crate::assets::{
        AssetAuthor, AssetBag, AssetBagAuthor, AssetFolderAuthor, IntoSoundRef, IntoTextureRef,
        SoundRef, TextureRef,
    };
    pub use crate::beginner::actors::{
        Area, AreaName, Checkpoint, CheckpointState, CollectSound, Collectible,
        DeathAnimationPolicy, DespawnOnCollect, DespawnOnHit, Door, DoorAction, DoorTarget,
        DropSpawned, DropsPrefab, Enemy, ExitDoor, HealValue, Lifetime, Name, Npc, Pickup, Player,
        PlayerMovement, Projectile, ProjectileDamage, ProjectileImpact, ScoreValue, Solid, Spawner,
        Speed, TriggerArea,
    };
    pub use crate::beginner::animation::{
        Animation, AnimationClip, AnimationSet, AnimationSheet, SpriteSheet, attack_frames,
        die_frames, frames, idle_frames, walk_frames,
    };
    pub use crate::beginner::audio::{AudioBus, AudioOps, MusicPlayback, SoundPlayback};
    pub use crate::beginner::camera::CameraShake;
    pub use crate::beginner::collections::{
        CameraOps, EnemyCollection, FiredShot, PickupCollection, PlayerActor, Score, ScoreOps,
        ShootAuthor, TaggedActors,
    };
    pub use crate::beginner::combat::MeleeCombatConfig;
    pub use crate::beginner::custom_rules::{
        CountdownRuleAuthor, CustomRuleAuthor, TaggedCustomRuleAuthor,
    };
    pub use crate::beginner::debug::DebugOverlay;
    pub use crate::beginner::defaults::TopDownGameAuthor;
    pub use crate::beginner::defaults::{
        AnimationUpdateBehavior, CameraFollowBehavior, CameraShakeBehavior, CameraZoomBehavior,
        CollisionBehavior, DeathStateBehavior, DirectionalAttackBehavior,
        EnemyAnimationByMovementBehavior, EnemyChaseBehavior, EnemyDirectionalAnimationBehavior,
        EnemyPatrolBehavior, MeleeCombatBehavior, MovementBehavior, PauseDeathUiBehavior,
        PlayerAnimationByMovementBehavior, PlayerDirectionalAnimationBehavior,
        PlayerFacingBehavior, SimpleGameStartupBehavior,
    };
    pub use crate::beginner::events::{
        AnimationFinishedEvent, CollectEvent, CollisionEvent, DoorEvent, EnemyDeathEvent,
        EventActor, MapChangedEvent, ProjectileHitEvent,
    };
    pub use crate::beginner::prefabs::{
        AreaPrefabAuthor, DoorPrefabAuthor, EnemyPrefabAuthor, PickupPrefabAuthor,
        PlayerPrefabAuthor, ProjectilePrefabAuthor, SpawnerPrefabAuthor,
    };
    pub use crate::beginner::rules::RulesAuthor;
    pub use crate::beginner::rules::{
        CheckpointActivationBehavior, CheckpointRespawnBehavior, CollectPickupsBehavior,
        DeadEnemiesDespawnBehavior, DeathAnimationBehavior, DeathAnimationDespawnBehavior,
        DoorsChangeMapsBehavior, EnemyDropsBehavior, HighLevelUiBehavior, ProjectileDamageBehavior,
        ProjectileImpactDespawnBehavior, ProjectileLifetimeBehavior, ProjectileMovementBehavior,
        RulesAnimationUpdateBehavior, RulesEnemyAnimationByMovementBehavior,
        RulesEnemyDirectionalAnimationBehavior, RulesPlayerDirectionalAnimationBehavior,
        SpawnerBehavior, WinConditionBehavior,
    };
    pub use crate::beginner::scene::{SceneRegistry, SceneState, SimpleSceneFlowAuthor};
    pub use crate::beginner::spawn::SpawnAuthor;
    pub use crate::beginner::state::SimpleGameState;
    pub use crate::beginner::tuning::TuningFile;
    pub use crate::beginner::ui::{
        UiButton, UiFocus, UiMenu, UiMenuButton, UiOps, UiPanel, UiStatusPanel, UiText,
    };
    pub use crate::bundle::{Bundle, vec2s};
    pub use crate::context::{Commands, GameCtx, StartupGameCtx};
    pub use crate::data::{
        BeginnerAssetsFile, BeginnerControlsFile, BeginnerGameFile, BeginnerMapFile,
        BeginnerPrefabFile, BeginnerRuleFile, BeginnerScriptRuleFile, RuleEffectFile,
        load_beginner_game_file,
    };
    pub use crate::helpers::{
        InputDriven, MovementSpeed, SimulationState, camera_follow_first, stop_all_velocity,
    };
    pub use crate::input::{ActionAuthor, Axis2dAuthor, InputAuthor, TopDownControls};
    pub use crate::map::{MapAuthor, TileTheme};
    pub use crate::prefab::PrefabAuthor;
    pub use crate::prefab::{IntoContentName, IntoMovementAxis};
    pub use crate::system::{GameSystem, StartupSystem};
}

/// Compatibility prelude.
///
/// New beginner code should import `game_kit::beginner::prelude::*`.
/// Advanced code should import `game_kit::advanced::prelude::*`.
/// This broad prelude exists to avoid breaking older examples while the
/// authoring facade stabilizes. Do not use it in new beginner code, docs, or
/// templates.
#[deprecated(note = "Use game_kit::beginner::prelude::* or game_kit::advanced::prelude::*")]
pub mod prelude {
    pub use anyhow::{Context, Result};
    pub use glam::{Vec2, Vec4, vec2, vec4};

    // Engine-neutral gameplay types and behaviors, surfaced through one prelude so
    // content never reaches into `game_core`/`game_map`/`game_ai`/... directly.
    pub use game_ai::{AiController, ChaseTarget, PathFollow, Patrol};
    pub use game_combat::{Faction, FactionId, Health, MeleeAttack};
    pub use game_core::backend::{FontHandle, SoundHandle, TextureHandle};
    pub use game_core::camera::Camera2D;
    pub use game_core::input::{ActionId, Axis2dId, GamepadAxis, GamepadButton, Key, MouseButton};
    pub use game_core::world::{Component, EntityId, Sprite, Transform, Velocity};
    pub use game_map::{MapCell, cell};
    pub use game_physics::Collider;

    // The authoring facade itself.
    pub use crate::app::{
        DebugOverlayAuthor, FnGamePlugin, GameApp, GamePlugin, plugin, plugin_fn,
    };
    pub use crate::assets::{
        AssetAuthor, AssetBag, AssetBagAuthor, AssetFolderAuthor, IntoSoundRef, SoundRef,
        TextureRef,
    };
    pub use crate::beginner::actors::{
        Area, AreaName, Checkpoint, CheckpointState, CollectSound, Collectible,
        DeathAnimationPolicy, DespawnOnCollect, DespawnOnHit, Door, DoorAction, DoorTarget,
        DropSpawned, DropsPrefab, Enemy, ExitDoor, HealValue, Lifetime, Name, Npc, Pickup, Player,
        PlayerMovement, Projectile, ProjectileDamage, ProjectileImpact, ScoreValue, Solid, Spawner,
        Speed, TriggerArea,
    };
    pub use crate::beginner::animation::{
        Animation, AnimationClip, AnimationSet, AnimationSheet, SpriteSheet, attack_frames,
        die_frames, frames, idle_frames, walk_frames,
    };
    pub use crate::beginner::camera::CameraShake;
    pub use crate::beginner::collections::{
        CameraOps, EnemyCollection, FiredShot, PickupCollection, PlayerActor, Score, ScoreOps,
        ShootAuthor, TaggedActors,
    };
    pub use crate::beginner::combat::MeleeCombatConfig;
    pub use crate::beginner::context::{Game, Seconds, StartupGame};
    pub use crate::beginner::debug::DebugOverlay;
    pub use crate::beginner::defaults::TopDownGameAuthor;
    pub use crate::beginner::events::{
        AnimationFinishedEvent, CollectEvent, CollisionEvent, DoorEvent, EnemyDeathEvent,
        EventActor, MapChangedEvent, ProjectileHitEvent,
    };
    pub use crate::beginner::prefabs::{
        AreaPrefabAuthor, DoorPrefabAuthor, EnemyPrefabAuthor, PickupPrefabAuthor,
        PlayerPrefabAuthor, ProjectilePrefabAuthor, SpawnerPrefabAuthor,
    };
    pub use crate::beginner::rules::RulesAuthor;
    pub use crate::beginner::scene::{SceneRegistry, SceneState, SimpleSceneFlowAuthor};
    pub use crate::beginner::spawn::SpawnAuthor;
    pub use crate::beginner::state::SimpleGameState;
    pub use crate::beginner::ui::{
        UiButton, UiFocus, UiMenu, UiMenuButton, UiOps, UiPanel, UiStatusPanel, UiText,
    };
    pub use crate::bundle::{Bundle, vec2s};
    pub use crate::context::{Commands, GameCtx, StartupGameCtx};
    pub use crate::helpers::{InputDriven, MovementSpeed, SimulationState};
    pub use crate::input::{ActionAuthor, Axis2dAuthor, InputAuthor, TopDownControls};
    pub use crate::map::{MapAuthor, TileTheme};
    pub use crate::prefab::{IntoContentName, IntoMovementAxis, PrefabAuthor};
}

#[deprecated(note = "Use game_kit::advanced::prelude::*")]
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

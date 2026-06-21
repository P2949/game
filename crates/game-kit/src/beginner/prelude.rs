//! Beginner import surface.
//!
//! This module intentionally keeps low-level query and scheduling plumbing out
//! of the default beginner import path. Use `game_kit::advanced::prelude::*`
//! when a project needs the lower-level facade.

pub use anyhow::{Context, Result};
pub use glam::{Vec2, Vec4, vec2, vec4};

pub use game_core::backend::{FontHandle, SoundHandle, TextureHandle};
pub use game_core::input::{ActionId, Axis2dId, GamepadAxis, GamepadButton, Key, MouseButton};
pub use game_map::{MapCell, cell};

pub use crate::app::{GameApp, GamePlugin, Plugin, plugin, plugin_fn};
pub use crate::assets::{
    AssetAuthor, AssetBag, AssetBagAuthor, AssetFolderAuthor, IntoTextureRef, SoundRef, TextureRef,
};
pub use crate::beginner::actors::{
    Area, AreaName, Checkpoint, CheckpointState, DeathAnimationPolicy, Door, DropsPrefab, Enemy,
    HealValue, Name, Npc, Pickup, Player, PlayerMovement, Projectile, ProjectileImpact, ScoreValue,
    Solid, Speed, TriggerArea,
};
pub use crate::beginner::animation::{
    Animation, AnimationClip, AnimationSet, AnimationSheet, SpriteSheet, attack_frames, die_frames,
    frames, idle_frames, walk_frames,
};
pub use crate::beginner::audio::{AudioBus, AudioOps, MusicPlayback, SoundPlayback};
pub use crate::beginner::camera::CameraShake;
pub use crate::beginner::collections::{
    CameraOps, EnemyCollection, FiredShot, PickupCollection, PlayerActor, Score, ScoreOps,
    ShootAuthor, TaggedActors,
};
pub use crate::beginner::combat::MeleeCombatConfig;
pub use crate::beginner::context::{Game, Seconds, StartupGame};
pub use crate::beginner::debug::DebugOverlay;
pub use crate::beginner::defaults::TopDownGameAuthor;
pub use crate::beginner::defaults::{
    AnimationUpdateBehavior, CameraFollowBehavior, CameraShakeBehavior, CameraZoomBehavior,
    CollisionBehavior, DeathStateBehavior, DirectionalAttackBehavior,
    EnemyAnimationByMovementBehavior, EnemyChaseBehavior, EnemyDirectionalAnimationBehavior,
    EnemyPatrolBehavior, MeleeCombatBehavior, MovementBehavior, PauseDeathUiBehavior,
    PlayerAnimationByMovementBehavior, PlayerDirectionalAnimationBehavior, PlayerFacingBehavior,
    SimpleGameStartupBehavior,
};
pub use crate::beginner::events::{
    AnimationFinishedEvent, CollectEvent, CollisionEvent, EnemyDeathEvent, EventActor,
};
pub use crate::beginner::prefabs::{
    AreaPrefabAuthor, DoorPrefabAuthor, EnemyPrefabAuthor, PickupPrefabAuthor, PlayerPrefabAuthor,
    ProjectilePrefabAuthor, SpawnerPrefabAuthor,
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
pub use crate::bundle::vec2s;
/// Defines a content plugin without requiring `GamePlugin` boilerplate.
pub use crate::content_plugin;
pub use crate::input::{ActionAuthor, Axis2dAuthor, InputAuthor, TopDownControls};
pub use crate::map::MapAuthor;
pub use crate::prefab::{IntoContentName, IntoMovementAxis};

//! Advanced content imports.
//!
//! This surface keeps the lower-level facade available for custom systems,
//! manual prefab bundles, and engine-facing tests.

pub use anyhow::{Context, Result};
pub use glam::{Vec2, Vec4, vec2, vec4};

pub use game_ai::{AiController, ChaseTarget, PathFollow, Patrol};
pub use game_combat::{Faction, FactionId, Health, MeleeAttack};
pub use game_core::backend::{FontHandle, SoundHandle, TextureHandle};
pub use game_core::camera::Camera2D;
pub use game_core::input::{
    ActionId, Axis2dId, GamepadAxis, GamepadButton, Input, Key, MouseButton,
};
pub use game_core::query::{DeltaTime, Query, Res, ResMut, With, Without};
pub use game_core::world::{Component, EntityId, Sprite, Transform, Velocity};
pub use game_map::{MapCell, cell};
pub use game_physics::Collider;

pub use crate::app::{DebugOverlayAuthor, FnGamePlugin, GameApp, GamePlugin, plugin, plugin_fn};
pub use crate::assets::{
    AssetAuthor, AssetBag, AssetBagAuthor, AssetFolderAuthor, SoundRef, TextureRef,
};
pub use crate::beginner::actors::{
    CollectSound, Collectible, DeathAnimationPolicy, DespawnOnCollect, DespawnOnHit, Door,
    DoorAction, DoorTarget, Enemy, ExitDoor, Lifetime, Name, Npc, Pickup, Player, PlayerMovement,
    Projectile, ProjectileDamage, ScoreValue, Solid, Spawner, Speed,
};
pub use crate::beginner::animation::{
    Animation, AnimationClip, AnimationSet, SpriteSheet, attack_frames, die_frames, frames,
    idle_frames, walk_frames,
};
pub use crate::beginner::camera::CameraShake;
pub use crate::beginner::collections::{
    CameraOps, EnemyCollection, FiredShot, PickupCollection, PlayerActor, Score, ScoreOps,
    ShootAuthor,
};
pub use crate::beginner::combat::MeleeCombatConfig;
pub use crate::beginner::context::{Game, Seconds, StartupGame};
pub use crate::beginner::debug::DebugOverlay;
pub use crate::beginner::defaults::TopDownGameAuthor;
pub use crate::beginner::events::{
    AnimationFinishedEvent, CollectEvent, CollisionEvent, EnemyDeathEvent, EventActor,
};
pub use crate::beginner::prefabs::{
    DoorPrefabAuthor, EnemyPrefabAuthor, PickupPrefabAuthor, PlayerPrefabAuthor,
    ProjectilePrefabAuthor, SpawnerPrefabAuthor,
};
pub use crate::beginner::rules::RulesAuthor;
pub use crate::beginner::scene::{SceneRegistry, SceneState, SimpleSceneFlowAuthor};
pub use crate::beginner::spawn::SpawnAuthor;
pub use crate::beginner::state::SimpleGameState;
pub use crate::beginner::ui::{UiOps, UiText};
pub use crate::bundle::{Bundle, vec2s};
pub use crate::context::{Commands, GameCtx, StartupGameCtx};
pub use crate::helpers::{
    InputDriven, MovementSpeed, SimulationState, camera_follow_first, stop_all_velocity,
};
pub use crate::input::{ActionAuthor, Axis2dAuthor, InputAuthor, TopDownControls};
pub use crate::map::{MapAuthor, TileTheme};
pub use crate::prefab::PrefabAuthor;
pub use crate::system::{GameSystem, StartupSystem};

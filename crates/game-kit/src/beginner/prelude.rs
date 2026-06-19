//! Beginner import surface.
//!
//! This module intentionally keeps low-level query and scheduling plumbing out
//! of the default beginner import path. Use `game_kit::advanced::prelude::*`
//! when a project needs the lower-level facade.

pub use anyhow::{Context, Result};
pub use glam::{Vec2, Vec4, vec2, vec4};

pub use game_core::backend::{FontHandle, SoundHandle, TextureHandle};
pub use game_core::input::{ActionId, Axis2dId, Key, MouseButton};
pub use game_map::{MapCell, cell};

pub use crate::app::{GameApp, GamePlugin, Plugin, plugin, plugin_fn};
pub use crate::assets::{AssetAuthor, AssetBag, AssetBagAuthor};
pub use crate::beginner::actors::{
    Door, Enemy, Name, Npc, Pickup, Player, PlayerMovement, Projectile, ScoreValue, Solid, Speed,
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
pub use crate::bundle::vec2s;
pub use crate::input::{ActionAuthor, Axis2dAuthor, InputAuthor, TopDownControls};
pub use crate::map::MapAuthor;

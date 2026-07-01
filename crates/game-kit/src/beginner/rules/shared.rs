pub(super) use std::collections::HashMap;

pub(super) use glam::Vec2;

pub(super) use anyhow::Result;

pub(super) use crate::app::{GameApp, GamePlugin};
pub(super) use crate::beginner::actors::{
    Checkpoint, CheckpointState, DeathAnimationPolicy, DespawnOnHit, Door, DoorAction, DoorTarget,
    DropSpawned, DropsPrefab, Enemy, Lifetime, PlayerProjectile, PrefabName, Projectile,
    ProjectileDamage, ProjectileImpact, SpawnPlacement, Spawner,
};
pub(super) use crate::beginner::defaults::{
    enemy_directional_animation_system, player_directional_animation_system,
};
pub(super) use crate::beginner::events::DEFAULT_PICKUP_COLLECT_RANGE;
pub(super) use crate::beginner::state::SimpleGameState;
pub(super) use crate::context::GameCtx;
pub(super) use crate::diagnostics::bad_rule_combo_error;
pub(super) use crate::input::TopDownControls;

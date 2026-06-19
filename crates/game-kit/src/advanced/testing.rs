//! Advanced test imports for raw ECS/world inspection.

pub mod prelude {
    pub use anyhow::{Context, Result};
    pub use glam::{Vec2, Vec4, vec2, vec4};

    pub use game_ai::{AiController, ChaseTarget, PathFollow, Patrol, chase_system, patrol_system};
    pub use game_combat::{Faction, FactionId, Health, MeleeAttack, apply_damage};
    pub use game_core::backend::{FontHandle, SoundHandle, TextureHandle};
    pub use game_core::builder::PrefabId;
    pub use game_core::camera::Camera2D;
    pub use game_core::input::{ActionId, Axis2dId, Input, Key, MouseButton};
    pub use game_core::nav::NavGrid;
    pub use game_core::tilemap::TileMap;
    pub use game_core::world::{Component, Entity, EntityId, Sprite, Transform, Velocity, World};
    pub use game_map::{MapCell, cell};
    pub use game_physics::{Collider, movement_system};

    pub use crate::beginner::testing::TestEntity;
    pub use crate::prelude::*;
    pub use crate::testing::GameTestHarness;
}

//! Standard beginner actor components.

use game_core::backend::SoundHandle;
use game_core::input::Axis2dId;

use crate::helpers::{InputDriven, MovementSpeed};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Name(pub String);

impl Name {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The source prefab registered for an entity. This stays internal to the
/// beginner facade so custom rules can match authored prefab names without
/// exposing entity ids or ECS components.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PrefabName(pub String);

impl PrefabName {
    pub(crate) fn matches(&self, name: &str) -> bool {
        self.0 == name
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Player;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Enemy;

/// Enemy prefab policy for playing a configured `die` clip before removal.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DeathAnimationPolicy {
    pub despawn_after_animation: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Npc;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Pickup;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Collectible;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScoreValue(pub i32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CollectSound(pub SoundHandle);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DespawnOnCollect;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Door;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ExitDoor;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DoorAction {
    ChangeMap(String),
    ChangeScene(String),
    RestartLevel,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DoorTarget {
    pub action: DoorAction,
    pub requires_all_enemies_dead: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Projectile;

/// Marks a projectile produced through [`crate::beginner::collections::PlayerActor::shoot`].
/// Rules use it to target enemies without exposing factions or raw ECS queries to
/// beginner content.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PlayerProjectile;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProjectileDamage {
    pub amount: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lifetime {
    pub seconds_left: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DespawnOnHit;

#[derive(Clone, Debug, PartialEq)]
pub enum SpawnPlacement {
    AtSpawner,
    NearPlayer { radius: f32 },
    AtFirstFloor,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Spawner {
    pub prefab: String,
    pub every_seconds: f32,
    pub timer: f32,
    pub max_alive: Option<usize>,
    pub placement: SpawnPlacement,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Solid;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Speed(pub f32);

impl Speed {
    pub fn new(units_per_second: f32) -> Self {
        Self(units_per_second)
    }
}

impl MovementSpeed for Speed {
    fn units_per_second(&self) -> f32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayerMovement {
    pub axis: Axis2dId,
}

impl PlayerMovement {
    pub fn axis(axis: Axis2dId) -> Self {
        Self { axis }
    }
}

impl InputDriven for PlayerMovement {
    fn movement_axis(&self) -> Axis2dId {
        self.axis
    }
}

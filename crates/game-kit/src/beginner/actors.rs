//! Standard beginner actor components.

use game_core::backend::SoundHandle;
use game_core::input::Axis2dId;
use glam::Vec2;

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

/// The player's most recent movement direction. It is maintained internally by
/// directional movement/attack rules so a stationary player can still use the
/// expected directional attack clip.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum FacingDirection {
    Up,
    #[default]
    Down,
    Left,
    Right,
}

impl FacingDirection {
    pub(crate) fn from_motion(motion: Vec2) -> Option<Self> {
        if motion.length_squared() <= 0.0001 {
            return None;
        }
        Some(if motion.x.abs() >= motion.y.abs() {
            if motion.x >= 0.0 {
                Self::Right
            } else {
                Self::Left
            }
        } else if motion.y >= 0.0 {
            Self::Down
        } else {
            Self::Up
        })
    }

    pub(crate) const fn attack_clip(self) -> &'static str {
        match self {
            Self::Up => "attack_up",
            Self::Down => "attack_down",
            Self::Left => "attack_left",
            Self::Right => "attack_right",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Enemy;

/// Marks a non-solid overlap zone authored through `game.area_prefab(...)`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TriggerArea;

/// Marks an authored trigger area as a respawn checkpoint.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Checkpoint {
    pub enabled: bool,
}

/// The last checkpoint activated by the player. It survives world resets so a
/// checkpoint can move the player after a restart.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CheckpointState {
    pub position: Option<Vec2>,
}

/// A semantic alias for an authored trigger area. Keeping this separate from
/// [`TriggerArea`] lets future area rules add behavior without changing the
/// lightweight trigger marker.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Area;

/// The authored name of an area. Event callbacks use prefab names for matching,
/// while this component keeps that intent available to future area features.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AreaName(pub String);

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

/// Amount of health restored when a pickup is collected. A value of zero keeps
/// the pickup as a score-only collectible.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HealValue(pub i32);

/// A prefab dropped at a defeated enemy's position. A zero chance disables the
/// drop, which keeps the component harmless on enemies without `.drops(...)`.
#[derive(Clone, Debug, PartialEq)]
pub struct DropsPrefab {
    pub prefab: String,
    pub chance: f32,
}

impl Default for DropsPrefab {
    fn default() -> Self {
        Self {
            prefab: String::new(),
            chance: 0.0,
        }
    }
}

/// Internal marker preventing a defeated enemy from creating the same drop on
/// every simulation tick before it despawns.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DropSpawned;

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

/// Marks a projectile that has hit and is playing its optional `impact` clip.
/// Projectile rules freeze it until that one-shot clip finishes, then remove it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ProjectileImpact;

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

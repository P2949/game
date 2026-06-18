//! Beginner import surface.

pub use crate::beginner::actors::{
    Door, Enemy, Name, Npc, Pickup, Player, PlayerMovement, Projectile, Solid, Speed,
};
pub use crate::beginner::animation::{
    Animation, AnimationClip, AnimationSet, PlayerActor, SpriteSheet,
};
pub use crate::beginner::combat::MeleeCombatConfig;
pub use crate::beginner::context::{Game, Seconds, StartupGame};
pub use crate::beginner::debug::DebugOverlay;
pub use crate::beginner::defaults::TopDownGameAuthor;
pub use crate::beginner::prefabs::{EnemyPrefabAuthor, PlayerPrefabAuthor};
pub use crate::beginner::scene::{SceneRegistry, SceneState};
pub use crate::beginner::state::SimpleGameState;
pub use crate::prelude::*;

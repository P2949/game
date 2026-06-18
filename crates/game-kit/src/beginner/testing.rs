//! Beginner test helpers.

use game_combat::Health;
use game_core::world::{EntityId, Transform, World};
use glam::Vec2;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TestEntity {
    id: EntityId,
    health: Option<Health>,
    position: Option<Vec2>,
}

impl TestEntity {
    pub(crate) fn from_world(id: EntityId, world: &World) -> Self {
        Self {
            id,
            health: world.get::<Health>(id).copied(),
            position: world.get::<Transform>(id).map(|transform| transform.pos),
        }
    }

    pub(crate) fn id(self) -> EntityId {
        self.id
    }

    pub fn health(self) -> i32 {
        self.health
            .unwrap_or_else(|| panic!("entity {:?} has no Health component", self.id))
            .current
    }

    pub fn max_health(self) -> i32 {
        self.health
            .unwrap_or_else(|| panic!("entity {:?} has no Health component", self.id))
            .max
    }

    pub fn is_dead(self) -> bool {
        self.health
            .unwrap_or_else(|| panic!("entity {:?} has no Health component", self.id))
            .is_dead()
    }

    pub fn position(self) -> Vec2 {
        self.position
            .unwrap_or_else(|| panic!("entity {:?} has no Transform component", self.id))
    }
}

use game_core::world::EntityId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeathEvent {
    pub entity: EntityId,
}

impl DeathEvent {
    pub fn new(entity: EntityId) -> Self {
        Self { entity }
    }
}

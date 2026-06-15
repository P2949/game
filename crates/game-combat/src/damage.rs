use game_core::world::EntityId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DamageEvent {
    pub source: Option<EntityId>,
    pub target: EntityId,
    pub amount: i32,
}

impl DamageEvent {
    pub fn new(source: Option<EntityId>, target: EntityId, amount: i32) -> Self {
        Self {
            source,
            target,
            amount,
        }
    }
}

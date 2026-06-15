use game_core::world::{EntityId, World};

use crate::Health;

pub fn apply_damage(world: &mut World, target: EntityId, amount: i32) -> bool {
    let Some(health) = world.get_mut::<Health>(target) else {
        return false;
    };
    health.damage(amount);
    true
}

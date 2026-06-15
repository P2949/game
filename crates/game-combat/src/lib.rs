pub mod damage;
pub mod events;
pub mod faction;
pub mod health;
pub mod melee;
pub mod systems;

pub use damage::DamageEvent;
pub use events::DeathEvent;
pub use faction::{Faction, FactionId};
pub use health::Health;
pub use melee::MeleeAttack;
pub use systems::apply_damage;

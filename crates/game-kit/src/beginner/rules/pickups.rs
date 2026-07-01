use super::shared::*;

/// Collects ordinary pickup prefabs near the player every fixed tick.
pub struct CollectPickupsBehavior;

impl GamePlugin for CollectPickupsBehavior {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        game.fixed(|game: &mut GameCtx<'_, '_>, _dt| {
            game.collect_pickups_near_player(DEFAULT_PICKUP_COLLECT_RANGE);
        });
        Ok(())
    }
}

use super::shared::*;

/// Changes to the conventional `win` scene once selected objectives are met.
pub struct WinConditionBehavior {
    pub require_pickups: bool,
    pub require_enemies: bool,
}

impl GamePlugin for WinConditionBehavior {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        let require_pickups = self.require_pickups;
        let require_enemies = self.require_enemies;
        game.fixed(move |game: &mut GameCtx<'_, '_>, _dt| {
            let pickups_done = !require_pickups || game.pickups().count() == 0;
            let enemies_done = !require_enemies || game.enemies().alive().count() == 0;
            if pickups_done && enemies_done {
                game.change_scene_or_log("win");
            }
        });
        Ok(())
    }
}

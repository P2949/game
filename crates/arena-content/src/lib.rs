pub mod actor;
pub mod assets;
pub mod combat;
pub mod input;
pub mod level;
pub mod prefabs;
pub mod state;
pub mod systems;

use game_kit::prelude::*;

pub struct ArenaPlugin;

pub fn plugin() -> game_kit::Plugin<ArenaPlugin> {
    game_kit::plugin(ArenaPlugin)
}

impl GamePlugin for ArenaPlugin {
    fn build(&self, game: &mut GameApp) -> anyhow::Result<()> {
        let assets = game.assets(crate::assets::register)?;
        let actions = game.input(crate::input::register)?;

        crate::prefabs::register(game, assets, actions)?;
        crate::level::register(game, assets);
        crate::systems::register(game, assets, actions);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use game_kit::testing::prelude::*;

    use super::ArenaPlugin;
    use crate::actor::{EnemyTag, PlayerController};

    #[test]
    fn arena_plugin_builds_and_spawns_start_map() {
        let game = GameTestHarness::from_plugin(ArenaPlugin).unwrap();

        assert_eq!(game.world().ids_with::<PlayerController>().len(), 1);
        assert_eq!(game.world().ids_with::<EnemyTag>().len(), 1);
    }
}

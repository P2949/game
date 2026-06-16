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

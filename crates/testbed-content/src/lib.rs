//! Second demo content plugin. It defines a distinct map, three prefabs (player,
//! chasing enemy, patrolling enemy), and its own systems while depending on the
//! author-facing `game-kit` facade rather than runtime/backend crates.

pub mod actor;
pub mod assets;
pub mod combat;
pub mod input;
pub mod level;
pub mod prefabs;
pub mod state;
pub mod systems;

use game_kit::prelude::*;

pub struct TestbedPlugin;

pub fn plugin() -> game_kit::Plugin<TestbedPlugin> {
    game_kit::plugin(TestbedPlugin)
}

impl GamePlugin for TestbedPlugin {
    fn build(&self, game: &mut GameApp) -> anyhow::Result<()> {
        let assets = game.assets(assets::register)?;
        let actions = game.input(input::register)?;

        prefabs::register(game, assets, actions)?;

        game.map_from_ron(level::TESTBED_MAP_RON)
            .theme(level::theme(&assets))
            .require_object("player_start")
            .start();

        systems::register(game, assets, actions);
        Ok(())
    }
}

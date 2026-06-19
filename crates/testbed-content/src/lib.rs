//! Advanced testbed content plugin.
//!
//! This crate intentionally demonstrates the advanced `game-kit` surface:
//! manual system wiring, RON maps, patrol setup, and custom state. Beginner
//! examples should copy `simple-content` or `arena-content`, not this crate.

pub mod assets;
pub mod combat;
pub mod input;
pub mod level;
pub mod prefabs;
pub mod state;
pub mod systems;

use game_kit::advanced::prelude::*;

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

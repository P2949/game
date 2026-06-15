pub mod actor;
pub mod ai;
pub mod assets;
pub mod combat;
pub mod input;
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
        let assets = game.assets(crate::assets::register);
        let actions = game.input(crate::input::register);

        crate::prefabs::register(game, assets, actions);

        game.map("arena")
            .tile_size(32.0)
            .tiles([
                "###############",
                "#.............#",
                "#....#####....#",
                "#....#...#....#",
                "#.............#",
                "#....#...#....#",
                "#....#####....#",
                "#.............#",
                "###############",
            ])
            .theme(TileTheme {
                floor: Sprite::new(assets.floor, vec2(32.0, 32.0))
                    .layer(0)
                    .tint(vec4(0.12, 0.12, 0.16, 1.0)),
                wall: Sprite::new(assets.wall, vec2(32.0, 32.0))
                    .layer(1)
                    .tint(vec4(0.40, 0.42, 0.50, 1.0)),
            })
            .spawn("player_start", crate::prefabs::PLAYER, cell(3, 4))
            .spawn("enemy_01", crate::prefabs::SLIME, cell(9, 4))
            .require_object("player_start")
            .start();

        crate::systems::register(game, assets, actions);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use game_kit::prelude::*;

    use super::ArenaPlugin;
    use crate::actor::{EnemyTag, PlayerController};

    #[test]
    fn arena_plugin_builds_and_spawns_start_map() {
        let game = GameTestHarness::from_plugin(ArenaPlugin).unwrap();

        assert_eq!(game.world().ids_with::<PlayerController>().len(), 1);
        assert_eq!(game.world().ids_with::<EnemyTag>().len(), 1);
    }
}

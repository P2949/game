use game_kit::prelude::*;

use crate::assets::ArenaAssets;
use crate::{PLAYER, SLIME};

pub fn register(game: &mut GameApp, assets: ArenaAssets) {
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
        .simple_theme(assets.floor, assets.wall)
        .spawn("player_start", PLAYER, cell(3, 4))
        .spawn("enemy_01", SLIME, cell(9, 4))
        .require_object("player_start")
        .start();
}

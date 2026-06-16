use game_kit::prelude::*;

use crate::assets::ArenaAssets;
use crate::prefabs::{PLAYER, SLIME};

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
        .theme(TileTheme {
            floor: Sprite::new(assets.floor, vec2(32.0, 32.0))
                .layer(0)
                .tint(vec4(0.12, 0.12, 0.16, 1.0)),
            wall: Sprite::new(assets.wall, vec2(32.0, 32.0))
                .layer(1)
                .tint(vec4(0.40, 0.42, 0.50, 1.0)),
        })
        .spawn("player_start", PLAYER, cell(3, 4))
        .spawn("enemy_01", SLIME, cell(9, 4))
        .require_object("player_start")
        .start();
}

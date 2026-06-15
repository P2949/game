use game_map::{GameMap, MapBuilder, cell};
use glam::{Vec2, Vec4};

use crate::assets::ArenaAssets;
use crate::prefabs::ArenaPrefabs;
use game_core::app::TileTheme;
use game_core::world::Sprite;

pub const TILE: f32 = 32.0;

pub fn arena_map(prefabs: ArenaPrefabs) -> GameMap {
    MapBuilder::new("arena", TILE)
        .tile_layer(
            "collision",
            &[
                "###############",
                "#.............#",
                "#....#####....#",
                "#....#...#....#",
                "#.............#",
                "#....#...#....#",
                "#....#####....#",
                "#.............#",
                "###############",
            ],
        )
        .object("player_start", prefabs.player, cell(3, 4))
        .object("enemy_01", prefabs.slime, cell(9, 4))
        .finish()
}

pub fn theme(assets: &ArenaAssets) -> TileTheme {
    let square = Vec2::splat(TILE);
    TileTheme {
        floor: Sprite::new(assets.floor, square)
            .layer(0)
            .tint(Vec4::new(0.12, 0.12, 0.16, 1.0)),
        wall: Sprite::new(assets.wall, square)
            .layer(1)
            .tint(Vec4::new(0.40, 0.42, 0.50, 1.0)),
    }
}

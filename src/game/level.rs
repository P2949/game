use glam::{Vec2, Vec4};

use crate::engine::app::TileTheme;
use crate::engine::assets::Assets;
use crate::engine::tilemap::TileMap;
use crate::engine::world::Sprite;

pub const TILE: f32 = 32.0;

pub fn arena() -> TileMap {
    TileMap::from_rows(
        &[
            "###############",
            "#.............#",
            "#....#####....#",
            "#....#...#....#",
            "#..P.....E....#",
            "#....#...#....#",
            "#....#####....#",
            "#.............#",
            "###############",
        ],
        TILE,
    )
}

pub fn theme(assets: &Assets) -> TileTheme {
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

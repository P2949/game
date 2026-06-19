use game_kit::advanced::prelude::*;

use crate::assets::TestbedAssets;

pub const TILE: f32 = 32.0;

/// The testbed map as an external RON content file (Phase 13). Embedded at build
/// time so the demo runs from external content without runtime file IO.
pub const TESTBED_MAP_RON: &str = include_str!("../maps/testbed.ron");

pub fn theme(assets: &TestbedAssets) -> TileTheme {
    let square = Vec2::splat(TILE);
    TileTheme {
        floor: Sprite::new(assets.floor, square)
            .layer(0)
            .tint(Vec4::new(0.10, 0.14, 0.12, 1.0)),
        wall: Sprite::new(assets.wall, square)
            .layer(1)
            .tint(Vec4::new(0.30, 0.46, 0.40, 1.0)),
    }
}

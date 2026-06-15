use game_core::app::TileTheme;
use game_core::builder::PrefabRegistry;
use game_core::world::Sprite;
use game_map::{GameMap, MapBuilder, cell, load_game_map_ron};
use glam::{Vec2, Vec4};

use crate::assets::TestbedAssets;
use crate::prefabs::TestbedPrefabs;

pub const TILE: f32 = 32.0;

/// The testbed map as an external RON content file (Phase 13). Embedded at build
/// time so the demo runs from external content without runtime file IO.
pub const TESTBED_MAP_RON: &str = include_str!("../maps/testbed.ron");

/// Loads the testbed map from the embedded RON file, resolving prefab names
/// against `prefabs`. This is the engine's external-content path in action.
pub fn testbed_map_from_ron(prefabs: &PrefabRegistry) -> anyhow::Result<GameMap> {
    load_game_map_ron(TESTBED_MAP_RON, |name| prefabs.id(name))
}

/// A larger, distinct layout from the arena (17x11 with a central pillar) to prove
/// content — not the engine — owns the map. The top corridor (row 1) is left open
/// for the patroller's horizontal route.
pub fn testbed_map(prefabs: TestbedPrefabs) -> GameMap {
    MapBuilder::new("testbed", TILE)
        .tile_layer(
            "collision",
            &[
                "#################",
                "#...............#",
                "#...............#",
                "#...............#",
                "#......###......#",
                "#......###......#",
                "#......###......#",
                "#...............#",
                "#...............#",
                "#...............#",
                "#################",
            ],
        )
        .object("player_start", prefabs.player, cell(1, 1))
        .object("patroller_01", prefabs.patroller, cell(8, 1))
        .object("chaser_01", prefabs.chaser, cell(15, 9))
        .finish()
}

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

#[cfg(test)]
mod tests {
    use game_core::builder::PrefabRegistry;
    use game_core::input::InputRegistry;
    use game_map::validate_map;

    use crate::assets::TestbedAssets;
    use crate::prefabs;

    use super::{testbed_map, testbed_map_from_ron};

    /// The external RON map must stay in lockstep with the Rust builder reference:
    /// same dimensions, objects, prefab assignments, and spawn positions.
    #[test]
    fn ron_map_matches_builder_reference_and_validates() {
        let assets = TestbedAssets::load();
        let mut input = InputRegistry::new();
        let actions = crate::input::register(&mut input);
        let mut registry = PrefabRegistry::new();
        let prefab_ids = prefabs::register(&mut registry, assets, actions);

        let builder_map = testbed_map(prefab_ids);
        let ron_map = testbed_map_from_ron(&registry).unwrap();

        assert_eq!(
            ron_map.collision_tilemap().width(),
            builder_map.collision_tilemap().width()
        );
        assert_eq!(
            ron_map.collision_tilemap().height(),
            builder_map.collision_tilemap().height()
        );
        assert_eq!(ron_map.objects.len(), builder_map.objects.len());
        for (ron_obj, builder_obj) in ron_map.objects.iter().zip(builder_map.objects.iter()) {
            assert_eq!(ron_obj.id, builder_obj.id);
            assert_eq!(ron_obj.prefab, builder_obj.prefab);
            assert_eq!(ron_obj.position, builder_obj.position);
        }

        validate_map(&ron_map).unwrap();
    }
}

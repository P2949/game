use game_core::builder::{MapId, MapRegistry};
use game_map::GameMap;

use crate::assets::TestbedAssets;

/// Registers a prebuilt testbed map's collision tilemap and theme, returning the
/// start map id. The `GameMap` is constructed by the caller (from RON), keeping
/// this function agnostic to where the map data came from.
pub fn register(maps: &mut MapRegistry, assets: &TestbedAssets, map: &GameMap) -> MapId {
    maps.register(
        "testbed",
        map.collision_tilemap(),
        crate::level::theme(assets),
    )
}

#[cfg(test)]
mod tests {
    use super::register;
    use crate::assets::TestbedAssets;
    use crate::prefabs;
    use game_core::builder::{MapRegistry, PrefabRegistry};
    use game_core::input::InputRegistry;

    #[test]
    fn testbed_map_registers_with_three_objects() {
        let assets = TestbedAssets::load();
        let mut input = InputRegistry::new();
        let actions = crate::input::register(&mut input);
        let mut prefab_registry = PrefabRegistry::new();
        let _ = prefabs::register(&mut prefab_registry, assets, actions);
        let map = crate::level::testbed_map_from_ron(&prefab_registry).unwrap();

        let mut registry = MapRegistry::new();
        let id = register(&mut registry, &assets, &map);

        let registered = registry.get(id).unwrap();
        assert_eq!(registered.name, "testbed");
        assert_eq!(registered.data.tilemap.width(), 17);
        assert_eq!(registered.data.tilemap.height(), 11);
        assert_eq!(map.objects.len(), 3);
    }
}

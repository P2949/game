use crate::assets::ArenaAssets;
use crate::engine::builder::{MapId, MapRegistry};
use crate::prefabs::ArenaPrefabs;
use game_map::GameMap;

pub fn register(
    maps: &mut MapRegistry,
    assets: &ArenaAssets,
    prefabs: ArenaPrefabs,
) -> (MapId, GameMap) {
    let map = crate::level::arena_map(prefabs);
    let id = maps.register(
        "arena",
        map.collision_tilemap(),
        crate::level::theme(assets),
    );
    (id, map)
}

#[cfg(test)]
mod tests {
    use super::register;
    use crate::assets::ArenaAssets;
    use crate::engine::builder::MapRegistry;
    use crate::engine::input::InputRegistry;
    use crate::prefabs;

    #[test]
    fn arena_map_registers_as_startable_map_data() {
        let assets = ArenaAssets::load();
        let mut input = InputRegistry::new();
        let actions = crate::input::register(&mut input);
        let mut prefab_registry = crate::engine::builder::PrefabRegistry::new();
        let prefabs = prefabs::register(&mut prefab_registry, assets, actions);
        let mut registry = MapRegistry::new();
        let (id, game_map) = register(&mut registry, &assets, prefabs);

        let map = registry.get(id).unwrap();
        assert_eq!(map.name, "arena");
        assert_eq!(map.data.tilemap.width(), 15);
        assert_eq!(map.data.tilemap.height(), 9);
        assert_eq!(game_map.objects.len(), 2);
    }
}

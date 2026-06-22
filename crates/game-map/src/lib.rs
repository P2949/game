pub mod builder;
pub mod external;
pub mod ldtk;
pub mod nav;
pub mod object;
pub mod region;
pub mod tiled;
pub mod tilemap;
pub mod validation;

use tilemap::TileMap;

// The map identifier and property types are core, lowest-level concepts owned by
// `game-core` (which no longer depends on `game-map`); re-exported here so map
// content can keep referring to them as `game_map::{MapId, PrefabId, PropertyBag}`.
pub use game_core::builder::{MapId, PrefabId, PropertyBag};

#[derive(Clone, Debug)]
pub struct GameMap {
    /// Author-facing name of the map (as written in the builder or RON file). This
    /// is distinct from the registry-assigned [`MapId`], which is minted by the
    /// host's map registry when the map is registered — a `GameMap` on its own has
    /// no runtime id.
    pub name: String,
    pub tile_size: f32,
    pub layers: Vec<TileLayer>,
    pub objects: Vec<MapObject>,
    pub regions: Vec<MapRegion>,
}

impl GameMap {
    pub fn collision_tilemap(&self) -> TileMap {
        self.layers
            .iter()
            .find(|layer| layer.id == "collision")
            .map(|layer| layer.tiles.clone())
            .unwrap_or_else(|| TileMap::from_rows(&[], self.tile_size))
    }
}

#[derive(Clone, Debug)]
pub struct TileLayer {
    pub id: String,
    pub tiles: TileMap,
}

pub use builder::{MapBuilder, MapCell, cell};
pub use external::{GameMapFile, MapObjectFile, TileLayerFile, load_game_map_ron};
pub use ldtk::{ImportedLdtkEntity, ImportedLdtkLevel, load_ldtk_level, load_ldtk_level_file};
pub use object::MapObject;
pub use region::{MapRegion, RegionShape, Tags};
pub use tiled::{ImportedTiledMap, ImportedTiledObject, load_tiled_map, load_tiled_map_file};
pub use validation::{MapValidator, validate_map, validate_map_prefabs};

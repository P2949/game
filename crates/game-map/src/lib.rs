pub mod builder;
pub mod external;
pub mod nav;
pub mod object;
pub mod region;
pub mod tilemap;
pub mod validation;

use std::collections::HashMap;

use tilemap::TileMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MapId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PrefabId(pub u32);

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PropertyBag {
    values: HashMap<String, String>,
}

impl PropertyBag {
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.values.insert(key.into(), value.into());
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(String::as_str)
    }
}

#[derive(Clone, Debug)]
pub struct GameMap {
    pub id: MapId,
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
pub use object::MapObject;
pub use region::{MapRegion, RegionShape, Tags};
pub use validation::{MapValidator, validate_map};

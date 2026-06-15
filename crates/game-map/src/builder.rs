use crate::tilemap::TileMap;
use crate::{GameMap, MapId, MapObject, MapRegion, PrefabId, PropertyBag, TileLayer};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MapCell {
    col: usize,
    row: usize,
}

pub fn cell(col: usize, row: usize) -> MapCell {
    MapCell { col, row }
}

pub struct MapBuilder {
    tile_size: f32,
    layers: Vec<TileLayer>,
    objects: Vec<MapObject>,
    regions: Vec<MapRegion>,
}

impl MapBuilder {
    pub fn new(_id: impl Into<String>, tile_size: f32) -> Self {
        Self {
            tile_size,
            layers: Vec::new(),
            objects: Vec::new(),
            regions: Vec::new(),
        }
    }

    pub fn tile_layer(mut self, id: impl Into<String>, rows: &[&str]) -> Self {
        self.layers.push(TileLayer {
            id: id.into(),
            tiles: TileMap::from_rows(rows, self.tile_size),
        });
        self
    }

    pub fn object(mut self, id: impl Into<String>, prefab: PrefabId, cell: MapCell) -> Self {
        self.objects.push(MapObject {
            id: id.into(),
            prefab,
            position: self.cell_center(cell),
            properties: PropertyBag::default(),
        });
        self
    }

    pub fn region(mut self, region: MapRegion) -> Self {
        self.regions.push(region);
        self
    }

    pub fn finish(self) -> GameMap {
        GameMap {
            id: MapId(0),
            tile_size: self.tile_size,
            layers: self.layers,
            objects: self.objects,
            regions: self.regions,
        }
    }

    fn cell_center(&self, cell: MapCell) -> glam::Vec2 {
        glam::vec2(
            (cell.col as f32 + 0.5) * self.tile_size,
            (cell.row as f32 + 0.5) * self.tile_size,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{MapBuilder, cell};
    use crate::PrefabId;

    #[test]
    fn builder_adds_collision_layer_and_objects() {
        let map = MapBuilder::new("arena", 32.0)
            .tile_layer("collision", &["#.", ".."])
            .object("player_start", PrefabId(0), cell(1, 1))
            .finish();

        assert_eq!(map.collision_tilemap().width(), 2);
        assert_eq!(map.objects[0].position, glam::vec2(48.0, 48.0));
    }
}

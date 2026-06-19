use crate::tilemap::TileMap;
use crate::{GameMap, MapObject, MapRegion, PrefabId, PropertyBag, TileLayer};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MapCell {
    col: usize,
    row: usize,
}

impl MapCell {
    pub fn col(self) -> usize {
        self.col
    }

    pub fn row(self) -> usize {
        self.row
    }
}

pub fn cell(col: usize, row: usize) -> MapCell {
    MapCell { col, row }
}

pub struct MapBuilder {
    name: String,
    tile_size: f32,
    layers: Vec<TileLayer>,
    objects: Vec<MapObject>,
    regions: Vec<MapRegion>,
}

impl MapBuilder {
    pub fn new(name: impl Into<String>, tile_size: f32) -> Self {
        Self {
            name: name.into(),
            tile_size,
            layers: Vec::new(),
            objects: Vec::new(),
            regions: Vec::new(),
        }
    }

    /// Adds a tile layer from strict rows (`.`/`#`, rectangular), **panicking** on
    /// malformed rows. Intended for in-code maps whose rows are compile-time
    /// literals: a bad literal is a programming error that should fail loudly at
    /// startup rather than be silently rewritten to floor. For untrusted/external
    /// rows (e.g. RON files) use [`Self::try_tile_layer`].
    pub fn tile_layer(self, id: impl Into<String>, rows: &[&str]) -> Self {
        self.try_tile_layer(id, rows)
            .expect("in-code tile layer rows must be valid")
    }

    /// Fallible counterpart to [`Self::tile_layer`]: returns an error (rather than
    /// sanitizing) when the rows contain an invalid character or are not
    /// rectangular, so malformed external content is rejected by validation.
    pub fn try_tile_layer(mut self, id: impl Into<String>, rows: &[&str]) -> anyhow::Result<Self> {
        self.layers.push(TileLayer {
            id: id.into(),
            tiles: TileMap::try_from_rows(rows, self.tile_size)?,
        });
        Ok(self)
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
            name: self.name,
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

        assert_eq!(map.name, "arena");
        assert_eq!(map.collision_tilemap().width(), 2);
        assert_eq!(map.objects[0].position, glam::vec2(48.0, 48.0));
    }

    #[test]
    fn try_tile_layer_rejects_invalid_rows() {
        // `MapBuilder` is not `Debug`, so inspect the error via `.err()` rather
        // than `.unwrap_err()`.
        let err = MapBuilder::new("bad", 32.0)
            .try_tile_layer("collision", &["#P"])
            .err()
            .expect("invalid rows must be rejected");

        assert!(err.to_string().contains("invalid tile character"));
    }
}

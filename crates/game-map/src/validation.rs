use std::collections::HashSet;

use crate::nav::NavGrid;
use crate::{GameMap, PrefabId, RegionShape};

pub fn validate_map(map: &GameMap) -> anyhow::Result<()> {
    MapValidator::new().validate(map)
}

#[derive(Default)]
pub struct MapValidator {
    known_prefabs: HashSet<PrefabId>,
    required_objects: HashSet<String>,
}

impl MapValidator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn allow_prefab(mut self, prefab: PrefabId) -> Self {
        self.known_prefabs.insert(prefab);
        self
    }

    pub fn allow_prefabs(mut self, prefabs: impl IntoIterator<Item = PrefabId>) -> Self {
        self.known_prefabs.extend(prefabs);
        self
    }

    pub fn require_object(mut self, id: impl Into<String>) -> Self {
        self.required_objects.insert(id.into());
        self
    }

    pub fn validate(&self, map: &GameMap) -> anyhow::Result<()> {
        if !map.tile_size.is_finite() || map.tile_size <= 0.0 {
            anyhow::bail!("map {:?} tile size must be positive", map.id);
        }
        if map.layers.is_empty() {
            anyhow::bail!("map {:?} must contain at least one tile layer", map.id);
        }

        let collision = map
            .layers
            .iter()
            .find(|layer| layer.id == "collision")
            .ok_or_else(|| anyhow::anyhow!("map {:?} is missing collision layer", map.id))?;
        if collision.tiles.width() == 0 || collision.tiles.height() == 0 {
            anyhow::bail!("map {:?} collision layer must be non-empty", map.id);
        }
        if !collision.tiles.tile_size().is_finite() || collision.tiles.tile_size() <= 0.0 {
            anyhow::bail!("map {:?} collision tile size must be positive", map.id);
        }

        for layer in &map.layers {
            if layer.id.trim().is_empty() {
                anyhow::bail!("map {:?} has a tile layer with an empty id", map.id);
            }
            if layer.tiles.width() == 0 || layer.tiles.height() == 0 {
                anyhow::bail!("map {:?} layer '{}' must be non-empty", map.id, layer.id);
            }
            if layer.tiles.width() != collision.tiles.width()
                || layer.tiles.height() != collision.tiles.height()
            {
                anyhow::bail!(
                    "map {:?} layer '{}' dimensions differ from collision layer",
                    map.id,
                    layer.id
                );
            }
        }

        let mut object_ids = HashSet::new();
        for object in &map.objects {
            if object.id.trim().is_empty() {
                anyhow::bail!("map {:?} contains an object with an empty id", map.id);
            }
            if !object_ids.insert(object.id.clone()) {
                anyhow::bail!(
                    "map {:?} contains duplicate object id '{}'",
                    map.id,
                    object.id
                );
            }
            if !self.known_prefabs.is_empty() && !self.known_prefabs.contains(&object.prefab) {
                anyhow::bail!(
                    "map {:?} object '{}' references unknown prefab {:?}",
                    map.id,
                    object.id,
                    object.prefab
                );
            }
            if !object.position.is_finite() {
                anyhow::bail!(
                    "map {:?} object '{}' has invalid position",
                    map.id,
                    object.id
                );
            }
            let col = (object.position.x / collision.tiles.tile_size()).floor() as i32;
            let row = (object.position.y / collision.tiles.tile_size()).floor() as i32;
            if collision.tiles.is_wall(col, row) {
                anyhow::bail!(
                    "map {:?} object '{}' spawns in blocked cell ({col}, {row})",
                    map.id,
                    object.id
                );
            }
        }

        for required in &self.required_objects {
            if !object_ids.contains(required) {
                anyhow::bail!("map {:?} is missing required object '{}'", map.id, required);
            }
        }

        for region in &map.regions {
            if region.id.trim().is_empty() {
                anyhow::bail!("map {:?} contains a region with an empty id", map.id);
            }
            match region.shape {
                RegionShape::Rect { min, max } => {
                    if !min.is_finite() || !max.is_finite() || max.x <= min.x || max.y <= min.y {
                        anyhow::bail!("map {:?} region '{}' has invalid rect", map.id, region.id);
                    }
                }
                RegionShape::Circle { center, radius } => {
                    if !center.is_finite() || !radius.is_finite() || radius <= 0.0 {
                        anyhow::bail!("map {:?} region '{}' has invalid circle", map.id, region.id);
                    }
                }
            }
        }

        let _nav = NavGrid::from_tilemap(&collision.tiles);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{MapBuilder, MapValidator, PrefabId, cell};

    #[test]
    fn map_validator_accepts_current_shape_with_known_prefab_and_required_spawn() {
        let player = PrefabId(0);
        let map = MapBuilder::new("arena", 32.0)
            .tile_layer("collision", &["###", "#.#", "###"])
            .object("player_start", player, cell(1, 1))
            .finish();

        MapValidator::new()
            .allow_prefab(player)
            .require_object("player_start")
            .validate(&map)
            .unwrap();
    }

    #[test]
    fn map_validator_rejects_blocked_spawns() {
        let player = PrefabId(0);
        let map = MapBuilder::new("arena", 32.0)
            .tile_layer("collision", &["###", "###", "###"])
            .object("player_start", player, cell(1, 1))
            .finish();

        let err = MapValidator::new()
            .allow_prefab(player)
            .require_object("player_start")
            .validate(&map)
            .unwrap_err();

        assert!(err.to_string().contains("blocked cell"));
    }
}

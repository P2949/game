use std::collections::HashSet;

use crate::nav::NavGrid;
use crate::{GameMap, PrefabId, RegionShape};

pub fn validate_map(map: &GameMap) -> anyhow::Result<()> {
    MapValidator::new().validate(map)
}

/// Checks that every object in `map` references a prefab that exists in
/// `prefabs`. This lives in `game-map` (not in `game-core`'s `PrefabValidator`)
/// so the core prefab registry never needs to know about the higher-level
/// [`GameMap`] type.
pub fn validate_map_prefabs(
    map: &GameMap,
    prefabs: &game_core::builder::PrefabRegistry,
) -> anyhow::Result<()> {
    for object in &map.objects {
        if !prefabs.contains(object.prefab) {
            anyhow::bail!(
                "map {:?} object '{}' references unknown prefab {:?}",
                map.name,
                object.id,
                object.prefab
            );
        }
    }
    Ok(())
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
            anyhow::bail!("map {:?} tile size must be positive", map.name);
        }
        if map.layers.is_empty() {
            anyhow::bail!("map {:?} must contain at least one tile layer", map.name);
        }

        let collision = map
            .layers
            .iter()
            .find(|layer| layer.id == "collision")
            .ok_or_else(|| anyhow::anyhow!("map {:?} is missing collision layer", map.name))?;
        if collision.tiles.width() == 0 || collision.tiles.height() == 0 {
            anyhow::bail!("map {:?} collision layer must be non-empty", map.name);
        }
        if !collision.tiles.tile_size().is_finite() || collision.tiles.tile_size() <= 0.0 {
            anyhow::bail!("map {:?} collision tile size must be positive", map.name);
        }

        for layer in &map.layers {
            if layer.id.trim().is_empty() {
                anyhow::bail!("map {:?} has a tile layer with an empty id", map.name);
            }
            if layer.tiles.width() == 0 || layer.tiles.height() == 0 {
                anyhow::bail!("map {:?} layer '{}' must be non-empty", map.name, layer.id);
            }
            if layer.tiles.width() != collision.tiles.width()
                || layer.tiles.height() != collision.tiles.height()
            {
                anyhow::bail!(
                    "map {:?} layer '{}' dimensions differ from collision layer",
                    map.name,
                    layer.id
                );
            }
        }

        let mut object_ids = HashSet::new();
        for object in &map.objects {
            if object.id.trim().is_empty() {
                anyhow::bail!("map {:?} contains an object with an empty id", map.name);
            }
            if !object_ids.insert(object.id.clone()) {
                anyhow::bail!(
                    "map {:?} contains duplicate object id '{}'",
                    map.name,
                    object.id
                );
            }
            if !self.known_prefabs.is_empty() && !self.known_prefabs.contains(&object.prefab) {
                anyhow::bail!(
                    "map {:?} object '{}' references unknown prefab {:?}",
                    map.name,
                    object.id,
                    object.prefab
                );
            }
            if !object.position.is_finite() {
                anyhow::bail!(
                    "map {:?} object '{}' has invalid position",
                    map.name,
                    object.id
                );
            }
            let col = (object.position.x / collision.tiles.tile_size()).floor() as i32;
            let row = (object.position.y / collision.tiles.tile_size()).floor() as i32;
            if col < 0
                || row < 0
                || col as usize >= collision.tiles.width()
                || row as usize >= collision.tiles.height()
            {
                anyhow::bail!(
                    "map {:?} object '{}' spawns outside the collision map at cell ({col}, {row}); the map is {} by {} cells",
                    map.name,
                    object.id,
                    collision.tiles.width(),
                    collision.tiles.height(),
                );
            }
            if collision.tiles.is_wall(col, row) {
                if object.id == "player_start" {
                    anyhow::bail!(
                        "map {:?}: Player spawned inside a wall at cell ({col}, {row}). Move the `P` symbol or `player_start` object onto a `.` floor cell.",
                        map.name,
                    );
                }
                anyhow::bail!(
                    "map {:?} object '{}' spawns in blocked cell ({col}, {row}); move it onto a `.` floor cell",
                    map.name,
                    object.id
                );
            }
        }

        for required in &self.required_objects {
            if !object_ids.contains(required) {
                anyhow::bail!(
                    "map {:?} is missing required object '{}'",
                    map.name,
                    required
                );
            }
        }

        for region in &map.regions {
            if region.id.trim().is_empty() {
                anyhow::bail!("map {:?} contains a region with an empty id", map.name);
            }
            match region.shape {
                RegionShape::Rect { min, max } => {
                    if !min.is_finite() || !max.is_finite() || max.x <= min.x || max.y <= min.y {
                        anyhow::bail!("map {:?} region '{}' has invalid rect", map.name, region.id);
                    }
                }
                RegionShape::Circle { center, radius } => {
                    if !center.is_finite() || !radius.is_finite() || radius <= 0.0 {
                        anyhow::bail!(
                            "map {:?} region '{}' has invalid circle",
                            map.name,
                            region.id
                        );
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

        assert!(err.to_string().contains("Player spawned inside a wall"));
        assert!(err.to_string().contains("cell (1, 1)"));
    }

    #[test]
    fn map_validator_names_spawns_outside_the_collision_map() {
        let player = PrefabId(0);
        let map = MapBuilder::new("arena", 32.0)
            .tile_layer("collision", &["...", "...", "..."])
            .object("enemy_01", player, cell(3, 1))
            .finish();

        let err = MapValidator::new()
            .allow_prefab(player)
            .validate(&map)
            .unwrap_err();

        assert!(
            err.to_string()
                .contains("spawns outside the collision map at cell (3, 1)")
        );
        assert!(err.to_string().contains("3 by 3 cells"));
    }
}

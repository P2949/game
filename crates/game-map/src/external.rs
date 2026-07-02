//! Phase 13: optional external map content.
//!
//! `serde` data-transfer structs that mirror the [`MapBuilder`](crate::MapBuilder)
//! API one-to-one, plus a legacy/advanced RON loader. Files reference prefabs by name (as content
//! authors think of them); the caller supplies a resolver — typically
//! `|name| prefabs.id(name)` — so this layer never needs the numeric `PrefabId`
//! assignment to be stable across builds.

use serde::{Deserialize, Serialize};

use crate::{GameMap, MapBuilder, PrefabId, cell};

/// File-shaped mirror of [`GameMap`] for legacy/advanced RON map tests. The
/// struct is renamed to `GameMap` so a RON document reads
/// `GameMap( id: ..., layers: [...], objects: [...] )`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename = "GameMap")]
pub struct GameMapFile {
    pub id: String,
    pub tile_size: f32,
    pub layers: Vec<TileLayerFile>,
    #[serde(default)]
    pub objects: Vec<MapObjectFile>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename = "TileLayer")]
pub struct TileLayerFile {
    pub id: String,
    pub rows: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename = "MapObject")]
pub struct MapObjectFile {
    pub id: String,
    pub prefab: String,
    pub cell: (usize, usize),
}

impl GameMapFile {
    /// Builds a [`GameMap`], resolving each object's prefab name to a [`PrefabId`].
    /// Returns an error naming the offending object/prefab if a name is unknown.
    pub fn into_game_map(
        self,
        resolve: impl Fn(&str) -> Option<PrefabId>,
    ) -> anyhow::Result<GameMap> {
        let mut builder = MapBuilder::new(self.id, self.tile_size);
        for layer in self.layers {
            let rows: Vec<&str> = layer.rows.iter().map(String::as_str).collect();
            // External rows are untrusted, so reject malformed content here
            // instead of letting it be silently sanitized to floor.
            builder = builder.try_tile_layer(layer.id, &rows)?;
        }
        for object in self.objects {
            let prefab = resolve(&object.prefab).ok_or_else(|| {
                anyhow::anyhow!(
                    "map object '{}' references unknown prefab '{}'",
                    object.id,
                    object.prefab
                )
            })?;
            builder = builder.object(object.id, prefab, cell(object.cell.0, object.cell.1));
        }
        Ok(builder.finish())
    }
}

/// Parses a RON map document and resolves it into a [`GameMap`].
pub fn load_game_map_ron(
    text: &str,
    resolve: impl Fn(&str) -> Option<PrefabId>,
) -> anyhow::Result<GameMap> {
    let file: GameMapFile =
        ron::from_str(text).map_err(|err| anyhow::anyhow!("failed to parse RON map: {err}"))?;
    file.into_game_map(resolve)
}

#[cfg(test)]
mod tests {
    use crate::{PrefabId, validate_map};

    use super::load_game_map_ron;

    // The exact example from the architectural roadmap (Phase 13), loaded from a
    // real on-disk RON file to prove external content files parse end-to-end.
    const ARENA_RON: &str = include_str!("fixtures/arena_example.ron");

    fn resolver(name: &str) -> Option<PrefabId> {
        match name {
            "arena/player" => Some(PrefabId(0)),
            "arena/slime" => Some(PrefabId(1)),
            _ => None,
        }
    }

    #[test]
    fn loads_roadmap_ron_example_into_valid_map() {
        let map = load_game_map_ron(ARENA_RON, resolver).unwrap();

        assert_eq!(map.collision_tilemap().width(), 15);
        assert_eq!(map.collision_tilemap().height(), 9);
        assert_eq!(map.objects.len(), 2);
        assert_eq!(map.objects[0].id, "player_start");
        assert_eq!(map.objects[0].prefab, PrefabId(0));
        // cell (2, 4) at tile_size 32 -> center (80, 144).
        assert_eq!(map.objects[0].position, glam::vec2(80.0, 144.0));
        assert_eq!(map.objects[1].prefab, PrefabId(1));

        validate_map(&map).unwrap();
    }

    #[test]
    fn unknown_prefab_name_is_reported() {
        let ron = r#"GameMap(
    id: "x",
    tile_size: 16.0,
    layers: [ TileLayer(id: "collision", rows: ["...", "...", "..."]) ],
    objects: [ MapObject(id: "p", prefab: "missing/prefab", cell: (1, 1)) ],
)"#;

        let err = load_game_map_ron(ron, resolver).unwrap_err();
        assert!(err.to_string().contains("missing/prefab"));
    }

    #[test]
    fn malformed_ron_is_reported() {
        let err = load_game_map_ron("not ron at all", resolver).unwrap_err();
        assert!(err.to_string().contains("failed to parse RON map"));
    }

    #[test]
    fn invalid_tile_rows_are_rejected_not_sanitized() {
        // A stray glyph in an external layer must fail loading rather than be
        // silently rewritten to floor before validation can see it.
        // Rows are written so no `#` immediately follows a `"` (which would close
        // the `r#"…"#` literal early); `X` is the invalid character under test.
        let ron = r#"GameMap(
    id: "x",
    tile_size: 16.0,
    layers: [ TileLayer(id: "collision", rows: [".#", ".X"]) ],
)"#;

        let err = load_game_map_ron(ron, resolver).unwrap_err();
        assert!(err.to_string().contains("invalid tile character"));
    }
}

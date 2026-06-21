//! Minimal LDtk import for beginner collision maps and entity layers.
//!
//! The importer intentionally supports the smallest useful LDtk contract:
//! one IntGrid layer (zero is floor; non-zero is wall) and zero or more Entities
//! layers. `game-kit` maps those entity identifiers to registered prefabs.

use anyhow::{Context, Result, anyhow};
use serde::Deserialize;

use crate::{MapCell, cell};

#[derive(Clone, Debug)]
pub struct ImportedLdtkLevel {
    pub tile_size: f32,
    pub collision_rows: Vec<String>,
    pub entities: Vec<ImportedLdtkEntity>,
}

#[derive(Clone, Debug)]
pub struct ImportedLdtkEntity {
    pub identifier: String,
    pub cell: MapCell,
}

#[derive(Deserialize)]
struct ProjectFile {
    #[serde(default)]
    levels: Vec<LevelFile>,
}

#[derive(Deserialize)]
struct LevelFile {
    identifier: String,
    #[serde(rename = "layerInstances")]
    layers: Option<Vec<LayerFile>>,
}

#[derive(Deserialize)]
struct LayerFile {
    #[serde(rename = "__identifier")]
    identifier: String,
    #[serde(rename = "__type")]
    kind: String,
    #[serde(rename = "__gridSize")]
    grid_size: i32,
    #[serde(rename = "__cWid", default)]
    width: usize,
    #[serde(rename = "__cHei", default)]
    height: usize,
    #[serde(rename = "intGridCsv", default)]
    int_grid: Vec<i64>,
    #[serde(rename = "entityInstances", default)]
    entities: Vec<EntityFile>,
}

#[derive(Deserialize)]
struct EntityFile {
    #[serde(rename = "__identifier")]
    identifier: String,
    px: [i32; 2],
}

/// Parses one embedded LDtk level into collision rows plus named entity cells.
pub fn load_ldtk_level(text: &str, level_name: &str) -> Result<ImportedLdtkLevel> {
    let project: ProjectFile = serde_json::from_str(text)
        .map_err(|error| anyhow!("failed to parse LDtk project: {error}"))?;
    let level = project
        .levels
        .into_iter()
        .find(|level| level.identifier == level_name)
        .ok_or_else(|| anyhow!("LDtk project has no level '{level_name}'"))?;
    let layers = level.layers.ok_or_else(|| {
        anyhow!(
            "LDtk level '{level_name}' stores layers externally, which this importer does not support yet"
        )
    })?;
    let collision = layers
        .iter()
        .find(|layer| layer.kind == "IntGrid")
        .ok_or_else(|| anyhow!("LDtk level '{level_name}' has no IntGrid collision layer"))?;
    if collision.grid_size <= 0 {
        anyhow::bail!("LDtk level '{level_name}' has an invalid IntGrid cell size");
    }
    let expected_cells = collision
        .width
        .checked_mul(collision.height)
        .ok_or_else(|| anyhow!("LDtk level '{level_name}' collision dimensions are too large"))?;
    if collision.int_grid.len() != expected_cells {
        anyhow::bail!(
            "LDtk IntGrid layer '{}' in level '{level_name}' has {} cells; expected {expected_cells}",
            collision.identifier,
            collision.int_grid.len()
        );
    }

    let mut collision_rows = Vec::with_capacity(collision.height);
    for row in 0..collision.height {
        let start = row * collision.width;
        let end = start + collision.width;
        collision_rows.push(
            collision.int_grid[start..end]
                .iter()
                .map(|value| if *value == 0 { '.' } else { '#' })
                .collect(),
        );
    }

    let mut entities = Vec::new();
    for layer in layers.iter().filter(|layer| layer.kind == "Entities") {
        for entity in &layer.entities {
            if entity.px[0] < 0 || entity.px[1] < 0 {
                anyhow::bail!(
                    "LDtk entity '{}' in level '{level_name}' has a negative position",
                    entity.identifier
                );
            }
            let col = (entity.px[0] / collision.grid_size) as usize;
            let row = (entity.px[1] / collision.grid_size) as usize;
            if col >= collision.width || row >= collision.height {
                anyhow::bail!(
                    "LDtk entity '{}' in level '{level_name}' is outside the IntGrid collision layer",
                    entity.identifier
                );
            }
            entities.push(ImportedLdtkEntity {
                identifier: entity.identifier.clone(),
                cell: cell(col, row),
            });
        }
    }

    Ok(ImportedLdtkLevel {
        tile_size: collision.grid_size as f32,
        collision_rows,
        entities,
    })
}

/// Reads and parses one LDtk project file.
pub fn load_ldtk_level_file(
    path: impl AsRef<std::path::Path>,
    level_name: &str,
) -> Result<ImportedLdtkLevel> {
    let path = path.as_ref();
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read LDtk project '{}'", path.display()))?;
    load_ldtk_level(&text, level_name)
}

#[cfg(test)]
mod tests {
    use super::load_ldtk_level;

    const PROJECT: &str = r#"{
        "levels": [{
            "identifier": "Level_1",
            "layerInstances": [
                {
                    "__identifier": "Collision",
                    "__type": "IntGrid",
                    "__gridSize": 16,
                    "__cWid": 3,
                    "__cHei": 2,
                    "intGridCsv": [1, 0, 1, 1, 0, 1]
                },
                {
                    "__identifier": "Entities",
                    "__type": "Entities",
                    "__gridSize": 16,
                    "entityInstances": [
                        { "__identifier": "PlayerStart", "px": [16, 0] },
                        { "__identifier": "Slime", "px": [16, 16] }
                    ]
                }
            ]
        }]
    }"#;

    #[test]
    fn imports_intgrid_walls_and_entity_cells() {
        let level = load_ldtk_level(PROJECT, "Level_1").unwrap();
        assert_eq!(level.tile_size, 16.0);
        assert_eq!(level.collision_rows, ["#.#", "#.#"]);
        assert_eq!(level.entities.len(), 2);
        assert_eq!(level.entities[0].identifier, "PlayerStart");
        assert_eq!(level.entities[0].cell.col(), 1);
        assert_eq!(level.entities[1].cell.row(), 1);
    }

    #[test]
    fn reports_a_missing_level_by_name() {
        let error = load_ldtk_level(PROJECT, "Missing").unwrap_err().to_string();
        assert!(error.contains("no level 'Missing'"));
    }
}

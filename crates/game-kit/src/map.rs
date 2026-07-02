//! Map authoring (Phase 6) and content/runtime map services (Phase 7).
//!
//! [`MapAuthor`] declares a map in code or loads one from legacy/advanced RON,
//! referring to prefabs by **name** (matching the RON format) instead of `PrefabId`. On
//! finalization the facade resolves names, validates the map, registers its
//! collision tilemap + theme for rendering, and records the full [`GameMap`] (with
//! objects) into a [`ContentRuntime`] resource so startup/reset systems can spawn
//! map objects without capturing a cloned map or `Rc<PrefabRegistry>`.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

use anyhow::{Context, Result, anyhow};
use game_core::app::MapData;
use game_core::builder::{MapId, PrefabId, PrefabRegistry, PropertyBag};
use game_core::commands::{CommandErrorKind, CommandErrors, CommandQueue};
use game_core::nav::NavGrid;
use game_core::world::World;
use game_core::world::{Sprite, Transform};
use game_map::{
    GameMap, MapBuilder, MapCell, MapValidator, cell, load_game_map_ron, load_ldtk_level_file,
    load_tiled_map_file, validate_map_prefabs,
};

use crate::app::GameApp;
use crate::assets::TextureRef;
use crate::beginner::actors::{Door, Enemy, Player, TriggerArea};
use crate::bundle::vec2s;
use crate::diagnostics::bad_map_symbol_error;
use crate::prefab::IntoContentName;

/// Tile floor/wall sprites for a map. Re-exported from the engine so content can
/// build a theme with the prelude's `Sprite` type: `TileTheme { floor, wall }`.
pub use game_core::app::TileTheme;

/// Where a pending map's tiles/objects come from.
enum MapSource {
    InCode {
        tile_size: f32,
        rows: Vec<String>,
        objects: Vec<(String, String, MapCell)>,
        legends: Vec<(char, String)>,
    },
    Text {
        path: String,
        tile_size: f32,
        objects: Vec<(String, String, MapCell)>,
        legends: Vec<(char, String)>,
    },
    Ron {
        text: String,
    },
    Ldtk {
        path: String,
        level: Option<String>,
        entities: Vec<(String, String)>,
    },
    Tiled {
        path: String,
        objects: Vec<(String, String)>,
    },
}

/// A map declared through [`MapAuthor`] but not yet resolved/validated; finalized
/// in [`GameApp::finish`].
pub(crate) struct PendingMap {
    name: String,
    source: MapSource,
    required_objects: Vec<String>,
    theme: Option<TileTheme>,
    start: bool,
    misuse_errors: Vec<String>,
}

/// The information needed to rebuild one text map without re-running Rust
/// content setup. Stored only for maps created with `map_from_text*`.
#[derive(Clone)]
pub(crate) struct TextMapReloadSource {
    path: String,
    tile_size: f32,
    objects: Vec<(String, String, MapCell)>,
    legends: Vec<(char, String)>,
    required_objects: Vec<String>,
    theme: TileTheme,
}

impl PendingMap {
    /// Resolves prefab names, builds and validates the [`GameMap`], and returns it
    /// alongside its theme and start flag for registration.
    pub(crate) fn resolve(
        self,
        prefabs: &PrefabRegistry,
    ) -> Result<(GameMap, TileTheme, bool, Option<TextMapReloadSource>)> {
        if !self.misuse_errors.is_empty() {
            anyhow::bail!(
                "map '{}' has invalid authoring calls:\n{}",
                self.name,
                self.misuse_errors.join("\n")
            );
        }

        let theme = self
            .theme
            .ok_or_else(|| {
                anyhow!(
                    "Map '{}' has no tile theme.\n\nAdd:\n    .theme(TileTheme {{\n        floor: Sprite::new(assets.floor, vec2s(32.0)),\n        wall: Sprite::new(assets.wall, vec2s(32.0)),\n    }})\n\nOr use the beginner helper:\n    .simple_theme(assets.floor, assets.wall)",
                    self.name
                )
            })?;
        let reload_source = match &self.source {
            MapSource::Text {
                path,
                tile_size,
                objects,
                legends,
            } => Some(TextMapReloadSource {
                path: path.clone(),
                tile_size: *tile_size,
                objects: objects.clone(),
                legends: legends.clone(),
                required_objects: self.required_objects.clone(),
                theme,
            }),
            MapSource::InCode { .. }
            | MapSource::Ron { .. }
            | MapSource::Ldtk { .. }
            | MapSource::Tiled { .. } => None,
        };

        let game_map = match self.source {
            MapSource::InCode {
                tile_size,
                rows,
                mut objects,
                legends,
            } => {
                validate_map_row_widths(&self.name, &rows)?;
                let rows = expand_symbolic_tiles(&self.name, rows, &mut objects, legends)?;
                let row_refs: Vec<&str> = rows.iter().map(String::as_str).collect();
                let mut builder = MapBuilder::new(self.name.clone(), tile_size)
                    .try_tile_layer("collision", &row_refs)
                    .with_context(|| format!("map '{}' has invalid tiles", self.name))?;
                for (id, prefab_name, cell) in objects {
                    let prefab = prefabs.id(&prefab_name).ok_or_else(|| {
                        anyhow!(
                            "map '{}' object '{}' references unknown prefab '{}'",
                            self.name,
                            id,
                            prefab_name
                        )
                    })?;
                    builder = builder.object(id, prefab, cell);
                }
                builder.finish()
            }
            MapSource::Text {
                path,
                tile_size,
                objects,
                legends,
            } => load_text_game_map(&self.name, &path, tile_size, objects, legends, prefabs)?,
            MapSource::Ron { text } => load_game_map_ron(&text, |name| prefabs.id(name))
                .with_context(|| format!("map '{}' failed to load from RON", self.name))?,
            MapSource::Ldtk {
                path,
                level,
                entities,
            } => load_ldtk_game_map(&self.name, &path, level, entities, prefabs)?,
            MapSource::Tiled { path, objects } => {
                load_tiled_game_map(&self.name, &path, objects, prefabs)?
            }
        };

        validate_game_map(&game_map, &self.required_objects, prefabs)?;

        Ok((game_map, theme, self.start, reload_source))
    }
}

fn load_ldtk_game_map(
    map_name: &str,
    path: &str,
    level: Option<String>,
    entities: Vec<(String, String)>,
    prefabs: &PrefabRegistry,
) -> Result<GameMap> {
    let level = level.ok_or_else(|| {
        anyhow!("LDtk map '{map_name}' needs a level. Add:\n    .level(\"Level_1\")")
    })?;
    let full_path = beginner_asset_path(path);
    let imported = load_ldtk_level_file(&full_path, &level)
        .with_context(|| format!("map '{map_name}' failed to load LDtk level '{level}'"))?;
    let mut entity_lookup = HashMap::new();
    for (identifier, prefab) in entities {
        if entity_lookup.insert(identifier.clone(), prefab).is_some() {
            anyhow::bail!("LDtk map '{map_name}' has duplicate mapping for entity '{identifier}'");
        }
    }

    let row_refs = imported
        .collision_rows
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let mut builder = MapBuilder::new(map_name, imported.tile_size)
        .try_tile_layer("collision", &row_refs)
        .with_context(|| format!("LDtk map '{map_name}' has invalid collision cells"))?;
    let mut counts: HashMap<String, usize> = HashMap::new();
    for entity in imported.entities {
        let prefab_name = entity_lookup.get(&entity.identifier).ok_or_else(|| {
            anyhow!(
                "LDtk map '{map_name}' has entity '{}' with no prefab mapping.\n\nAdd:\n    .entity(\"{}\", \"some_prefab\")",
                entity.identifier,
                entity.identifier
            )
        })?;
        let prefab = prefabs.id(prefab_name).ok_or_else(|| {
            anyhow!(
                "LDtk entity '{}' in map '{map_name}' maps to unknown prefab '{prefab_name}'",
                entity.identifier
            )
        })?;
        let count = counts.entry(prefab_name.clone()).or_default();
        *count += 1;
        builder = builder.object(
            generated_object_id(prefab_name, *count),
            prefab,
            entity.cell,
        );
    }
    Ok(builder.finish())
}

fn load_text_game_map(
    map_name: &str,
    path: &str,
    tile_size: f32,
    mut objects: Vec<(String, String, MapCell)>,
    legends: Vec<(char, String)>,
    prefabs: &PrefabRegistry,
) -> Result<GameMap> {
    let full_path = beginner_asset_path(path);
    let text = fs::read_to_string(&full_path).with_context(|| {
        format!(
            "map '{map_name}' could not read text map '{}'. Place it under assets/",
            full_path.display()
        )
    })?;
    let rows = text
        .lines()
        .map(|line| line.trim_end_matches('\r').to_owned())
        .collect::<Vec<_>>();
    validate_map_row_widths(map_name, &rows)?;
    let rows = expand_symbolic_tiles(map_name, rows, &mut objects, legends)?;
    let row_refs = rows.iter().map(String::as_str).collect::<Vec<_>>();
    let mut builder = MapBuilder::new(map_name, tile_size)
        .try_tile_layer("collision", &row_refs)
        .with_context(|| format!("map '{map_name}' has invalid text-map tiles"))?;
    for (id, prefab_name, cell) in objects {
        let prefab = prefabs.id(&prefab_name).ok_or_else(|| {
            anyhow!("map '{map_name}' object '{id}' references unknown prefab '{prefab_name}'")
        })?;
        builder = builder.object(id, prefab, cell);
    }
    Ok(builder.finish())
}

fn load_tiled_game_map(
    map_name: &str,
    path: &str,
    objects: Vec<(String, String)>,
    prefabs: &PrefabRegistry,
) -> Result<GameMap> {
    let full_path = beginner_asset_path(path);
    let imported = load_tiled_map_file(&full_path)
        .with_context(|| format!("map '{map_name}' failed to load Tiled TMX map"))?;
    let mut object_lookup = HashMap::new();
    for (identifier, prefab) in objects {
        if object_lookup.insert(identifier.clone(), prefab).is_some() {
            anyhow::bail!("Tiled map '{map_name}' has duplicate mapping for object '{identifier}'");
        }
    }

    let row_refs = imported
        .collision_rows
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let mut builder = MapBuilder::new(map_name, imported.tile_size)
        .try_tile_layer("collision", &row_refs)
        .with_context(|| format!("Tiled map '{map_name}' has invalid collision cells"))?;
    let mut counts: HashMap<String, usize> = HashMap::new();
    for object in imported.objects {
        let prefab_name = object_lookup.get(&object.identifier).ok_or_else(|| {
            anyhow!(
                "Tiled map '{map_name}' has object '{}' with no prefab mapping.\n\nAdd:\n    .object(\"{}\", \"some_prefab\")",
                object.identifier,
                object.identifier
            )
        })?;
        let prefab = prefabs.id(prefab_name).ok_or_else(|| {
            anyhow!(
                "Tiled object '{}' in map '{map_name}' maps to unknown prefab '{prefab_name}'",
                object.identifier
            )
        })?;
        let count = counts.entry(prefab_name.clone()).or_default();
        *count += 1;
        builder = builder.object(
            generated_object_id(prefab_name, *count),
            prefab,
            object.cell,
        );
    }
    Ok(builder.finish())
}

fn validate_game_map(
    game_map: &GameMap,
    required_objects: &[String],
    prefabs: &PrefabRegistry,
) -> Result<()> {
    let mut validator = MapValidator::new();
    for required in required_objects {
        validator = validator.require_object(required);
    }
    validator
        .validate(game_map)
        .with_context(|| format!("map '{}' validation failed", game_map.name))?;
    validate_map_prefabs(game_map, prefabs)
        .with_context(|| format!("map '{}' references unknown prefab", game_map.name))?;
    Ok(())
}

pub(crate) fn beginner_asset_path(path: &str) -> PathBuf {
    crate::paths::beginner_asset_file(path)
}

/// Builder for one map. Created by [`GameApp::map`] (in-code) or
/// [`GameApp::map_from_ron`] (legacy/advanced external content). Finalized lazily by
/// [`Self::start`].
pub struct MapAuthor<'a, 'app> {
    app: &'a mut GameApp<'app>,
    pending: PendingMap,
}

impl<'a, 'app> MapAuthor<'a, 'app> {
    pub(crate) fn in_code(app: &'a mut GameApp<'app>, name: String) -> Self {
        Self {
            app,
            pending: PendingMap {
                name,
                source: MapSource::InCode {
                    tile_size: 32.0,
                    rows: Vec::new(),
                    objects: Vec::new(),
                    legends: Vec::new(),
                },
                required_objects: Vec::new(),
                theme: None,
                start: false,
                misuse_errors: Vec::new(),
            },
        }
    }

    pub(crate) fn from_ron(app: &'a mut GameApp<'app>, text: String) -> Self {
        Self {
            app,
            pending: PendingMap {
                name: "<ron>".to_owned(),
                source: MapSource::Ron { text },
                required_objects: Vec::new(),
                theme: None,
                start: false,
                misuse_errors: Vec::new(),
            },
        }
    }

    pub(crate) fn from_text(app: &'a mut GameApp<'app>, name: String, path: String) -> Self {
        Self {
            app,
            pending: PendingMap {
                name,
                source: MapSource::Text {
                    path,
                    tile_size: 32.0,
                    objects: Vec::new(),
                    legends: Vec::new(),
                },
                required_objects: Vec::new(),
                theme: None,
                start: false,
                misuse_errors: Vec::new(),
            },
        }
    }

    pub(crate) fn from_ldtk(app: &'a mut GameApp<'app>, name: String, path: String) -> Self {
        Self {
            app,
            pending: PendingMap {
                name,
                source: MapSource::Ldtk {
                    path,
                    level: None,
                    entities: Vec::new(),
                },
                required_objects: Vec::new(),
                theme: None,
                start: false,
                misuse_errors: Vec::new(),
            },
        }
    }

    pub(crate) fn from_tiled(app: &'a mut GameApp<'app>, name: String, path: String) -> Self {
        Self {
            app,
            pending: PendingMap {
                name,
                source: MapSource::Tiled {
                    path,
                    objects: Vec::new(),
                },
                required_objects: Vec::new(),
                theme: None,
                start: false,
                misuse_errors: Vec::new(),
            },
        }
    }

    /// Sets the tile size in world units (in-code maps only; RON maps carry it).
    pub fn tile_size(mut self, tile_size: f32) -> Self {
        match &mut self.pending.source {
            MapSource::InCode { tile_size: t, .. } | MapSource::Text { tile_size: t, .. } => {
                *t = tile_size;
            }
            MapSource::Ron { .. } => {
                self.pending
                    .misuse_errors
                    .push("tile_size() is only valid on in-code maps, not RON maps".to_owned());
            }
            MapSource::Ldtk { .. } | MapSource::Tiled { .. } => {
                self.pending
                    .misuse_errors
                    .push("tile_size() is only valid on in-code or text maps".to_owned());
            }
        }
        self
    }

    /// Sets the collision layer from rows of `.` (floor) / `#` (wall) (in-code
    /// maps only).
    pub fn tiles<const N: usize>(mut self, rows: [&str; N]) -> Self {
        match &mut self.pending.source {
            MapSource::InCode { rows: r, .. } => {
                *r = rows.iter().map(|row| (*row).to_owned()).collect();
            }
            MapSource::Text { .. }
            | MapSource::Ron { .. }
            | MapSource::Ldtk { .. }
            | MapSource::Tiled { .. } => {
                self.pending
                    .misuse_errors
                    .push("tiles() is only valid on in-code maps; text maps read their rows from the file".to_owned());
            }
        }
        self
    }

    /// Adds a map object spawning `prefab` (by name) at `cell` (in-code maps only;
    /// RON maps declare objects in the file).
    pub fn spawn(
        mut self,
        id: impl IntoContentName,
        prefab: impl IntoContentName,
        cell: MapCell,
    ) -> Self {
        match &mut self.pending.source {
            MapSource::InCode { objects, .. } | MapSource::Text { objects, .. } => {
                objects.push((id.into_content_name(), prefab.into_content_name(), cell));
            }
            MapSource::Ron { .. } => {
                self.pending
                    .misuse_errors
                    .push("spawn() is only valid on in-code maps, not RON maps".to_owned());
            }
            MapSource::Ldtk { .. } | MapSource::Tiled { .. } => {
                self.pending
                    .misuse_errors
                    .push("spawn() is only valid on in-code or text maps".to_owned());
            }
        }
        self
    }

    /// Spawns a prefab wherever `symbol` appears in the tile rows. Symbols are
    /// treated as floor for collision. For example, `P` can mark the player start:
    /// `.tiles(["#P#"]).legend('P', "player")`.
    pub fn legend(mut self, symbol: char, prefab: impl IntoContentName) -> Self {
        match &mut self.pending.source {
            MapSource::InCode { legends, .. } | MapSource::Text { legends, .. } => {
                legends.push((symbol, prefab.into_content_name()));
            }
            MapSource::Ron { .. } => {
                self.pending
                    .misuse_errors
                    .push("legend() is only valid on in-code maps, not RON maps".to_owned());
            }
            MapSource::Ldtk { .. } | MapSource::Tiled { .. } => {
                self.pending
                    .misuse_errors
                    .push("legend() is only valid on in-code or text maps".to_owned());
            }
        }
        self
    }

    /// Sets the map's floor/wall theme.
    pub fn theme(mut self, theme: TileTheme) -> Self {
        self.pending.theme = Some(theme);
        self
    }

    /// Sets a simple floor/wall theme from texture handles, sizing each tile
    /// sprite to the map's current tile size. Each texture can instead be a
    /// registered asset key such as `"floor"` or `"wall"`.
    pub fn theme_from_textures(
        self,
        floor: impl Into<TextureRef>,
        wall: impl Into<TextureRef>,
    ) -> Self {
        self.simple_theme(floor, wall)
    }

    /// Beginner alias for [`Self::theme_from_textures`].
    pub fn simple_theme(
        mut self,
        floor: impl Into<TextureRef>,
        wall: impl Into<TextureRef>,
    ) -> Self {
        let floor = self.app.resolve_texture_ref(floor.into());
        let wall = self.app.resolve_texture_ref(wall.into());
        let tile_size = match &self.pending.source {
            MapSource::InCode { tile_size, .. } | MapSource::Text { tile_size, .. } => *tile_size,
            MapSource::Ron { .. } | MapSource::Ldtk { .. } | MapSource::Tiled { .. } => 32.0,
        };
        match (floor, wall) {
            (Ok(floor), Ok(wall)) => {
                self.pending.theme = Some(TileTheme {
                    floor: Sprite::new(floor, vec2s(tile_size)),
                    wall: Sprite::new(wall, vec2s(tile_size)),
                });
            }
            (floor, wall) => {
                self.pending.theme = None;
                self.pending.misuse_errors.extend(
                    floor
                        .err()
                        .into_iter()
                        .chain(wall.err())
                        .map(|error| error.to_string()),
                );
            }
        }
        self
    }

    /// Selects the level inside an LDtk project. Only valid after
    /// [`GameApp::map_from_ldtk`].
    pub fn level(mut self, level: impl Into<String>) -> Self {
        match &mut self.pending.source {
            MapSource::Ldtk {
                level: selected, ..
            } => *selected = Some(level.into()),
            _ => self
                .pending
                .misuse_errors
                .push("level() is only valid on LDtk maps".to_owned()),
        }
        self
    }

    /// Maps an LDtk entity identifier, such as `"PlayerStart"`, to a prefab
    /// name such as `"player"`.
    pub fn entity(mut self, identifier: impl Into<String>, prefab: impl Into<String>) -> Self {
        match &mut self.pending.source {
            MapSource::Ldtk { entities, .. } => entities.push((identifier.into(), prefab.into())),
            _ => self
                .pending
                .misuse_errors
                .push("entity() is only valid on LDtk maps".to_owned()),
        }
        self
    }

    /// Maps a Tiled object `class`, `type`, or `name` to a prefab. Only valid
    /// after [`GameApp::map_from_tiled`].
    pub fn object(mut self, identifier: impl Into<String>, prefab: impl Into<String>) -> Self {
        match &mut self.pending.source {
            MapSource::Tiled { objects, .. } => objects.push((identifier.into(), prefab.into())),
            _ => self
                .pending
                .misuse_errors
                .push("object() is only valid on Tiled maps".to_owned()),
        }
        self
    }

    /// Requires that the finalized map contains an object with this id (e.g.
    /// `"player_start"`), failing validation otherwise.
    pub fn require_object(mut self, id: impl Into<String>) -> Self {
        self.pending.required_objects.push(id.into());
        self
    }

    /// Declares this as the game's start map and records it for finalization.
    pub fn start(mut self) {
        self.pending.start = true;
        self.app.push_pending_map(self.pending);
    }

    /// Records this map without making it the start map.
    pub fn finish(self) {
        self.app.push_pending_map(self.pending);
    }
}

fn expand_symbolic_tiles(
    map_name: &str,
    rows: Vec<String>,
    objects: &mut Vec<(String, String, MapCell)>,
    legends: Vec<(char, String)>,
) -> Result<Vec<String>> {
    let mut legend_lookup = HashMap::new();
    for (symbol, prefab) in legends {
        if legend_lookup.insert(symbol, prefab).is_some() {
            anyhow::bail!("map '{map_name}' has duplicate legend for symbol {symbol:?}");
        }
    }

    let mut counts: HashMap<String, usize> = HashMap::new();
    let mut symbol_counts: HashMap<char, usize> = HashMap::new();
    let mut collision_rows = Vec::with_capacity(rows.len());
    for (row, line) in rows.into_iter().enumerate() {
        let mut collision_row = String::with_capacity(line.len());
        for (col, symbol) in line.chars().enumerate() {
            match symbol {
                '#' | '.' => collision_row.push(symbol),
                symbol => {
                    let prefab = legend_lookup.get(&symbol).ok_or_else(|| {
                        let known_symbols = legend_lookup.keys().copied().collect::<Vec<_>>();
                        bad_map_symbol_error(map_name, symbol, row, col, &known_symbols)
                    })?;
                    collision_row.push('.');
                    *symbol_counts.entry(symbol).or_default() += 1;
                    let count = counts.entry(prefab.clone()).or_default();
                    *count += 1;
                    objects.push((
                        generated_object_id(prefab, *count),
                        prefab.clone(),
                        cell(col, row),
                    ));
                }
            }
        }
        collision_rows.push(collision_row);
    }

    if legend_lookup
        .get(&'P')
        .is_some_and(|prefab| prefab == "player")
        && symbol_counts.get(&'P').copied().unwrap_or_default() == 0
    {
        anyhow::bail!(
            "Map '{map_name}' has no player spawn.\n\nAdd 'P' to the tile map and:\n    .legend('P', \"player\")"
        );
    }

    Ok(collision_rows)
}

fn validate_map_row_widths(map_name: &str, rows: &[String]) -> Result<()> {
    let Some(first) = rows.first() else {
        anyhow::bail!("Map '{map_name}' has no rows. Add at least one tile row.");
    };
    let expected_width = first.chars().count();
    for (index, row) in rows.iter().enumerate().skip(1) {
        let width = row.chars().count();
        if width != expected_width {
            anyhow::bail!(
                "Map rows have inconsistent widths in map '{map_name}'.\n\nRow 1 has width {expected_width}; row {} has width {width}.\n\nMake every row the same width.",
                index + 1,
            );
        }
    }
    Ok(())
}

fn generated_object_id(prefab: &str, count: usize) -> String {
    let mut base: String = prefab
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect();
    if base.is_empty() {
        base = "object".to_owned();
    }
    if base == "player" && count == 1 {
        "player_start".to_owned()
    } else {
        format!("{base}_{count:02}")
    }
}

/// Runtime content resource (Phase 7): the prefab registry and full maps (with
/// objects) needed to spawn/reset map objects. Inserted into the `World` by the
/// facade's built-in startup system and read by [`crate::context::GameCtx`] /
/// [`crate::context::StartupGameCtx`].
pub struct ContentRuntime {
    prefabs: Rc<PrefabRegistry>,
    maps: HashMap<String, GameMap>,
    map_ids: HashMap<String, MapId>,
    themes: HashMap<String, TileTheme>,
    text_maps: HashMap<String, TextMapReloadSource>,
    start_map: String,
    current_map: String,
}

impl ContentRuntime {
    pub(crate) fn new(
        prefabs: Rc<PrefabRegistry>,
        maps: HashMap<String, GameMap>,
        map_ids: HashMap<String, MapId>,
        themes: HashMap<String, TileTheme>,
        text_maps: HashMap<String, TextMapReloadSource>,
        start_map: String,
    ) -> Self {
        Self {
            prefabs,
            map_ids,
            themes,
            text_maps,
            current_map: start_map.clone(),
            start_map,
            maps,
        }
    }

    /// The author name of the map currently spawned.
    pub fn current_map_name(&self) -> &str {
        &self.current_map
    }

    pub fn prefab_id(&self, name: &str) -> Option<PrefabId> {
        self.prefabs.id(name)
    }

    pub(crate) fn spawn_prefab_by_name(
        &self,
        name: &str,
        world: &mut World,
        position: glam::Vec2,
        properties: &PropertyBag,
    ) -> Result<game_core::world::EntityId> {
        let prefab = self
            .prefab_id(name)
            .ok_or_else(|| anyhow!("unknown prefab '{name}'"))?;
        self.prefabs.spawn(prefab, world, position, properties)
    }

    pub fn map_id(&self, name: &str) -> Option<MapId> {
        self.map_ids.get(name).copied()
    }

    pub fn current_map_id(&self) -> Option<MapId> {
        self.map_id(&self.current_map)
    }

    pub fn start_map_id(&self) -> Option<MapId> {
        self.map_id(&self.start_map)
    }

    pub(crate) fn map_data(&self, name: &str) -> Result<(MapId, MapData)> {
        let map = self
            .maps
            .get(name)
            .ok_or_else(|| anyhow!("unknown map '{name}'"))?;
        let theme = *self
            .themes
            .get(name)
            .ok_or_else(|| anyhow!("map '{name}' has no registered theme"))?;
        let tilemap = map.collision_tilemap();
        let map_id = self
            .map_id(name)
            .ok_or_else(|| anyhow!("map '{name}' is not registered"))?;
        Ok((
            map_id,
            MapData {
                nav: NavGrid::from_tilemap(&tilemap),
                tilemap,
                theme,
            },
        ))
    }

    /// Rebuilds the active text map from its original source path. Maps declared
    /// in code or loaded from RON deliberately remain non-reloadable.
    pub(crate) fn reload_current_text_map(&mut self) -> Result<(MapId, MapData)> {
        let name = self.current_map.clone();
        let source = self.text_maps.get(&name).cloned().ok_or_else(|| {
            anyhow!("map '{name}' was not loaded from a text file and cannot be reloaded")
        })?;
        let map = load_text_game_map(
            &name,
            &source.path,
            source.tile_size,
            source.objects,
            source.legends,
            &self.prefabs,
        )?;
        validate_game_map(&map, &source.required_objects, &self.prefabs)?;
        let tilemap = map.collision_tilemap();
        let map_id = self
            .map_id(&name)
            .ok_or_else(|| anyhow!("map '{name}' is not registered"))?;
        self.themes.insert(name.clone(), source.theme);
        self.maps.insert(name, map);
        Ok((
            map_id,
            MapData {
                nav: NavGrid::from_tilemap(&tilemap),
                tilemap,
                theme: source.theme,
            },
        ))
    }

    fn map_by_name(&self, name: &str) -> Result<&GameMap> {
        self.maps
            .get(name)
            .ok_or_else(|| anyhow!("unknown map '{name}'"))
    }

    fn preflight_spawn_map(&self, map_name: &str) -> Result<()> {
        let map = self.map_by_name(map_name)?;
        validate_map_prefabs(map, &self.prefabs)
            .with_context(|| format!("map '{map_name}' references unknown prefab"))?;
        let mut scratch = World::new();
        spawn_map_objects(&mut scratch, map, &self.prefabs)
    }

    fn spawn_map_by_name(&self, world: &mut World, map_name: &str) -> Result<()> {
        let map = self.map_by_name(map_name)?;
        spawn_map_objects(world, map, &self.prefabs)
    }
}

/// Spawns every object of `map` through `prefabs` (Phase 7.2). Shared by startup
/// and reset, so neither content crate carries its own copy.
pub fn spawn_map_objects(world: &mut World, map: &GameMap, prefabs: &PrefabRegistry) -> Result<()> {
    for object in &map.objects {
        let entity = prefabs.spawn(object.prefab, world, object.position, &object.properties)?;
        validate_spawned_collision_components(world, entity, &map.name, &object.id)?;
    }
    Ok(())
}

/// Validates collision-bearing beginner roles immediately after their prefabs
/// spawn. Map validation catches bad cells before startup; this catches malformed
/// custom prefabs that claim a gameplay role but omit or invalidate the runtime
/// collider needed by that role.
fn validate_spawned_collision_components(
    world: &World,
    entity: game_core::world::EntityId,
    map_name: &str,
    object_id: &str,
) -> Result<()> {
    let role = if world.has::<Player>(entity) {
        Some("player")
    } else if world.has::<Enemy>(entity) {
        Some("enemy")
    } else if world.has::<Door>(entity) {
        Some("door")
    } else if world.has::<TriggerArea>(entity) {
        Some("trigger area")
    } else {
        None
    };
    let Some(role) = role else {
        return Ok(());
    };

    let collider = world.get::<game_physics::Collider>(entity).ok_or_else(|| {
        anyhow!(
            "map '{map_name}' object '{object_id}' spawned a {role} without a collider. Add a positive `.collider(...)` when defining that prefab."
        )
    })?;
    let size = collider.half_extents * 2.0;
    if !size.is_finite() || size.x <= 0.0 || size.y <= 0.0 {
        anyhow::bail!(
            "map '{map_name}' object '{object_id}' spawned a {role} with a zero-size or invalid collider ({}, {}). Give the prefab a positive `.collider(...)` size.",
            size.x,
            size.y,
        );
    }

    if world.get::<Transform>(entity).is_none() {
        anyhow::bail!(
            "map '{map_name}' object '{object_id}' spawned a {role} without a transform; map objects need a position."
        );
    }
    Ok(())
}

fn clear_world_for_map_respawn(world: &mut World) {
    world.clear();
    // `World::clear` preserves resources, so drop any commands queued against the
    // pre-reset world before respawning.
    if let Some(commands) = world.get_resource_mut::<CommandQueue>() {
        commands.clear();
    }
}

fn switch_world_to_map(world: &mut World, map_name: String) -> Result<MapId> {
    let mut content = world
        .remove_resource::<ContentRuntime>()
        .ok_or_else(|| anyhow!("content runtime missing; was the game-kit plugin used?"))?;
    let previous_map = content.current_map.clone();
    let Some(map_id) = content.map_id(&map_name) else {
        let message = format!("unknown map '{map_name}'");
        world.insert_resource(content);
        record_map_transition_error(world, message.clone());
        anyhow::bail!(message);
    };
    if let Err(error) = content.preflight_spawn_map(&map_name) {
        let message = format!("failed to switch to map '{map_name}': {error}");
        content.current_map = previous_map;
        world.insert_resource(content);
        record_map_transition_error(world, message.clone());
        anyhow::bail!(message);
    }
    clear_world_for_map_respawn(world);
    let result = content.spawn_map_by_name(world, &map_name).map(|()| map_id);
    if result.is_ok() {
        content.current_map = map_name.clone();
    } else if let Err(error) = &result {
        let mut message = format!("failed to switch to map '{map_name}': {error}");
        content.current_map = previous_map.clone();
        clear_world_for_map_respawn(world);
        if let Err(rollback) = content.spawn_map_by_name(world, &previous_map) {
            message.push_str(&format!(
                "; also failed to restore previous map '{previous_map}': {rollback}"
            ));
        }
        record_map_transition_error(world, message);
    }
    world.insert_resource(content);
    result
}

fn record_map_transition_error(world: &mut World, message: impl Into<String>) {
    world
        .resource_or_insert_with(CommandErrors::default)
        .push(CommandErrorKind::MapTransition, message);
}

pub(crate) fn change_to_map_world(world: &mut World, map_name: &str) -> Result<MapId> {
    switch_world_to_map(world, map_name.to_owned())
}

pub(crate) fn restart_current_map_world(world: &mut World) -> Result<MapId> {
    let map_name = world
        .get_resource::<ContentRuntime>()
        .ok_or_else(|| anyhow!("content runtime missing; was the game-kit plugin used?"))?
        .current_map_name()
        .to_owned();
    switch_world_to_map(world, map_name)
}

pub(crate) fn restart_start_map_world(world: &mut World) -> Result<MapId> {
    let map_name = world
        .get_resource::<ContentRuntime>()
        .ok_or_else(|| anyhow!("content runtime missing; was the game-kit plugin used?"))?
        .start_map
        .clone();
    switch_world_to_map(world, map_name)
}

/// Clears the world and (re)spawns the start map's objects. Used by both startup
/// and reset, so the behavior is generic and identical.
pub(crate) fn reset_to_start_map_world(world: &mut World) -> Result<()> {
    restart_start_map_world(world).map(|_| ())
}

#[cfg(test)]
mod tests {
    use game_core::world::{Entity, World};

    use super::{
        expand_symbolic_tiles, validate_map_row_widths, validate_spawned_collision_components,
    };
    use crate::beginner::actors::{Door, TriggerArea};

    #[test]
    fn text_map_diagnostics_name_unknown_symbols_and_their_fix() {
        let error =
            expand_symbolic_tiles("level_1", vec!["#X#".to_owned()], &mut Vec::new(), vec![])
                .unwrap_err()
                .to_string();
        assert!(error.contains("Map 'level_1' uses symbol 'X'"));
        assert!(error.contains(".legend('X', \"some_prefab\")"));
    }

    #[test]
    fn text_map_diagnostics_report_ragged_row_widths() {
        let error = validate_map_row_widths(
            "level_1",
            &["############".to_owned(), "##########".to_owned()],
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("Map rows have inconsistent widths"));
        assert!(error.contains("Row 1 has width 12; row 2 has width 10"));
    }

    #[test]
    fn text_map_diagnostics_require_a_player_symbol_when_legend_declares_player() {
        let error = expand_symbolic_tiles(
            "level_1",
            vec!["...".to_owned()],
            &mut Vec::new(),
            vec![('P', "player".to_owned())],
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("Map 'level_1' has no player spawn"));
        assert!(error.contains("Add 'P'"));
    }

    #[test]
    fn spawned_door_diagnostics_explain_missing_collider() {
        let mut world = World::new();
        let door = world.spawn(Entity::new(glam::Vec2::ZERO).with(Door));

        let error = validate_spawned_collision_components(&world, door, "level_1", "exit")
            .unwrap_err()
            .to_string();

        assert!(error.contains("spawned a door without a collider"));
        assert!(error.contains(".collider(...)"));
    }

    #[test]
    fn spawned_trigger_diagnostics_reject_zero_size_colliders() {
        let mut world = World::new();
        let trigger = world.spawn(
            Entity::new(glam::Vec2::ZERO)
                .with(TriggerArea)
                .with(game_physics::Collider::box_of(glam::Vec2::ZERO)),
        );

        let error = validate_spawned_collision_components(&world, trigger, "level_1", "goal")
            .unwrap_err()
            .to_string();

        assert!(error.contains("spawned a trigger area with a zero-size or invalid collider"));
    }
}

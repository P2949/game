//! Map authoring (Phase 6) and content/runtime map services (Phase 7).
//!
//! [`MapAuthor`] declares a map in code or loads one from RON, referring to
//! prefabs by **name** (matching the RON format) instead of `PrefabId`. On
//! finalization the facade resolves names, validates the map, registers its
//! collision tilemap + theme for rendering, and records the full [`GameMap`] (with
//! objects) into a [`ContentRuntime`] resource so startup/reset systems can spawn
//! map objects without capturing a cloned map or `Rc<PrefabRegistry>`.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use anyhow::{Context, Result, anyhow};
use game_core::builder::{MapId, PrefabId, PrefabRegistry};
use game_core::commands::CommandQueue;
use game_core::world::Sprite;
use game_core::world::World;
use game_map::{
    GameMap, MapBuilder, MapCell, MapValidator, cell, load_game_map_ron, validate_map_prefabs,
};

use crate::app::GameApp;
use crate::assets::TextureRef;
use crate::bundle::vec2s;

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

impl PendingMap {
    /// Resolves prefab names, builds and validates the [`GameMap`], and returns it
    /// alongside its theme and start flag for registration.
    pub(crate) fn resolve(self, prefabs: &PrefabRegistry) -> Result<(GameMap, TileTheme, bool)> {
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

        let game_map = match self.source {
            MapSource::InCode {
                tile_size,
                rows,
                mut objects,
                legends,
            } => {
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
                mut objects,
                legends,
            } => {
                let full_path = beginner_asset_path(&path);
                let text = fs::read_to_string(&full_path).with_context(|| {
                    format!(
                        "map '{}' could not read text map '{}'. Place it under assets/",
                        self.name,
                        full_path.display()
                    )
                })?;
                let rows = text
                    .lines()
                    .map(|line| line.trim_end_matches('\r').to_owned())
                    .collect::<Vec<_>>();
                let rows = expand_symbolic_tiles(&self.name, rows, &mut objects, legends)?;
                let row_refs = rows.iter().map(String::as_str).collect::<Vec<_>>();
                let mut builder = MapBuilder::new(self.name.clone(), tile_size)
                    .try_tile_layer("collision", &row_refs)
                    .with_context(|| format!("map '{}' has invalid text-map tiles", self.name))?;
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
            MapSource::Ron { text } => load_game_map_ron(&text, |name| prefabs.id(name))
                .with_context(|| format!("map '{}' failed to load from RON", self.name))?,
        };

        let mut validator = MapValidator::new();
        for required in &self.required_objects {
            validator = validator.require_object(required);
        }
        validator
            .validate(&game_map)
            .with_context(|| format!("map '{}' validation failed", game_map.name))?;
        validate_map_prefabs(&game_map, prefabs)
            .with_context(|| format!("map '{}' references unknown prefab", game_map.name))?;

        Ok((game_map, theme, self.start))
    }
}

fn beginner_asset_path(path: &str) -> PathBuf {
    let relative = Path::new("assets").join(path);
    let Ok(current_dir) = std::env::current_dir() else {
        return relative;
    };
    for directory in current_dir.ancestors() {
        let candidate = directory.join(&relative);
        if candidate.is_file() {
            return candidate;
        }
    }
    current_dir.join(relative)
}

/// Builder for one map. Created by [`GameApp::map`] (in-code) or
/// [`GameApp::map_from_ron`] (external content). Finalized lazily by
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
            MapSource::Text { .. } | MapSource::Ron { .. } => {
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
        id: impl Into<String>,
        prefab: impl Into<String>,
        cell: MapCell,
    ) -> Self {
        match &mut self.pending.source {
            MapSource::InCode { objects, .. } | MapSource::Text { objects, .. } => {
                objects.push((id.into(), prefab.into(), cell));
            }
            MapSource::Ron { .. } => {
                self.pending
                    .misuse_errors
                    .push("spawn() is only valid on in-code maps, not RON maps".to_owned());
            }
        }
        self
    }

    /// Spawns a prefab wherever `symbol` appears in the tile rows. Symbols are
    /// treated as floor for collision. For example, `P` can mark the player start:
    /// `.tiles(["#P#"]).legend('P', "player")`.
    pub fn legend(mut self, symbol: char, prefab: impl Into<String>) -> Self {
        match &mut self.pending.source {
            MapSource::InCode { legends, .. } | MapSource::Text { legends, .. } => {
                legends.push((symbol, prefab.into()));
            }
            MapSource::Ron { .. } => {
                self.pending
                    .misuse_errors
                    .push("legend() is only valid on in-code maps, not RON maps".to_owned());
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
            MapSource::Ron { .. } => 32.0,
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
                        anyhow!(
                            "map '{map_name}' uses symbol {symbol:?} at row {row}, col {col}, but no legend was declared.\n\nAdd:\n    .legend({symbol:?}, \"prefab_name\")"
                        )
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
/// facade's built-in startup system and read by [`crate::GameCtx`] /
/// [`crate::StartupGameCtx`].
pub struct ContentRuntime {
    prefabs: Rc<PrefabRegistry>,
    maps: HashMap<String, GameMap>,
    map_ids: HashMap<String, MapId>,
    start_map: String,
    current_map: String,
}

impl ContentRuntime {
    pub(crate) fn new(
        prefabs: Rc<PrefabRegistry>,
        maps: HashMap<String, GameMap>,
        map_ids: HashMap<String, MapId>,
        start_map: String,
    ) -> Self {
        Self {
            prefabs,
            map_ids,
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

    pub fn map_id(&self, name: &str) -> Option<MapId> {
        self.map_ids.get(name).copied()
    }

    pub fn current_map_id(&self) -> Option<MapId> {
        self.map_id(&self.current_map)
    }

    pub fn start_map_id(&self) -> Option<MapId> {
        self.map_id(&self.start_map)
    }

    fn spawn_current(&self, world: &mut World) -> Result<()> {
        let map = self
            .maps
            .get(&self.current_map)
            .ok_or_else(|| anyhow!("unknown current map '{}'", self.current_map))?;
        spawn_map_objects(world, map, &self.prefabs)
    }
}

/// Spawns every object of `map` through `prefabs` (Phase 7.2). Shared by startup
/// and reset, so neither content crate carries its own copy.
pub fn spawn_map_objects(world: &mut World, map: &GameMap, prefabs: &PrefabRegistry) -> Result<()> {
    for object in &map.objects {
        prefabs.spawn(object.prefab, world, object.position, &object.properties)?;
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
    let map_id = content
        .map_id(&map_name)
        .ok_or_else(|| anyhow!("unknown map '{map_name}'"))?;
    content.current_map = map_name;
    clear_world_for_map_respawn(world);
    let result = content.spawn_current(world).map(|()| map_id);
    world.insert_resource(content);
    result
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

//! Map authoring (Phase 6) and content/runtime map services (Phase 7).
//!
//! [`MapAuthor`] declares a map in code or loads one from RON, referring to
//! prefabs by **name** (matching the RON format) instead of `PrefabId`. On
//! finalization the facade resolves names, validates the map, registers its
//! collision tilemap + theme for rendering, and records the full [`GameMap`] (with
//! objects) into a [`ContentRuntime`] resource so startup/reset systems can spawn
//! map objects without capturing a cloned map or `Rc<PrefabRegistry>`.

use std::collections::HashMap;
use std::rc::Rc;

use anyhow::{Context, Result, anyhow};
use game_core::builder::PrefabRegistry;
use game_core::commands::CommandQueue;
use game_core::world::World;
use game_map::{
    GameMap, MapBuilder, MapCell, MapValidator, load_game_map_ron, validate_map_prefabs,
};

use crate::app::GameApp;

/// Tile floor/wall sprites for a map. Re-exported from the engine so content can
/// build a theme with the prelude's `Sprite` type: `TileTheme { floor, wall }`.
pub use game_core::app::TileTheme;

/// Where a pending map's tiles/objects come from.
enum MapSource {
    InCode {
        tile_size: f32,
        rows: Vec<String>,
        objects: Vec<(String, String, MapCell)>,
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
}

impl PendingMap {
    /// Resolves prefab names, builds and validates the [`GameMap`], and returns it
    /// alongside its theme and start flag for registration.
    pub(crate) fn resolve(self, prefabs: &PrefabRegistry) -> Result<(GameMap, TileTheme, bool)> {
        let theme = self
            .theme
            .ok_or_else(|| anyhow!("map '{}' has no theme; call .theme(..)", self.name))?;

        let game_map = match self.source {
            MapSource::InCode {
                tile_size,
                rows,
                objects,
            } => {
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
                },
                required_objects: Vec::new(),
                theme: None,
                start: false,
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
            },
        }
    }

    /// Sets the tile size in world units (in-code maps only; RON maps carry it).
    pub fn tile_size(mut self, tile_size: f32) -> Self {
        if let MapSource::InCode { tile_size: t, .. } = &mut self.pending.source {
            *t = tile_size;
        }
        self
    }

    /// Sets the collision layer from rows of `.` (floor) / `#` (wall) (in-code
    /// maps only).
    pub fn tiles<const N: usize>(mut self, rows: [&str; N]) -> Self {
        if let MapSource::InCode { rows: r, .. } = &mut self.pending.source {
            *r = rows.iter().map(|row| (*row).to_owned()).collect();
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
        if let MapSource::InCode { objects, .. } = &mut self.pending.source {
            objects.push((id.into(), prefab.into(), cell));
        }
        self
    }

    /// Sets the map's floor/wall theme.
    pub fn theme(mut self, theme: TileTheme) -> Self {
        self.pending.theme = Some(theme);
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
}

/// Runtime content resource (Phase 7): the prefab registry and full maps (with
/// objects) needed to spawn/reset map objects. Inserted into the `World` by the
/// facade's built-in startup system and read by [`crate::GameCtx`] /
/// [`crate::StartupGameCtx`].
pub struct ContentRuntime {
    prefabs: Rc<PrefabRegistry>,
    maps: HashMap<String, GameMap>,
    start_map: String,
    current_map: String,
}

impl ContentRuntime {
    pub(crate) fn new(
        prefabs: Rc<PrefabRegistry>,
        maps: HashMap<String, GameMap>,
        start_map: String,
    ) -> Self {
        Self {
            prefabs,
            current_map: start_map.clone(),
            start_map,
            maps,
        }
    }

    /// The author name of the map currently spawned.
    pub fn current_map_name(&self) -> &str {
        &self.current_map
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

/// Clears the world and (re)spawns the start map's objects. Used by both startup
/// and reset, so the behavior is generic and identical.
pub(crate) fn reset_to_start_map_world(world: &mut World) -> Result<()> {
    let mut content = world
        .remove_resource::<ContentRuntime>()
        .ok_or_else(|| anyhow!("content runtime missing; was the game-kit plugin used?"))?;
    content.current_map = content.start_map.clone();
    world.clear();
    // `World::clear` preserves resources, so drop any commands queued against the
    // pre-reset world before respawning.
    if let Some(commands) = world.get_resource_mut::<CommandQueue>() {
        commands.clear();
    }
    let result = content.spawn_current(world);
    world.insert_resource(content);
    result
}

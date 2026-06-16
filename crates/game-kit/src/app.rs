//! The content-facing application builder (Phases 2, 5, 9).
//!
//! [`GameApp`] is what a [`GamePlugin`] operates on. It wraps the engine's
//! `GameBuilder` and exposes asset/input/prefab/map/system authoring, hiding the
//! builder, schedule, registries, and validators. The [`Plugin`] adapter bridges a
//! `game-kit` plugin to the engine's plugin trait so the runtime can run it.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use anyhow::{Context, Result, anyhow};
use game_core::builder::{GameBuilder, PrefabValidator};
use game_core::commands::CommandQueue;
use game_map::GameMap;

use crate::assets::AssetAuthor;
use crate::context::{GameCtx, StartupGameCtx};
use crate::helpers::SimulationState;
use crate::input::InputAuthor;
use crate::map::{ContentRuntime, MapAuthor, PendingMap};
use crate::prefab::PrefabAuthor;
use crate::system::{GameSystem, StartupSystem};

/// A deferred prefab component requirement: applies one
/// `validator.require_component::<T>(name)` call during [`GameApp::finish`].
pub(crate) type PrefabRequirement = Box<dyn for<'v> FnOnce(&mut PrefabValidator<'v>)>;

/// A game, expressed as content: assets, controls, prefabs, maps, and systems.
///
/// Implemented by each content crate's plugin type and run by the runtime via
/// [`plugin`].
pub trait GamePlugin {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()>;
}

/// The content-facing builder. Methods register content into the underlying engine
/// builder; map finalization and prefab validation run in [`Self::finish`] before
/// the runtime enters its loop.
pub struct GameApp<'app> {
    builder: &'app mut GameBuilder,
    /// Filled in [`Self::finish`]; consumed by the built-in startup system that
    /// installs the content runtime resource.
    content: Rc<RefCell<Option<ContentRuntime>>>,
    pending_maps: Vec<PendingMap>,
    prefab_requirements: Vec<PrefabRequirement>,
}

impl<'app> GameApp<'app> {
    pub(crate) fn new(builder: &'app mut GameBuilder) -> Self {
        let content: Rc<RefCell<Option<ContentRuntime>>> = Rc::new(RefCell::new(None));

        // Built-in startup system, registered first so it runs before any content
        // startup system: install the content runtime (maps + prefabs) and the
        // command queue. Content therefore never inserts these itself (Phase 7.4).
        let content_for_startup = Rc::clone(&content);
        builder.schedule_mut().add_startup(move |ctx| {
            if let Some(runtime) = content_for_startup.borrow_mut().take() {
                ctx.world.insert_resource(runtime);
            }
            ctx.world.resource_or_insert_with(CommandQueue::new);
            Ok(())
        });

        Self {
            builder,
            content,
            pending_maps: Vec::new(),
            prefab_requirements: Vec::new(),
        }
    }

    /// Declares assets, returning whatever the closure builds (typically the
    /// content's asset-handle struct).
    pub fn assets<R>(&mut self, f: impl FnOnce(&mut AssetAuthor<'_>) -> Result<R>) -> Result<R> {
        let mut author = AssetAuthor::new(self.builder.assets_mut());
        f(&mut author)
    }

    /// Declares logical controls, returning whatever the closure builds (typically
    /// the content's controls struct).
    pub fn input<R>(&mut self, f: impl FnOnce(&mut InputAuthor<'_>) -> Result<R>) -> Result<R> {
        let mut author = InputAuthor::new(self.builder.input_mut());
        f(&mut author)
    }

    /// Defines a prefab under `name`.
    pub fn prefab(
        &mut self,
        name: impl Into<String>,
        build: impl FnOnce(&mut PrefabAuthor<'_>) -> Result<()>,
    ) -> Result<()> {
        let mut author = PrefabAuthor::new(
            name.into(),
            self.builder.prefabs_mut(),
            &mut self.prefab_requirements,
        );
        build(&mut author)
    }

    /// Begins declaring an in-code map.
    pub fn map(&mut self, name: impl Into<String>) -> MapAuthor<'_, 'app> {
        MapAuthor::in_code(self, name.into())
    }

    /// Begins declaring a map from an external RON document.
    pub fn map_from_ron(&mut self, ron: impl Into<String>) -> MapAuthor<'_, 'app> {
        MapAuthor::from_ron(self, ron.into())
    }

    pub(crate) fn push_pending_map(&mut self, pending: PendingMap) {
        self.pending_maps.push(pending);
    }

    /// Registers a startup system.
    pub fn startup(&mut self, mut system: impl StartupSystem) {
        self.builder.schedule_mut().add_startup(move |ctx| {
            let mut game = StartupGameCtx::new(ctx);
            system.run(&mut game)
        });
    }

    /// Registers a fixed-timestep system.
    pub fn fixed(&mut self, mut system: impl GameSystem) {
        self.builder.schedule_mut().add_fixed(move |ctx, dt| {
            let mut game = GameCtx::new(ctx);
            system.run(&mut game, dt);
        });
    }

    /// Registers a fixed-timestep system that runs only while resource `S` is
    /// present and active.
    pub fn fixed_active<S>(&mut self, mut system: impl GameSystem)
    where
        S: SimulationState + 'static,
    {
        self.fixed(move |game: &mut GameCtx<'_, '_>, dt| {
            if game.resource::<S>().is_some_and(SimulationState::active) {
                system.run(game, dt);
            }
        });
    }

    /// Registers a per-frame update system.
    pub fn update(&mut self, mut system: impl GameSystem) {
        self.builder.schedule_mut().add_update(move |ctx, dt| {
            let mut game = GameCtx::new(ctx);
            system.run(&mut game, dt);
        });
    }

    /// Registers an update system that runs only while resource `S` is present
    /// and active.
    pub fn update_active<S>(&mut self, mut system: impl GameSystem)
    where
        S: SimulationState + 'static,
    {
        self.update(move |game: &mut GameCtx<'_, '_>, dt| {
            if game.resource::<S>().is_some_and(SimulationState::active) {
                system.run(game, dt);
            }
        });
    }

    /// Registers a per-frame render-extraction system. (The runtime extracts
    /// tilemap/entity sprites itself, so most content does not need this.)
    pub fn render_extract(&mut self, mut system: impl GameSystem) {
        self.builder
            .schedule_mut()
            .add_render_extract(move |ctx, dt| {
                let mut game = GameCtx::new(ctx);
                system.run(&mut game, dt);
            });
    }

    /// Registers a per-frame UI system.
    pub fn ui(&mut self, mut system: impl GameSystem) {
        self.builder.schedule_mut().add_ui(move |ctx, dt| {
            let mut game = GameCtx::new(ctx);
            system.run(&mut game, dt);
        });
    }

    /// Declares that every fixed system self-guards against the paused/dead state,
    /// satisfying the schedule validator.
    pub fn fixed_systems_are_pause_guarded(&mut self) {
        self.builder.schedule_mut().mark_fixed_pause_guarded();
    }

    /// Finalizes content: resolves and validates maps, registers their collision
    /// tilemaps + themes, validates prefab requirements, and records the content
    /// runtime for startup. Called by the [`Plugin`] adapter after the plugin's
    /// `build`, so failures surface before any backend is created.
    pub(crate) fn finish(mut self) -> Result<()> {
        let mut maps: HashMap<String, GameMap> = HashMap::new();
        let mut start_map_name: Option<String> = None;

        for pending in std::mem::take(&mut self.pending_maps) {
            let (game_map, theme, start) = pending.resolve(self.builder.prefabs())?;
            let map_id = self.builder.maps_mut().try_register(
                game_map.name.clone(),
                game_map.collision_tilemap(),
                theme,
            )?;
            if start {
                if let Some(previous) = &start_map_name {
                    anyhow::bail!(
                        "multiple start maps declared: '{}' and '{}'; runtime map switching is not implemented",
                        previous,
                        game_map.name
                    );
                }

                self.builder.set_start_map(map_id);
                start_map_name = Some(game_map.name.clone());
            }
            maps.insert(game_map.name.clone(), game_map);
        }

        {
            let mut validator = PrefabValidator::new(self.builder.prefabs());
            for requirement in std::mem::take(&mut self.prefab_requirements) {
                requirement(&mut validator);
            }
            validator.validate().context("prefab validation failed")?;
        }

        let start_map = start_map_name
            .ok_or_else(|| anyhow!("no start map declared; call .start() on one map"))?;
        let prefabs = self.builder.prefabs_shared();
        *self.content.borrow_mut() = Some(ContentRuntime::new(prefabs, maps, start_map));
        Ok(())
    }
}

/// Adapts a [`GamePlugin`] (the content-facing trait) to the engine's
/// `game_core::plugin::GamePlugin` so the runtime can run it. Build a value with
/// [`plugin`].
pub struct Plugin<P>(P);

impl<P: GamePlugin> game_core::plugin::GamePlugin for Plugin<P> {
    fn build(&self, builder: &mut GameBuilder) -> Result<()> {
        let mut app = GameApp::new(builder);
        self.0.build(&mut app)?;
        app.finish()
    }
}

/// Wraps a content plugin so it can be handed to `game_runtime::run`. Content's
/// `pub fn plugin()` returns `game_kit::plugin(MyPlugin)`.
pub fn plugin<P: GamePlugin>(plugin: P) -> Plugin<P> {
    Plugin(plugin)
}

#[cfg(test)]
mod tests {
    use game_core::backend::TextureHandle;
    use game_core::builder::GameBuilder;
    use game_core::input::Key;
    use game_core::world::{Sprite, Transform};
    use game_map::cell;

    use super::GameApp;
    use crate::map::TileTheme;

    fn test_theme() -> TileTheme {
        TileTheme {
            floor: Sprite::new(TextureHandle(1), glam::Vec2::splat(16.0)),
            wall: Sprite::new(TextureHandle(2), glam::Vec2::splat(16.0)),
        }
    }

    #[test]
    fn duplicate_prefab_name_returns_error() {
        let mut builder = GameBuilder::new();
        let mut game = GameApp::new(&mut builder);

        game.prefab("duplicate", |prefab| {
            prefab.spawn(|at| (Transform::at(at),))?;
            Ok(())
        })
        .unwrap();

        let err = game
            .prefab("duplicate", |prefab| {
                prefab.spawn(|at| (Transform::at(at),))?;
                Ok(())
            })
            .unwrap_err();

        assert!(err.to_string().contains("duplicate prefab"));
    }

    #[test]
    fn duplicate_input_action_returns_error() {
        let mut builder = GameBuilder::new();
        let mut game = GameApp::new(&mut builder);

        let err = game
            .input(|input| {
                input.action("pause")?.key(Key::P);
                input.action("pause")?.key(Key::R);
                Ok(())
            })
            .unwrap_err();

        assert!(err.to_string().contains("duplicate input action"));
    }

    #[test]
    fn conflicting_texture_key_returns_error() {
        let mut builder = GameBuilder::new();
        let mut game = GameApp::new(&mut builder);

        let err = game
            .assets(|assets| {
                assets.texture("hero", "textures/a.png")?;
                assets.texture("hero", "textures/b.png")?;
                Ok(())
            })
            .unwrap_err();

        assert!(err.to_string().contains("texture asset key"));
    }

    #[test]
    fn ron_map_rejects_in_code_authoring_calls() {
        let mut builder = GameBuilder::new();
        let mut game = GameApp::new(&mut builder);

        game.map_from_ron("")
            .tile_size(16.0)
            .tiles(["."])
            .spawn("player_start", "demo/player", cell(0, 0))
            .theme(test_theme())
            .start();

        let err = game.finish().unwrap_err();
        let message = err.to_string();

        assert!(message.contains("map '<ron>' has invalid authoring calls"));
        assert!(message.contains("tile_size() is only valid on in-code maps"));
        assert!(message.contains("tiles() is only valid on in-code maps"));
        assert!(message.contains("spawn() is only valid on in-code maps"));
    }

    #[test]
    fn no_start_map_returns_error() {
        let mut builder = GameBuilder::new();
        let game = GameApp::new(&mut builder);

        let err = game.finish().unwrap_err();

        assert!(
            err.to_string()
                .contains("no start map declared; call .start() on one map")
        );
    }

    #[test]
    fn multiple_start_maps_return_error() {
        let mut builder = GameBuilder::new();
        let mut game = GameApp::new(&mut builder);

        game.map("first").tiles(["."]).theme(test_theme()).start();
        game.map("second").tiles(["."]).theme(test_theme()).start();

        let err = game.finish().unwrap_err();

        assert!(
            err.to_string()
                .contains("multiple start maps declared: 'first' and 'second'")
        );
    }
}

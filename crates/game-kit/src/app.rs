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
use game_core::input::ActionId;
use game_core::world::EntityId;
use game_map::GameMap;

use crate::assets::{AssetAuthor, AssetBagAuthor};
use crate::beginner::debug::{DebugOverlay, draw_debug_overlay};
use crate::beginner::defaults::TopDownGameAuthor;
use crate::beginner::events::DEFAULT_PICKUP_COLLECT_RANGE;
use crate::beginner::prefabs::{
    DoorPrefabAuthor, EnemyPrefabAuthor, PickupPrefabAuthor, PlayerPrefabAuthor,
    ProjectilePrefabAuthor, SpawnerPrefabAuthor,
};
use crate::beginner::rules::RulesAuthor;
use crate::beginner::scene::{SceneRegistry, SceneState, SimpleSceneFlowAuthor};
use crate::beginner::state::SimpleGameState;
use crate::beginner::time::MIN_TIMER_SECONDS;
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
    scenes: Vec<String>,
    start_scene: Option<String>,
    debug_overlay: Option<DebugOverlay>,
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
            scenes: Vec::new(),
            start_scene: None,
            debug_overlay: None,
        }
    }

    /// Declares assets, returning whatever the closure builds (typically the
    /// content's asset-handle struct).
    pub fn assets<R>(&mut self, f: impl FnOnce(&mut AssetAuthor<'_>) -> Result<R>) -> Result<R> {
        let mut author = AssetAuthor::new(self.builder.assets_mut());
        f(&mut author)
    }

    /// Begins a beginner-friendly named asset bag.
    pub fn asset_bag(&mut self) -> AssetBagAuthor<'_> {
        AssetBagAuthor::new(AssetAuthor::new(self.builder.assets_mut()))
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

    /// Begins a beginner-friendly player prefab.
    pub fn player_prefab(&mut self, name: impl Into<String>) -> PlayerPrefabAuthor<'_, 'app> {
        PlayerPrefabAuthor::new(self, name.into())
    }

    /// Begins a beginner-friendly enemy prefab.
    pub fn enemy_prefab(&mut self, name: impl Into<String>) -> EnemyPrefabAuthor<'_, 'app> {
        EnemyPrefabAuthor::new(self, name.into())
    }

    /// Begins a beginner-friendly pickup prefab.
    pub fn pickup_prefab(&mut self, name: impl Into<String>) -> PickupPrefabAuthor<'_, 'app> {
        PickupPrefabAuthor::new(self, name.into())
    }

    /// Begins a beginner-friendly door prefab.
    pub fn door_prefab(&mut self, name: impl Into<String>) -> DoorPrefabAuthor<'_, 'app> {
        DoorPrefabAuthor::new(self, name.into())
    }

    /// Begins a beginner-friendly projectile prefab.
    pub fn projectile_prefab(
        &mut self,
        name: impl Into<String>,
    ) -> ProjectilePrefabAuthor<'_, 'app> {
        ProjectilePrefabAuthor::new(self, name.into())
    }

    /// Begins a beginner-friendly spawner prefab.
    pub fn spawner_prefab(&mut self, name: impl Into<String>) -> SpawnerPrefabAuthor<'_, 'app> {
        SpawnerPrefabAuthor::new(self, name.into())
    }

    /// Begins configuring a beginner top-down game preset.
    pub fn use_top_down_game(&mut self) -> TopDownGameAuthor<'_, 'app> {
        TopDownGameAuthor::new(self)
    }

    /// Begins configuring declarative beginner rules.
    pub fn rules(&mut self) -> RulesAuthor<'_, 'app> {
        RulesAuthor::new(self)
    }

    /// Begins declaring an in-code map.
    pub fn map(&mut self, name: impl Into<String>) -> MapAuthor<'_, 'app> {
        MapAuthor::in_code(self, name.into())
    }

    /// Begins declaring a map from an external RON document.
    pub fn map_from_ron(&mut self, ron: impl Into<String>) -> MapAuthor<'_, 'app> {
        MapAuthor::from_ron(self, ron.into())
    }

    /// Declares a named beginner scene.
    pub fn scene(&mut self, name: impl Into<String>) -> &mut Self {
        let name = name.into();
        if !self.scenes.iter().any(|scene| scene == &name) {
            self.scenes.push(name);
        }
        self
    }

    /// Declares a named beginner menu scene.
    pub fn menu_scene(&mut self, name: impl Into<String>) -> &mut Self {
        self.scene(name)
    }

    /// Declares a named beginner level scene.
    pub fn level_scene(&mut self, name: impl Into<String>) -> &mut Self {
        self.scene(name)
    }

    /// Declares a named beginner game-over scene.
    pub fn game_over_scene(&mut self, name: impl Into<String>) -> &mut Self {
        self.scene(name)
    }

    /// Declares a named beginner scene and makes it the initial scene.
    pub fn start_scene(&mut self, name: impl Into<String>) -> &mut Self {
        let name = name.into();
        self.scene(name.clone());
        self.start_scene = Some(name);
        self
    }

    /// Begins configuring a simple menu -> level -> game-over scene flow.
    pub fn use_simple_scene_flow(&mut self) -> SimpleSceneFlowAuthor<'_, 'app> {
        SimpleSceneFlowAuthor::new(self)
    }

    pub fn enable_debug_overlay(&mut self) -> &mut Self {
        self.debug_overlay
            .get_or_insert_with(DebugOverlay::enabled)
            .enabled = true;
        self
    }

    pub fn debug(&mut self) -> DebugOverlayAuthor<'_, 'app> {
        self.enable_debug_overlay();
        DebugOverlayAuthor { app: self }
    }

    pub(crate) fn configure_debug_overlay(&mut self, f: impl FnOnce(&mut DebugOverlay)) {
        let overlay = self.debug_overlay.get_or_insert_with(DebugOverlay::enabled);
        f(overlay);
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

    /// Beginner alias for [`Self::startup`].
    pub fn on_start(&mut self, system: impl StartupSystem) {
        self.startup(system);
    }

    /// Registers a fixed-timestep system.
    pub fn fixed(&mut self, mut system: impl GameSystem) {
        self.builder.schedule_mut().add_fixed(move |ctx, dt| {
            let mut game = GameCtx::new(ctx);
            system.run(&mut game, dt);
        });
    }

    /// Beginner alias for [`Self::fixed`].
    pub fn every_tick(&mut self, system: impl GameSystem) {
        self.fixed(system);
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

    /// Beginner alias for [`Self::fixed_active`].
    pub fn every_active_tick<S>(&mut self, system: impl GameSystem)
    where
        S: SimulationState + 'static,
    {
        self.fixed_active::<S>(system);
    }

    /// Registers a per-frame update system.
    pub fn update(&mut self, mut system: impl GameSystem) {
        self.builder.schedule_mut().add_update(move |ctx, dt| {
            let mut game = GameCtx::new(ctx);
            system.run(&mut game, dt);
        });
    }

    /// Beginner alias for [`Self::update`].
    pub fn every_frame(&mut self, system: impl GameSystem) {
        self.update(system);
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

    /// Beginner alias for [`Self::update_active`].
    pub fn every_active_frame<S>(&mut self, system: impl GameSystem)
    where
        S: SimulationState + 'static,
    {
        self.update_active::<S>(system);
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

    /// Beginner alias for [`Self::ui`].
    pub fn draw_ui(&mut self, system: impl GameSystem) {
        self.ui(system);
    }

    /// Runs `f` on fixed ticks where `action` was pressed.
    pub fn on_action(
        &mut self,
        action: ActionId,
        mut f: impl FnMut(&mut GameCtx<'_, '_>) + 'static,
    ) {
        self.fixed(move |game: &mut GameCtx<'_, '_>, _dt| {
            if game.pressed(action) {
                f(game);
            }
        });
    }

    /// Runs `f` on fixed ticks where `action` was pressed and the simple
    /// beginner state is active.
    pub fn on_action_when_playing(
        &mut self,
        action: ActionId,
        mut f: impl FnMut(&mut GameCtx<'_, '_>) + 'static,
    ) {
        self.every_active_tick::<SimpleGameState>(move |game: &mut GameCtx<'_, '_>, _dt| {
            if game.pressed(action) {
                f(game);
            }
        });
    }

    pub fn on_action_cooldown(
        &mut self,
        action: ActionId,
        seconds: f32,
        mut f: impl FnMut(&mut GameCtx<'_, '_>) + 'static,
    ) {
        let seconds = seconds.max(0.0);
        let mut cooldown: f32 = 0.0;
        self.every_tick(move |game: &mut GameCtx<'_, '_>, dt: f32| {
            cooldown = (cooldown - dt).max(0.0);
            if cooldown == 0.0 && game.pressed(action) {
                cooldown = seconds;
                f(game);
            }
        });
    }

    pub fn every_seconds(
        &mut self,
        seconds: f32,
        mut f: impl FnMut(&mut GameCtx<'_, '_>) + 'static,
    ) {
        let seconds = seconds.max(MIN_TIMER_SECONDS);
        let mut timer = 0.0;
        self.every_tick(move |game: &mut GameCtx<'_, '_>, dt| {
            timer += dt;
            while timer >= seconds {
                timer -= seconds;
                f(game);
            }
        });
    }

    pub fn after_seconds(
        &mut self,
        seconds: f32,
        mut f: impl FnMut(&mut GameCtx<'_, '_>) + 'static,
    ) {
        let seconds = seconds.max(0.0);
        let mut timer = 0.0;
        let mut done = false;
        self.every_tick(move |game: &mut GameCtx<'_, '_>, dt| {
            if done {
                return;
            }
            timer += dt;
            if timer >= seconds {
                done = true;
                f(game);
            }
        });
    }

    pub fn on_enemy_death(&mut self, mut f: impl FnMut(&mut GameCtx<'_, '_>) + 'static) {
        let mut known_dead: Vec<EntityId> = Vec::new();
        self.every_tick(move |game: &mut GameCtx<'_, '_>, _dt| {
            let dead = game
                .enemy_ids()
                .into_iter()
                .filter(|id| game.is_dead(*id))
                .collect::<Vec<_>>();
            for id in &dead {
                if !known_dead.contains(id) {
                    f(game);
                }
            }
            known_dead = dead;
        });
    }

    pub fn on_player_collect_pickup(&mut self, f: impl FnMut(&mut GameCtx<'_, '_>) + 'static) {
        self.on_player_collect_pickup_within(DEFAULT_PICKUP_COLLECT_RANGE, f);
    }

    pub fn on_player_touching_pickup(&mut self, f: impl FnMut(&mut GameCtx<'_, '_>) + 'static) {
        self.on_player_collect_pickup(f);
    }

    pub fn on_player_collect_pickup_within(
        &mut self,
        range: f32,
        mut f: impl FnMut(&mut GameCtx<'_, '_>) + 'static,
    ) {
        self.every_tick(move |game: &mut GameCtx<'_, '_>, _dt| {
            if game.collect_pickups_near_player(range) > 0 {
                f(game);
            }
        });
    }

    pub fn on_scene_enter(
        &mut self,
        scene: impl Into<String>,
        mut f: impl FnMut(&mut GameCtx<'_, '_>) + 'static,
    ) {
        let scene = scene.into();
        let mut was_in_scene = false;
        self.every_frame(move |game: &mut GameCtx<'_, '_>, _dt| {
            let is_in_scene = game.current_scene_name().as_deref() == Some(scene.as_str());
            if is_in_scene && !was_in_scene {
                f(game);
            }
            was_in_scene = is_in_scene;
        });
    }

    pub fn on_scene(&mut self, scene: impl Into<String>, mut system: impl GameSystem) {
        let scene = scene.into();
        self.every_frame(move |game: &mut GameCtx<'_, '_>, dt| {
            if game.current_scene_name().as_deref() == Some(scene.as_str()) {
                system.run(game, dt);
            }
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
        let mut map_ids: HashMap<String, game_core::builder::MapId> = HashMap::new();
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
                        "Multiple start maps declared: '{}' and '{}'.\n\nMark exactly one map with .start(). Other maps should end with .finish().",
                        previous,
                        game_map.name
                    );
                }

                self.builder.set_start_map(map_id);
                start_map_name = Some(game_map.name.clone());
            }
            map_ids.insert(game_map.name.clone(), map_id);
            maps.insert(game_map.name.clone(), game_map);
        }

        {
            let mut validator = PrefabValidator::new(self.builder.prefabs());
            for requirement in std::mem::take(&mut self.prefab_requirements) {
                requirement(&mut validator);
            }
            validator.validate().context("prefab validation failed")?;
        }

        let start_map = start_map_name.ok_or_else(|| {
            anyhow!(
                "No start map declared.\n\nAdd .start() to exactly one map:\n    game.map(\"level_1\")\n        .tiles([\"...\"])\n        .simple_theme(assets.floor, assets.wall)\n        .start();"
            )
        })?;
        let prefabs = self.builder.prefabs_shared();
        *self.content.borrow_mut() = Some(ContentRuntime::new(prefabs, maps, map_ids, start_map));

        if !self.scenes.is_empty() {
            let registry = SceneRegistry::new(self.scenes.clone());
            let start_scene = self
                .start_scene
                .clone()
                .unwrap_or_else(|| self.scenes[0].clone());
            self.builder.schedule_mut().add_startup(move |ctx| {
                ctx.world.insert_resource(registry.clone());
                ctx.world
                    .insert_resource(SceneState::new(start_scene.clone()));
                Ok(())
            });
        }

        if let Some(overlay) = self.debug_overlay {
            self.builder.schedule_mut().add_startup(move |ctx| {
                ctx.world.insert_resource(overlay);
                Ok(())
            });
            self.builder.schedule_mut().add_ui(move |ctx, dt| {
                let mut game = GameCtx::new(ctx);
                draw_debug_overlay(&mut game, dt);
            });
        }
        Ok(())
    }
}

pub struct DebugOverlayAuthor<'a, 'app> {
    app: &'a mut GameApp<'app>,
}

impl DebugOverlayAuthor<'_, '_> {
    pub fn show_colliders(self) -> Self {
        self.app
            .configure_debug_overlay(|overlay| overlay.show_colliders = true);
        self
    }

    pub fn show_nav(self) -> Self {
        self.app
            .configure_debug_overlay(|overlay| overlay.show_nav = true);
        self
    }

    pub fn show_names(self) -> Self {
        self.app
            .configure_debug_overlay(|overlay| overlay.show_names = true);
        self
    }

    pub fn show_fps(self) -> Self {
        self.app
            .configure_debug_overlay(|overlay| overlay.show_fps = true);
        self
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

pub struct FnGamePlugin<F>(F);

impl<F> GamePlugin for FnGamePlugin<F>
where
    F: for<'app> Fn(&mut GameApp<'app>) -> Result<()>,
{
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        (self.0)(game)
    }
}

pub fn plugin_fn<F>(build: F) -> Plugin<FnGamePlugin<F>>
where
    F: for<'app> Fn(&mut GameApp<'app>) -> Result<()>,
{
    plugin(FnGamePlugin(build))
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

        assert!(err.to_string().contains("Duplicate input action"));
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

        assert!(err.to_string().contains("Texture asset key"));
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
    fn map_without_theme_points_to_simple_theme() {
        let mut builder = GameBuilder::new();
        let mut game = GameApp::new(&mut builder);

        game.map("demo").tiles(["."]).start();

        let err = game.finish().unwrap_err();
        let message = err.to_string();

        assert!(message.contains("Map 'demo' has no tile theme."));
        assert!(message.contains(".simple_theme(assets.floor, assets.wall)"));
    }

    #[test]
    fn simple_theme_satisfies_map_theme_requirement() {
        let mut builder = GameBuilder::new();
        let mut game = GameApp::new(&mut builder);

        game.map("demo")
            .tiles(["."])
            .simple_theme(TextureHandle(1), TextureHandle(2))
            .start();

        game.finish().unwrap();
    }

    #[test]
    fn no_start_map_returns_error() {
        let mut builder = GameBuilder::new();
        let game = GameApp::new(&mut builder);

        let err = game.finish().unwrap_err();

        let message = err.to_string();
        assert!(message.contains("No start map declared."));
        assert!(message.contains(".simple_theme(assets.floor, assets.wall)"));
    }

    #[test]
    fn multiple_start_maps_return_error() {
        let mut builder = GameBuilder::new();
        let mut game = GameApp::new(&mut builder);

        game.map("first").tiles(["."]).theme(test_theme()).start();
        game.map("second").tiles(["."]).theme(test_theme()).start();

        let err = game.finish().unwrap_err();

        let message = err.to_string();
        assert!(message.contains("Multiple start maps declared: 'first' and 'second'"));
        assert!(message.contains("Other maps should end with .finish()"));
    }
}

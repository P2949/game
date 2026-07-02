//! The content-facing application builder (Phases 2, 5, 9).
//!
//! [`GameApp`] is what a [`GamePlugin`] operates on. It wraps the engine's
//! `GameBuilder` and exposes asset/input/prefab/map/system authoring, hiding the
//! builder, schedule, registries, and validators. The [`Plugin`] adapter bridges a
//! `game-kit` plugin to the engine's plugin trait so the runtime can run it.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use anyhow::{Context, Result, anyhow};
use game_core::builder::{GameBuilder, PrefabValidator};
use game_core::commands::CommandQueue;
use game_core::input::ActionId;
use game_core::query::ParamSystem;
use game_core::world::EntityId;
use game_map::GameMap;
use glam::Vec2;

use crate::assets::{
    AssetAuthor, AssetBagAuthor, AssetFolderAuthor, AssetLookup, SoundRef, TextureRef,
    missing_asset_error,
};
use crate::beginner::actors::{Door, DoorTarget};
use crate::beginner::animation::AnimationFinishedEvents;
use crate::beginner::context::{Game as BeginnerGame, Seconds, StartupGame as BeginnerStartupGame};
use crate::beginner::custom_rules::CustomRuleAuthor;
use crate::beginner::debug::{DebugIterationInfo, DebugOverlay, draw_debug_overlay};
use crate::beginner::defaults::TopDownGameAuthor;
use crate::beginner::events::{
    ActorToken, AnimationFinishedEvent, CollectEvent, CollisionEvent, DEFAULT_PICKUP_COLLECT_RANGE,
    DoorEvent, EnemyDeathEvent, MapChangedEvent, OverlapTracker, ProjectileHitEvent,
};
use crate::beginner::prefabs::{
    AreaPrefabAuthor, DoorPrefabAuthor, EnemyPrefabAuthor, PickupPrefabAuthor, PlayerPrefabAuthor,
    ProjectilePrefabAuthor, SpawnerPrefabAuthor,
};
use crate::beginner::rules::RulesAuthor;
use crate::beginner::scene::{SceneRegistry, SceneState, SimpleSceneFlowAuthor};
use crate::beginner::state::SimpleGameState;
use crate::beginner::time::MIN_TIMER_SECONDS;
use crate::beginner::tuning::TuningFile;
use crate::context::{GameCtx, StartupGameCtx, drain_beginner_spawn_queue};
use crate::helpers::SimulationState;
use crate::input::InputAuthor;
use crate::map::{ContentRuntime, MapAuthor, PendingMap};
use crate::prefab::{IntoContentName, PrefabAuthor};
use crate::system::{GameSystem, StartupSystem};

mod debug;
mod plugin;
#[cfg(test)]
mod tests;
mod validation;

pub use debug::DebugOverlayAuthor;
pub use plugin::{FnGamePlugin, Plugin, plugin, plugin_fn};

use validation::{matching_nearby_prefabs, matching_overlaps, prefab_matches};

/// A deferred prefab component requirement: applies one
/// `validator.require_component::<T>(name)` call during [`GameApp::finish`].
pub(crate) type PrefabRequirement = Box<dyn for<'v> FnOnce(&mut PrefabValidator<'v>)>;

const DEFAULT_DOOR_TRIGGER_RANGE: f32 = 28.0;
const DEFAULT_PROJECTILE_HIT_RANGE: f32 = 16.0;

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
    asset_lookup: Rc<RefCell<Option<AssetLookup>>>,
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
        let asset_lookup: Rc<RefCell<Option<AssetLookup>>> = Rc::new(RefCell::new(None));
        let asset_lookup_for_startup = Rc::clone(&asset_lookup);
        builder.schedule_mut().add_startup(move |ctx| {
            if let Some(runtime) = content_for_startup.borrow_mut().take() {
                ctx.world.insert_resource(runtime);
            }
            if let Some(lookup) = asset_lookup_for_startup.borrow_mut().take() {
                ctx.world.insert_resource(lookup);
            }
            ctx.world.resource_or_insert_with(CommandQueue::new);
            Ok(())
        });

        Self {
            builder,
            content,
            pending_maps: Vec::new(),
            prefab_requirements: Vec::new(),
            asset_lookup,
            scenes: Vec::new(),
            start_scene: None,
            debug_overlay: None,
        }
    }

    /// Adds one independently composable game behavior.
    ///
    /// Beginner presets use this internally, and content can use the same
    /// behavior types when it needs a custom combination instead of a whole
    /// preset builder.
    pub fn use_behavior(&mut self, behavior: impl GamePlugin) -> Result<()> {
        behavior.build(self)
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

    /// Begins registering conventional `assets/textures`, `assets/sounds`, and
    /// `assets/music` filenames with friendly keys.
    pub fn assets_from_folders(&mut self) -> AssetFolderAuthor<'_> {
        AssetFolderAuthor::new(self.asset_bag())
    }

    /// Loads a primary no-Rust `game.toml` package file. The package's asset
    /// root is the `assets/` directory next to `game.toml`.
    pub fn load_authoring_file(
        &mut self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<crate::input::TopDownControls> {
        crate::data::load_authoring_file(self, path)
    }

    /// Loads a primary no-Rust `game.toml` package file with an explicit asset
    /// root. Relative asset roots are resolved from the package root.
    pub fn load_authoring_file_with_asset_root(
        &mut self,
        path: impl AsRef<std::path::Path>,
        asset_root: impl AsRef<std::path::Path>,
    ) -> Result<crate::input::TopDownControls> {
        crate::data::load_authoring_file_with_asset_root(self, path, asset_root)
    }

    /// Loads `assets/game.ron`-style beginner content through the same public
    /// asset, prefab, map, input, and rule builders used by Rust-authored games.
    /// Add custom Rust behavior after this call for the hybrid workflow.
    pub fn load_beginner_file(
        &mut self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<crate::input::TopDownControls> {
        crate::data::load_beginner_game_file(self, path)
    }

    /// Loads named numeric values from an `assets/` TOML file and installs the
    /// same file as a runtime resource for development-time reload support.
    /// Legacy RON tuning files remain supported while old projects migrate.
    pub fn tuning_from_file(&mut self, path: impl AsRef<std::path::Path>) -> Result<TuningFile> {
        let tuning = TuningFile::from_file(path)?;
        let startup_tuning = tuning.clone();
        self.startup(move |game: &mut StartupGameCtx<'_, '_>| {
            game.insert_resource(startup_tuning.clone());
            Ok(())
        });
        Ok(tuning)
    }

    pub(crate) fn resolve_texture(&self, key: &str) -> Result<game_core::backend::TextureHandle> {
        self.builder.assets().texture_handle(key).ok_or_else(|| {
            missing_asset_error("texture", key, self.builder.assets().texture_keys())
        })
    }

    pub(crate) fn resolve_sound(&self, key: &str) -> Result<game_core::backend::SoundHandle> {
        self.builder
            .assets()
            .sound_handle(key)
            .ok_or_else(|| missing_asset_error("sound", key, self.builder.assets().sound_keys()))
    }

    pub(crate) fn resolve_texture_ref(
        &self,
        texture: TextureRef,
    ) -> Result<game_core::backend::TextureHandle> {
        match texture {
            TextureRef::Handle(handle) => Ok(handle),
            TextureRef::Key(key) => self.resolve_texture(&key),
        }
    }

    pub(crate) fn resolve_sound_ref(
        &self,
        sound: SoundRef,
    ) -> Result<game_core::backend::SoundHandle> {
        match sound {
            SoundRef::Handle(handle) => Ok(handle),
            SoundRef::Key(key) => self.resolve_sound(&key),
        }
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
        name: impl IntoContentName,
        build: impl FnOnce(&mut PrefabAuthor<'_>) -> Result<()>,
    ) -> Result<()> {
        let mut author = PrefabAuthor::new(
            name.into_content_name(),
            self.builder.prefabs_mut(),
            &mut self.prefab_requirements,
        );
        build(&mut author)
    }

    /// Begins a beginner-friendly player prefab.
    pub fn player_prefab(&mut self, name: impl IntoContentName) -> PlayerPrefabAuthor<'_, 'app> {
        PlayerPrefabAuthor::new(self, name.into_content_name())
    }

    /// Begins a beginner-friendly enemy prefab.
    pub fn enemy_prefab(&mut self, name: impl IntoContentName) -> EnemyPrefabAuthor<'_, 'app> {
        EnemyPrefabAuthor::new(self, name.into_content_name())
    }

    /// Begins a beginner-friendly pickup prefab.
    pub fn pickup_prefab(&mut self, name: impl IntoContentName) -> PickupPrefabAuthor<'_, 'app> {
        PickupPrefabAuthor::new(self, name.into_content_name())
    }

    /// Begins a beginner-friendly door prefab.
    pub fn door_prefab(&mut self, name: impl IntoContentName) -> DoorPrefabAuthor<'_, 'app> {
        DoorPrefabAuthor::new(self, name.into_content_name())
    }

    /// Begins a non-solid area prefab that can drive enter/exit callbacks.
    pub fn area_prefab(&mut self, name: impl IntoContentName) -> AreaPrefabAuthor<'_, 'app> {
        AreaPrefabAuthor::new(self, name.into_content_name())
    }

    /// Alias for [`Self::area_prefab`].
    pub fn trigger_prefab(&mut self, name: impl IntoContentName) -> AreaPrefabAuthor<'_, 'app> {
        self.area_prefab(name)
    }

    /// Begins a non-solid checkpoint marker that rules can activate and use as
    /// a respawn position.
    pub fn checkpoint_prefab(&mut self, name: impl IntoContentName) -> AreaPrefabAuthor<'_, 'app> {
        AreaPrefabAuthor::new_checkpoint(self, name.into_content_name())
    }

    /// Begins a beginner-friendly projectile prefab.
    pub fn projectile_prefab(
        &mut self,
        name: impl IntoContentName,
    ) -> ProjectilePrefabAuthor<'_, 'app> {
        ProjectilePrefabAuthor::new(self, name.into_content_name())
    }

    /// Begins a beginner-friendly spawner prefab.
    pub fn spawner_prefab(&mut self, name: impl IntoContentName) -> SpawnerPrefabAuthor<'_, 'app> {
        SpawnerPrefabAuthor::new(self, name.into_content_name())
    }

    /// Begins configuring a beginner top-down game preset.
    pub fn use_top_down_game(&mut self) -> TopDownGameAuthor<'_, 'app> {
        TopDownGameAuthor::new(self)
    }

    /// Begins configuring declarative beginner rules.
    pub fn rules(&mut self) -> RulesAuthor<'_, 'app> {
        RulesAuthor::new(self)
    }

    /// Begins a concrete declarative custom rule.
    pub fn custom_rule(&mut self, name: impl Into<String>) -> CustomRuleAuthor<'_, 'app> {
        CustomRuleAuthor::new(self, name.into())
    }

    /// Begins declaring an in-code map.
    pub fn map(&mut self, name: impl IntoContentName) -> MapAuthor<'_, 'app> {
        MapAuthor::in_code(self, name.into_content_name())
    }

    /// Begins declaring a map from a legacy/advanced external RON document.
    pub fn map_from_ron(&mut self, ron: impl Into<String>) -> MapAuthor<'_, 'app> {
        MapAuthor::from_ron(self, ron.into())
    }

    /// Begins a beginner-friendly text map loaded from `assets/<path>`.
    pub fn map_from_text(
        &mut self,
        name: impl IntoContentName,
        path: impl Into<String>,
    ) -> MapAuthor<'_, 'app> {
        MapAuthor::from_text(self, name.into_content_name(), path.into())
    }

    /// Begins a text map named `<name>` from `assets/maps/<name>.txt`.
    pub fn map_from_text_auto(&mut self, name: impl IntoContentName) -> MapAuthor<'_, 'app> {
        let name = name.into_content_name();
        let path = format!("maps/{name}.txt");
        MapAuthor::from_text(self, name, path)
    }

    /// Begins a map imported from an LDtk project under `assets/<path>`.
    pub fn map_from_ldtk(
        &mut self,
        name: impl IntoContentName,
        path: impl Into<String>,
    ) -> MapAuthor<'_, 'app> {
        MapAuthor::from_ldtk(self, name.into_content_name(), path.into())
    }

    /// Begins a map imported from a Tiled TMX project under `assets/<path>`.
    /// The beginner importer supports an orthogonal TMX map with a CSV
    /// `Collision` layer and object mappings configured through `.object(...)`.
    pub fn map_from_tiled(
        &mut self,
        name: impl IntoContentName,
        path: impl Into<String>,
    ) -> MapAuthor<'_, 'app> {
        MapAuthor::from_tiled(self, name.into_content_name(), path.into())
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
        DebugOverlayAuthor::new(self)
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

    /// Registers a beginner startup callback with inferable arguments.
    pub fn on_start(
        &mut self,
        mut start: impl FnMut(&mut BeginnerStartupGame<'_, '_, '_>) -> Result<()> + 'static,
    ) {
        self.builder.schedule_mut().add_startup(move |ctx| {
            let mut ctx = StartupGameCtx::new(ctx);
            let mut game = BeginnerStartupGame::new(&mut ctx);
            start(&mut game)
        });
    }

    /// Registers a fixed-timestep system.
    pub fn fixed(&mut self, mut system: impl GameSystem) {
        self.builder.schedule_mut().add_fixed(move |ctx, dt| {
            let mut game = GameCtx::new(ctx);
            system.run(&mut game, dt);
        });
    }

    /// Registers an advanced fixed-timestep function whose arguments are
    /// extracted from the current frame, such as a component Query or Res<Input>.
    ///
    /// This path validates all declared component and resource borrows while the
    /// content plugin is built, before the schedule can run the system.
    pub fn fixed_params<Marker>(&mut self, mut system: impl ParamSystem<Marker>) -> Result<()> {
        system.validate_params()?;
        self.builder.schedule_mut().add_fixed(move |ctx, dt| {
            system.run_params(ctx.world, ctx.input, dt);
        });
        Ok(())
    }

    /// Registers a beginner fixed-timestep callback.
    ///
    /// Taking the callback's concrete signature, rather than only a trait
    /// bound, lets Rust infer `game` and `dt` for a closure such as
    /// `game.every_tick(|game, dt| { ... })`.
    pub fn every_tick(
        &mut self,
        mut system: impl FnMut(&mut BeginnerGame<'_, '_, '_>, Seconds) + 'static,
    ) {
        self.builder.schedule_mut().add_fixed(move |ctx, dt| {
            let mut ctx = GameCtx::new(ctx);
            let mut game = BeginnerGame::new(&mut ctx);
            system(&mut game, dt);
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

    /// Registers a beginner fixed-timestep callback while state `S` is active.
    ///
    /// Like [`Self::every_tick`], this accepts the concrete callback signature
    /// so content closures keep their arguments inferred.
    pub fn every_active_tick<S>(
        &mut self,
        mut system: impl FnMut(&mut BeginnerGame<'_, '_, '_>, Seconds) + 'static,
    ) where
        S: SimulationState + 'static,
    {
        self.fixed(move |game: &mut GameCtx<'_, '_>, dt| {
            if game.resource::<S>().is_some_and(SimulationState::active) {
                let mut game = BeginnerGame::new(game);
                system(&mut game, dt);
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

    /// Registers an advanced frame-rate-paced parameter system.
    pub fn update_params<Marker>(&mut self, mut system: impl ParamSystem<Marker>) -> Result<()> {
        system.validate_params()?;
        self.builder.schedule_mut().add_update(move |ctx, dt| {
            system.run_params(ctx.world, ctx.input, dt);
        });
        Ok(())
    }

    /// Registers a beginner per-frame callback with inferred closure arguments.
    pub fn every_frame(
        &mut self,
        mut system: impl FnMut(&mut BeginnerGame<'_, '_, '_>, Seconds) + 'static,
    ) {
        self.builder.schedule_mut().add_update(move |ctx, dt| {
            let mut ctx = GameCtx::new(ctx);
            let mut game = BeginnerGame::new(&mut ctx);
            system(&mut game, dt);
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

    /// Registers a beginner per-frame callback while state `S` is active.
    pub fn every_active_frame<S>(
        &mut self,
        mut system: impl FnMut(&mut BeginnerGame<'_, '_, '_>, Seconds) + 'static,
    ) where
        S: SimulationState + 'static,
    {
        self.update(move |game: &mut GameCtx<'_, '_>, dt| {
            if game.resource::<S>().is_some_and(SimulationState::active) {
                let mut game = BeginnerGame::new(game);
                system(&mut game, dt);
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

    /// Registers a beginner UI callback.
    ///
    /// This intentionally accepts the closure directly instead of going through
    /// [`GameSystem`]. That keeps the callback arguments inferable, so beginner
    /// code can write `game.draw_ui(|game, dt| { ... })` without exposing context
    /// lifetime annotations.
    pub fn draw_ui(
        &mut self,
        mut draw: impl FnMut(&mut BeginnerGame<'_, '_, '_>, Seconds) + 'static,
    ) {
        self.builder.schedule_mut().add_ui(move |ctx, dt| {
            let mut ctx = GameCtx::new(ctx);
            let mut game = BeginnerGame::new(&mut ctx);
            draw(&mut game, dt);
        });
    }

    /// Runs `f` on fixed ticks where `action` was pressed.
    pub fn on_action(
        &mut self,
        action: ActionId,
        mut f: impl FnMut(&mut BeginnerGame<'_, '_, '_>) + 'static,
    ) {
        self.fixed(move |game: &mut GameCtx<'_, '_>, _dt| {
            if game.pressed(action) {
                let mut game = BeginnerGame::new(game);
                f(&mut game);
            }
        });
    }

    /// Runs `f` on fixed ticks where `action` was pressed and the simple
    /// beginner state is active.
    pub fn on_action_when_playing(
        &mut self,
        action: ActionId,
        mut f: impl FnMut(&mut BeginnerGame<'_, '_, '_>) + 'static,
    ) {
        self.fixed_active::<SimpleGameState>(move |game: &mut GameCtx<'_, '_>, _dt| {
            if game.pressed(action) {
                let mut game = BeginnerGame::new(game);
                f(&mut game);
            }
        });
    }

    pub fn on_action_cooldown(
        &mut self,
        action: ActionId,
        seconds: f32,
        mut f: impl FnMut(&mut BeginnerGame<'_, '_, '_>) + 'static,
    ) {
        let seconds = seconds.max(0.0);
        let mut cooldown: f32 = 0.0;
        self.fixed(move |game: &mut GameCtx<'_, '_>, dt: f32| {
            cooldown = (cooldown - dt).max(0.0);
            if cooldown == 0.0 && game.pressed(action) {
                cooldown = seconds;
                let mut game = BeginnerGame::new(game);
                f(&mut game);
            }
        });
    }

    /// Runs every fixed tick while matching prefab colliders overlap. This only
    /// observes contact; it does not apply pickup, door, or damage behavior.
    pub fn on_collision(
        &mut self,
        a_prefab: impl Into<String>,
        b_prefab: impl Into<String>,
        mut f: impl FnMut(&mut CollisionEvent<'_, '_, '_>) + 'static,
    ) {
        let a_prefab = a_prefab.into();
        let b_prefab = b_prefab.into();
        self.fixed(move |game: &mut GameCtx<'_, '_>, _dt| {
            for (a, b) in matching_overlaps(game, &a_prefab, &b_prefab) {
                let mut event = CollisionEvent::new(game, ActorToken::new(a), ActorToken::new(b));
                f(&mut event);
            }
        });
    }

    /// Runs once when the matching actor starts overlapping the named area.
    pub fn on_enter_area(
        &mut self,
        actor_prefab: impl Into<String>,
        area_prefab: impl Into<String>,
        mut f: impl FnMut(&mut CollisionEvent<'_, '_, '_>) + 'static,
    ) {
        let actor_prefab = actor_prefab.into();
        let area_prefab = area_prefab.into();
        let event_key = format!("{actor_prefab}:{area_prefab}");
        let mut tracker = OverlapTracker::default();
        self.fixed(move |game: &mut GameCtx<'_, '_>, _dt| {
            let current = matching_overlaps(game, &actor_prefab, &area_prefab)
                .into_iter()
                .collect::<HashSet<_>>();
            for &(actor, area) in &current {
                if tracker.active.insert((actor, area, event_key.clone())) {
                    let mut event =
                        CollisionEvent::new(game, ActorToken::new(actor), ActorToken::new(area));
                    f(&mut event);
                }
            }
            tracker.active.retain(|entry| {
                entry.2.as_str() != event_key.as_str() || current.contains(&(entry.0, entry.1))
            });
        });
    }

    /// Runs once when the matching actor stops overlapping the named area.
    pub fn on_exit_area(
        &mut self,
        actor_prefab: impl Into<String>,
        area_prefab: impl Into<String>,
        mut f: impl FnMut(&mut CollisionEvent<'_, '_, '_>) + 'static,
    ) {
        let actor_prefab = actor_prefab.into();
        let area_prefab = area_prefab.into();
        let event_key = format!("{actor_prefab}:{area_prefab}");
        let mut tracker = OverlapTracker::default();
        self.fixed(move |game: &mut GameCtx<'_, '_>, _dt| {
            let current = matching_overlaps(game, &actor_prefab, &area_prefab)
                .into_iter()
                .collect::<HashSet<_>>();
            let exited = tracker
                .active
                .iter()
                .filter(|entry| {
                    entry.2.as_str() == event_key.as_str() && !current.contains(&(entry.0, entry.1))
                })
                .map(|entry| (entry.0, entry.1))
                .collect::<Vec<_>>();
            for (actor, area) in exited {
                tracker.active.remove(&(actor, area, event_key.clone()));
                let mut event =
                    CollisionEvent::new(game, ActorToken::new(actor), ActorToken::new(area));
                f(&mut event);
            }
            for (actor, area) in current {
                tracker.active.insert((actor, area, event_key.clone()));
            }
        });
    }

    pub fn every_seconds(
        &mut self,
        seconds: f32,
        mut f: impl FnMut(&mut BeginnerGame<'_, '_, '_>) + 'static,
    ) {
        let seconds = seconds.max(MIN_TIMER_SECONDS);
        let mut timer = 0.0;
        self.fixed(move |game: &mut GameCtx<'_, '_>, dt| {
            timer += dt;
            while timer >= seconds {
                timer -= seconds;
                let mut game = BeginnerGame::new(game);
                f(&mut game);
            }
        });
    }

    pub fn after_seconds(
        &mut self,
        seconds: f32,
        mut f: impl FnMut(&mut BeginnerGame<'_, '_, '_>) + 'static,
    ) {
        let seconds = seconds.max(0.0);
        let mut timer = 0.0;
        let mut done = false;
        self.fixed(move |game: &mut GameCtx<'_, '_>, dt| {
            if done {
                return;
            }
            timer += dt;
            if timer >= seconds {
                done = true;
                let mut game = BeginnerGame::new(game);
                f(&mut game);
            }
        });
    }

    pub fn on_player_death(&mut self, mut f: impl FnMut(&mut BeginnerGame<'_, '_, '_>) + 'static) {
        let mut was_dead = false;
        self.fixed(move |game: &mut GameCtx<'_, '_>, _dt| {
            let is_dead = game.player_id().is_some_and(|id| game.is_dead(id));
            if is_dead && !was_dead {
                let mut game = BeginnerGame::new(game);
                f(&mut game);
            }
            was_dead = is_dead;
        });
    }

    pub fn on_player_respawn(
        &mut self,
        mut f: impl FnMut(&mut BeginnerGame<'_, '_, '_>) + 'static,
    ) {
        let mut was_dead = false;
        self.fixed(move |game: &mut GameCtx<'_, '_>, _dt| {
            let is_dead = game.player_id().is_some_and(|id| game.is_dead(id));
            let is_alive = game.player_id().is_some_and(|id| !game.is_dead(id));
            if was_dead && is_alive {
                let mut game = BeginnerGame::new(game);
                f(&mut game);
            }
            was_dead = is_dead;
        });
    }

    pub fn on_score_reaches(
        &mut self,
        score: i32,
        mut f: impl FnMut(&mut BeginnerGame<'_, '_, '_>) + 'static,
    ) {
        let mut fired = false;
        self.fixed(move |game: &mut GameCtx<'_, '_>, _dt| {
            if fired {
                return;
            }
            let current = game.score().value();
            if current >= score {
                fired = true;
                let mut game = BeginnerGame::new(game);
                f(&mut game);
            }
        });
    }

    pub fn on_wave_cleared(&mut self, mut f: impl FnMut(&mut BeginnerGame<'_, '_, '_>) + 'static) {
        let mut had_living_enemies = false;
        self.fixed(move |game: &mut GameCtx<'_, '_>, _dt| {
            let has_living_enemies = !game.living_enemy_ids().is_empty();
            if had_living_enemies && !has_living_enemies {
                let mut game = BeginnerGame::new(game);
                f(&mut game);
            }
            had_living_enemies = has_living_enemies;
        });
    }

    pub fn on_timer(
        &mut self,
        name: impl Into<String>,
        seconds: f32,
        mut f: impl FnMut(&mut BeginnerGame<'_, '_, '_>) + 'static,
    ) {
        let _timer_name = name.into();
        let seconds = seconds.max(0.0);
        let mut timer = 0.0;
        let mut done = false;
        self.fixed(move |game: &mut GameCtx<'_, '_>, dt| {
            if done {
                return;
            }
            timer += dt;
            if timer >= seconds {
                done = true;
                let mut game = BeginnerGame::new(game);
                f(&mut game);
            }
        });
    }

    pub fn every_seconds_while_playing(
        &mut self,
        seconds: f32,
        mut f: impl FnMut(&mut BeginnerGame<'_, '_, '_>) + 'static,
    ) {
        let seconds = seconds.max(MIN_TIMER_SECONDS);
        let mut timer = 0.0;
        self.fixed_active::<SimpleGameState>(move |game: &mut GameCtx<'_, '_>, dt| {
            timer += dt;
            while timer >= seconds {
                timer -= seconds;
                let mut game = BeginnerGame::new(game);
                f(&mut game);
            }
        });
    }

    pub fn on_enemy_death(&mut self, mut f: impl FnMut(&mut BeginnerGame<'_, '_, '_>) + 'static) {
        let mut known_dead: Vec<EntityId> = Vec::new();
        self.fixed(move |game: &mut GameCtx<'_, '_>, _dt| {
            let dead = game
                .enemy_ids()
                .into_iter()
                .filter(|id| game.is_dead(*id))
                .collect::<Vec<_>>();
            for id in &dead {
                if !known_dead.contains(id) {
                    let mut game = BeginnerGame::new(game);
                    f(&mut game);
                }
            }
            known_dead = dead;
        });
    }

    /// Runs once for each enemy that has just died, supplying an object-shaped
    /// event instead of exposing a world context or entity id.
    pub fn on_enemy_death_event(
        &mut self,
        mut f: impl FnMut(&mut EnemyDeathEvent<'_, '_, '_>) + 'static,
    ) {
        let mut known_dead: Vec<EntityId> = Vec::new();
        self.fixed(move |game: &mut GameCtx<'_, '_>, _dt| {
            let dead = game
                .enemy_ids()
                .into_iter()
                .filter(|id| game.is_dead(*id))
                .collect::<Vec<_>>();
            for id in &dead {
                if !known_dead.contains(id) {
                    let mut event = EnemyDeathEvent::new(game, ActorToken::new(*id));
                    f(&mut event);
                }
            }
            known_dead = dead;
        });
    }

    pub fn on_projectile_hit(
        &mut self,
        projectile_prefab: impl Into<String>,
        enemy_prefab: impl Into<String>,
        mut f: impl FnMut(&mut ProjectileHitEvent<'_, '_, '_>) + 'static,
    ) {
        let projectile_prefab = projectile_prefab.into();
        let enemy_prefab = enemy_prefab.into();
        let event_key = format!("{projectile_prefab}:{enemy_prefab}");
        let mut tracker = OverlapTracker::default();
        self.fixed(move |game: &mut GameCtx<'_, '_>, _dt| {
            let current = matching_nearby_prefabs(
                game,
                &projectile_prefab,
                &enemy_prefab,
                DEFAULT_PROJECTILE_HIT_RANGE,
            )
            .into_iter()
            .map(|(projectile, enemy, _)| (projectile, enemy))
            .collect::<HashSet<_>>();

            for &(projectile, enemy) in &current {
                if tracker
                    .active
                    .insert((projectile, enemy, event_key.clone()))
                {
                    let position = game
                        .position(projectile)
                        .or_else(|| game.position(enemy))
                        .unwrap_or(Vec2::ZERO);
                    let mut event = ProjectileHitEvent::new(
                        game,
                        ActorToken::new(projectile),
                        ActorToken::new(enemy),
                        position,
                    );
                    f(&mut event);
                }
            }
            tracker.active.retain(|entry| {
                entry.2.as_str() != event_key.as_str() || current.contains(&(entry.0, entry.1))
            });
        });
    }

    /// Runs for matching player/pickup collection interactions. Both filters are
    /// source prefab names registered by the beginner builders.
    pub fn on_collect(
        &mut self,
        collector_prefab: impl Into<String>,
        pickup_prefab: impl Into<String>,
        mut f: impl FnMut(&mut CollectEvent<'_, '_, '_>) + 'static,
    ) {
        let collector_prefab = collector_prefab.into();
        let pickup_prefab = pickup_prefab.into();
        self.fixed(move |game: &mut GameCtx<'_, '_>, _dt| {
            let Some(collector) = game.player_id() else {
                return;
            };
            if !prefab_matches(game, collector, &collector_prefab) {
                return;
            }

            let pickups = game
                .pickup_ids_near_player(DEFAULT_PICKUP_COLLECT_RANGE)
                .into_iter()
                .filter(|pickup| prefab_matches(game, *pickup, &pickup_prefab))
                .collect::<Vec<_>>();
            for pickup in pickups {
                if game.collect_pickup(pickup) {
                    let mut event = CollectEvent::new(
                        game,
                        ActorToken::new(collector),
                        ActorToken::new(pickup),
                    );
                    f(&mut event);
                }
            }
        });
    }

    pub fn on_player_collect_pickup(
        &mut self,
        f: impl FnMut(&mut BeginnerGame<'_, '_, '_>) + 'static,
    ) {
        self.on_player_collect_pickup_within(DEFAULT_PICKUP_COLLECT_RANGE, f);
    }

    /// Runs for player/pickup overlap with an object-shaped collision event.
    /// Unlike [`Self::on_collect`], this only observes touching; it does not
    /// apply pickup score, sound, or despawn effects.
    pub fn on_player_touching_pickup(
        &mut self,
        mut f: impl FnMut(&mut CollisionEvent<'_, '_, '_>) + 'static,
    ) {
        self.fixed(move |game: &mut GameCtx<'_, '_>, _dt| {
            let Some(player) = game.player_id() else {
                return;
            };
            for pickup in game.pickup_ids_near_player(DEFAULT_PICKUP_COLLECT_RANGE) {
                let mut event =
                    CollisionEvent::new(game, ActorToken::new(player), ActorToken::new(pickup));
                f(&mut event);
            }
        });
    }

    pub fn on_player_collect_pickup_within(
        &mut self,
        range: f32,
        mut f: impl FnMut(&mut BeginnerGame<'_, '_, '_>) + 'static,
    ) {
        self.fixed(move |game: &mut GameCtx<'_, '_>, _dt| {
            if game.collect_pickups_near_player(range) > 0 {
                let mut game = BeginnerGame::new(game);
                f(&mut game);
            }
        });
    }

    pub fn on_door_open(
        &mut self,
        door_prefab: impl Into<String>,
        mut f: impl FnMut(&mut DoorEvent<'_, '_, '_>) + 'static,
    ) {
        let door_prefab = door_prefab.into();
        let event_key = format!("door:{door_prefab}");
        let mut tracker = OverlapTracker::default();
        self.fixed(move |game: &mut GameCtx<'_, '_>, _dt| {
            let Some(player) = game.player_id() else {
                return;
            };
            let Some(player_pos) = game.position(player) else {
                return;
            };
            let living_enemies = game.living_enemy_ids().len();
            let current = game
                .entities_with::<Door>()
                .into_iter()
                .filter(|door| prefab_matches(game, *door, &door_prefab))
                .filter(|door| {
                    let Some(door_pos) = game.position(*door) else {
                        return false;
                    };
                    if door_pos.distance(player_pos) > DEFAULT_DOOR_TRIGGER_RANGE {
                        return false;
                    }
                    game.component::<DoorTarget>(*door).is_none_or(|target| {
                        !target.requires_all_enemies_dead || living_enemies == 0
                    })
                })
                .map(|door| (player, door))
                .collect::<HashSet<_>>();

            for &(player, door) in &current {
                if tracker.active.insert((player, door, event_key.clone())) {
                    let mut event =
                        DoorEvent::new(game, ActorToken::new(player), ActorToken::new(door));
                    f(&mut event);
                }
            }
            tracker.active.retain(|entry| {
                entry.2.as_str() != event_key.as_str() || current.contains(&(entry.0, entry.1))
            });
        });
    }

    /// Runs for player/door overlap with an object-shaped collision event. The
    /// event observes contact only; map-changing behavior remains opt-in through
    /// `game.rules().doors_change_maps()`.
    pub fn on_player_touching_door(
        &mut self,
        mut f: impl FnMut(&mut CollisionEvent<'_, '_, '_>) + 'static,
    ) {
        const DOOR_TOUCH_RANGE: f32 = 28.0;
        self.fixed(move |game: &mut GameCtx<'_, '_>, _dt| {
            let Some(player) = game.player_id() else {
                return;
            };
            let Some(position) = game.position(player) else {
                return;
            };
            let doors = game
                .entities_with::<Door>()
                .into_iter()
                .filter(|door| {
                    game.position(*door)
                        .is_some_and(|door_pos| door_pos.distance(position) <= DOOR_TOUCH_RANGE)
                })
                .collect::<Vec<_>>();
            for door in doors {
                let mut event =
                    CollisionEvent::new(game, ActorToken::new(player), ActorToken::new(door));
                f(&mut event);
            }
        });
    }

    /// Runs when a matching non-looping animation reaches its final frame.
    /// The callback receives a safe actor wrapper rather than an entity id.
    pub fn on_animation_finished(
        &mut self,
        name: impl Into<String>,
        mut f: impl FnMut(&mut AnimationFinishedEvent<'_, '_, '_>) + 'static,
    ) {
        let name = name.into();
        let mut last_seen = 0u64;
        self.update(move |game: &mut GameCtx<'_, '_>, _dt| {
            let records = game
                .resource::<AnimationFinishedEvents>()
                .map(|events| {
                    events
                        .after(last_seen)
                        .map(|record| (record.sequence, record.entity, record.name.clone()))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            for (sequence, entity, event_name) in records {
                last_seen = last_seen.max(sequence);
                if event_name == name {
                    let mut event =
                        AnimationFinishedEvent::new(game, ActorToken::new(entity), event_name);
                    f(&mut event);
                }
            }
        });
    }

    pub fn on_scene_enter(
        &mut self,
        scene: impl Into<String>,
        mut f: impl FnMut(&mut BeginnerGame<'_, '_, '_>) + 'static,
    ) {
        let scene = scene.into();
        let mut was_in_scene = false;
        self.update(move |game: &mut GameCtx<'_, '_>, _dt| {
            let is_in_scene = game.current_scene_name().as_deref() == Some(scene.as_str());
            if is_in_scene && !was_in_scene {
                let mut game = BeginnerGame::new(game);
                f(&mut game);
            }
            was_in_scene = is_in_scene;
        });
    }

    pub fn on_scene(
        &mut self,
        scene: impl Into<String>,
        mut system: impl FnMut(&mut BeginnerGame<'_, '_, '_>, Seconds) + 'static,
    ) {
        let scene = scene.into();
        self.update(move |game: &mut GameCtx<'_, '_>, dt| {
            if game.current_scene_name().as_deref() == Some(scene.as_str()) {
                let mut game = BeginnerGame::new(game);
                system(&mut game, dt);
            }
        });
    }

    pub fn on_map_enter(
        &mut self,
        map: impl Into<String>,
        mut f: impl FnMut(&mut BeginnerGame<'_, '_, '_>) + 'static,
    ) {
        let map = map.into();
        let mut was_in_map = false;
        self.update(move |game: &mut GameCtx<'_, '_>, _dt| {
            let is_in_map = game.current_map_name().as_deref() == Some(map.as_str());
            if is_in_map && !was_in_map {
                let mut game = BeginnerGame::new(game);
                f(&mut game);
            }
            was_in_map = is_in_map;
        });
    }

    pub fn on_map_exit(
        &mut self,
        map: impl Into<String>,
        mut f: impl FnMut(&mut BeginnerGame<'_, '_, '_>) + 'static,
    ) {
        let map = map.into();
        let mut was_in_map = false;
        self.update(move |game: &mut GameCtx<'_, '_>, _dt| {
            let is_in_map = game.current_map_name().as_deref() == Some(map.as_str());
            if was_in_map && !is_in_map {
                let mut game = BeginnerGame::new(game);
                f(&mut game);
            }
            was_in_map = is_in_map;
        });
    }

    pub fn on_map_changed(
        &mut self,
        mut f: impl FnMut(&mut MapChangedEvent<'_, '_, '_>) + 'static,
    ) {
        let mut initialized = false;
        let mut previous: Option<String> = None;
        self.update(move |game: &mut GameCtx<'_, '_>, _dt| {
            let current = game.current_map_name();
            if initialized && current != previous {
                let mut event = MapChangedEvent::new(game, previous.clone(), current.clone());
                f(&mut event);
            }
            initialized = true;
            previous = current;
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
    pub(crate) fn finish_for_reload(self) -> Result<ContentRuntime> {
        let content = Rc::clone(&self.content);
        self.finish()?;
        content
            .borrow_mut()
            .take()
            .ok_or_else(|| anyhow!("beginner file reload did not produce content runtime"))
    }

    pub(crate) fn finish(mut self) -> Result<()> {
        let mut maps: HashMap<String, GameMap> = HashMap::new();
        let mut map_ids: HashMap<String, game_core::builder::MapId> = HashMap::new();
        let mut themes: HashMap<String, game_core::app::TileTheme> = HashMap::new();
        let mut start_map_name: Option<String> = None;
        let mut text_maps = HashMap::new();

        for pending in std::mem::take(&mut self.pending_maps) {
            let (game_map, theme, start, reload_source) =
                pending.resolve(self.builder.prefabs())?;
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
            if let Some(reload_source) = reload_source {
                text_maps.insert(game_map.name.clone(), reload_source);
            }
            themes.insert(game_map.name.clone(), theme);
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
        *self.content.borrow_mut() = Some(ContentRuntime::new(
            prefabs, maps, map_ids, themes, text_maps, start_map,
        ));
        let asset_count = self.builder.assets().texture_keys().count()
            + self.builder.assets().sound_keys().count()
            + self.builder.assets().font_keys().count();
        *self.asset_lookup.borrow_mut() = Some(AssetLookup::from_registry(self.builder.assets()));

        if self.builder.schedule().has_fixed_systems() {
            self.builder
                .schedule_mut()
                .add_fixed(|ctx, _dt| drain_beginner_spawn_queue(ctx.world));
        }
        self.builder
            .schedule_mut()
            .add_update(|ctx, _dt| drain_beginner_spawn_queue(ctx.world));

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
            let iteration_info = DebugIterationInfo::new(asset_count);
            self.builder.schedule_mut().add_startup(move |ctx| {
                ctx.world.insert_resource(overlay);
                ctx.world.insert_resource(iteration_info.clone());
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

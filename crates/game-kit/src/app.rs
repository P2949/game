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
use game_core::world::{EntityId, Transform};
use game_map::GameMap;
use game_physics::Collider;

use crate::assets::{
    AssetAuthor, AssetBagAuthor, AssetFolderAuthor, AssetLookup, SoundRef, TextureRef,
    missing_asset_error,
};
use crate::beginner::actors::{Door, PrefabName};
use crate::beginner::animation::AnimationFinishedEvents;
use crate::beginner::debug::{DebugIterationInfo, DebugOverlay, draw_debug_overlay};
use crate::beginner::defaults::TopDownGameAuthor;
use crate::beginner::events::{
    ActorToken, AnimationFinishedEvent, CollectEvent, CollisionEvent, DEFAULT_PICKUP_COLLECT_RANGE,
    EnemyDeathEvent, OverlapTracker,
};
use crate::beginner::prefabs::{
    AreaPrefabAuthor, DoorPrefabAuthor, EnemyPrefabAuthor, PickupPrefabAuthor, PlayerPrefabAuthor,
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

    /// Begins a non-solid area prefab that can drive enter/exit callbacks.
    pub fn area_prefab(&mut self, name: impl Into<String>) -> AreaPrefabAuthor<'_, 'app> {
        AreaPrefabAuthor::new(self, name.into())
    }

    /// Alias for [`Self::area_prefab`].
    pub fn trigger_prefab(&mut self, name: impl Into<String>) -> AreaPrefabAuthor<'_, 'app> {
        self.area_prefab(name)
    }

    /// Begins a non-solid checkpoint marker that rules can activate and use as
    /// a respawn position.
    pub fn checkpoint_prefab(&mut self, name: impl Into<String>) -> AreaPrefabAuthor<'_, 'app> {
        AreaPrefabAuthor::new_checkpoint(self, name.into())
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

    /// Begins a beginner-friendly text map loaded from `assets/<path>`.
    pub fn map_from_text(
        &mut self,
        name: impl Into<String>,
        path: impl Into<String>,
    ) -> MapAuthor<'_, 'app> {
        MapAuthor::from_text(self, name.into(), path.into())
    }

    /// Begins a text map named `<name>` from `assets/maps/<name>.txt`.
    pub fn map_from_text_auto(&mut self, name: impl Into<String>) -> MapAuthor<'_, 'app> {
        let name = name.into();
        let path = format!("maps/{name}.txt");
        MapAuthor::from_text(self, name, path)
    }

    /// Begins a map imported from an LDtk project under `assets/<path>`.
    pub fn map_from_ldtk(
        &mut self,
        name: impl Into<String>,
        path: impl Into<String>,
    ) -> MapAuthor<'_, 'app> {
        MapAuthor::from_ldtk(self, name.into(), path.into())
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

    /// Registers a beginner startup callback with inferable arguments.
    pub fn on_start(
        &mut self,
        mut start: impl FnMut(&mut StartupGameCtx<'_, '_>) -> Result<()> + 'static,
    ) {
        self.builder.schedule_mut().add_startup(move |ctx| {
            let mut game = StartupGameCtx::new(ctx);
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

    /// Registers a beginner UI callback.
    ///
    /// This intentionally accepts the closure directly instead of going through
    /// [`GameSystem`]. That keeps the callback arguments inferable, so beginner
    /// code can write `game.draw_ui(|game, dt| { ... })` without exposing context
    /// lifetime annotations.
    pub fn draw_ui(&mut self, mut draw: impl FnMut(&mut GameCtx<'_, '_>, f32) + 'static) {
        self.builder.schedule_mut().add_ui(move |ctx, dt| {
            let mut game = GameCtx::new(ctx);
            draw(&mut game, dt);
        });
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
        self.every_tick(move |game: &mut GameCtx<'_, '_>, _dt| {
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
        self.every_tick(move |game: &mut GameCtx<'_, '_>, _dt| {
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
        self.every_tick(move |game: &mut GameCtx<'_, '_>, _dt| {
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

    /// Runs once for each enemy that has just died, supplying an object-shaped
    /// event instead of exposing a world context or entity id.
    pub fn on_enemy_death_event(
        &mut self,
        mut f: impl FnMut(&mut EnemyDeathEvent<'_, '_, '_>) + 'static,
    ) {
        let mut known_dead: Vec<EntityId> = Vec::new();
        self.every_tick(move |game: &mut GameCtx<'_, '_>, _dt| {
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
        self.every_tick(move |game: &mut GameCtx<'_, '_>, _dt| {
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

    pub fn on_player_collect_pickup(&mut self, f: impl FnMut(&mut GameCtx<'_, '_>) + 'static) {
        self.on_player_collect_pickup_within(DEFAULT_PICKUP_COLLECT_RANGE, f);
    }

    /// Runs for player/pickup overlap with an object-shaped collision event.
    /// Unlike [`Self::on_collect`], this only observes touching; it does not
    /// apply pickup score, sound, or despawn effects.
    pub fn on_player_touching_pickup(
        &mut self,
        mut f: impl FnMut(&mut CollisionEvent<'_, '_, '_>) + 'static,
    ) {
        self.every_tick(move |game: &mut GameCtx<'_, '_>, _dt| {
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
        mut f: impl FnMut(&mut GameCtx<'_, '_>) + 'static,
    ) {
        self.every_tick(move |game: &mut GameCtx<'_, '_>, _dt| {
            if game.collect_pickups_near_player(range) > 0 {
                f(game);
            }
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
        self.every_tick(move |game: &mut GameCtx<'_, '_>, _dt| {
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
        self.every_frame(move |game: &mut GameCtx<'_, '_>, _dt| {
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
            prefabs, maps, map_ids, text_maps, start_map,
        ));
        let asset_count = self.builder.assets().texture_keys().count()
            + self.builder.assets().sound_keys().count()
            + self.builder.assets().font_keys().count();
        *self.asset_lookup.borrow_mut() = Some(AssetLookup::from_registry(self.builder.assets()));

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

fn prefab_matches(game: &GameCtx<'_, '_>, entity: EntityId, expected: &str) -> bool {
    game.component::<PrefabName>(entity)
        .is_some_and(|name| name.matches(expected))
}

fn matching_overlaps(
    game: &GameCtx<'_, '_>,
    a_prefab: &str,
    b_prefab: &str,
) -> Vec<(EntityId, EntityId)> {
    let a_entities = game
        .entities_with::<PrefabName>()
        .into_iter()
        .filter(|entity| prefab_matches(game, *entity, a_prefab))
        .collect::<Vec<_>>();
    let b_entities = game
        .entities_with::<PrefabName>()
        .into_iter()
        .filter(|entity| prefab_matches(game, *entity, b_prefab))
        .collect::<Vec<_>>();
    let mut overlaps = Vec::new();

    for a in a_entities {
        for &b in &b_entities {
            if a != b && colliders_overlap(game, a, b) {
                overlaps.push((a, b));
            }
        }
    }
    overlaps
}

fn colliders_overlap(game: &GameCtx<'_, '_>, a: EntityId, b: EntityId) -> bool {
    let Some(a_transform) = game.component::<Transform>(a) else {
        return false;
    };
    let Some(a_collider) = game.component::<Collider>(a) else {
        return false;
    };
    let Some(b_transform) = game.component::<Transform>(b) else {
        return false;
    };
    let Some(b_collider) = game.component::<Collider>(b) else {
        return false;
    };

    let delta = a_transform.pos - b_transform.pos;
    delta.x.abs() < a_collider.half_extents.x + b_collider.half_extents.x
        && delta.y.abs() < a_collider.half_extents.y + b_collider.half_extents.y
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

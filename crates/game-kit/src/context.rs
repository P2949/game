//! Content-facing system contexts (Phase 8).
//!
//! [`GameCtx`] wraps the engine's per-step `Ctx` and [`StartupGameCtx`] wraps the
//! startup `StartCtx`, exposing readable helpers for input, camera, world,
//! resources, UI text, audio, commands, and map services — so content systems
//! never name `Ctx`, `StartCtx`, raw `World`, or `CommandQueue`.

use anyhow::Result;
use game_combat::{Faction, FactionId, Health, MeleeAttack};
use game_core::app::{Ctx, StartCtx};
use game_core::backend::SoundHandle;
use game_core::builder::{PrefabId, PropertyBag};
use game_core::camera::Camera2D;
use game_core::commands::{CommandQueue, MapReload};
use game_core::input::{ActionId, Axis2dId, Input, MouseButton};
use game_core::world::{Component, EntityId, Transform, Velocity};
use game_map::MapCell;
use glam::{Vec2, Vec4};

use crate::assets::AssetLookup;
use crate::beginner::audio::AudioOps;
use crate::beginner::debug::DebugIterationInfo;
use crate::beginner::tuning::TuningFile;
use crate::helpers::{InputDriven, MovementSpeed};
use crate::map::{
    ContentRuntime, change_to_map_world, reset_to_start_map_world, restart_current_map_world,
    restart_start_map_world,
};

/// Per-step context handed to fixed/update/render/ui systems.
pub struct GameCtx<'a, 'w> {
    inner: &'a mut Ctx<'w>,
}

impl<'a, 'w> GameCtx<'a, 'w> {
    pub(crate) fn new(inner: &'a mut Ctx<'w>) -> Self {
        Self { inner }
    }

    /// Resolved input for this step (logical actions and axes).
    pub fn input(&self) -> &Input {
        self.inner.input
    }

    /// The active 2D camera.
    pub fn camera(&self) -> &Camera2D {
        self.inner.camera
    }

    /// The active 2D camera, mutably (e.g. to follow the player or zoom).
    pub fn camera_mut(&mut self) -> &mut Camera2D {
        self.inner.camera
    }

    /// True if `action` was pressed this frame.
    pub fn pressed(&self, action: ActionId) -> bool {
        self.input().pressed(action)
    }

    /// True while `action` is held.
    pub fn down(&self, action: ActionId) -> bool {
        self.input().down(action)
    }

    /// Current value of a logical 2D axis.
    pub fn axis2d(&self, axis: Axis2dId) -> Vec2 {
        self.input().axis2d(axis)
    }

    /// Current mouse position in window pixels.
    pub fn mouse_position(&self) -> Vec2 {
        self.input().mouse_position()
    }

    /// True on the current frame when a physical mouse button was pressed.
    /// Beginner gameplay should normally bind actions instead; this is exposed
    /// for immediate-mode UI buttons.
    pub fn mouse_pressed(&self, button: MouseButton) -> bool {
        self.input().mouse_pressed(button)
    }

    /// Current mouse position in world coordinates, using the active 2D camera.
    pub fn mouse_world_position(&self) -> Vec2 {
        let viewport = self.input().viewport_size();
        if viewport.x <= 0.0 || viewport.y <= 0.0 {
            return self.camera().center();
        }
        let centered = self.input().mouse_position() - viewport * 0.5;
        self.camera().center() + centered / self.camera().zoom()
    }

    /// Queues UI text for this frame.
    pub fn text(&mut self, text: &str, pos: Vec2, color: Vec4) {
        self.inner.gfx.text(text, pos, color);
    }

    /// Queues a screen-space UI rectangle. High-level panels and buttons use
    /// this internally; ordinary beginner content can keep using `game.ui()`.
    pub fn ui_rect(&mut self, pos: Vec2, size: Vec2, color: Vec4) {
        self.ui_rect_at_layer(pos, size, color, 9_900);
    }

    /// Internal layering hook for the high-level immediate-mode UI helpers.
    /// Content should use [`Self::ui`] rather than selecting renderer layers.
    pub(crate) fn ui_rect_at_layer(&mut self, pos: Vec2, size: Vec2, color: Vec4, layer: i16) {
        self.inner.gfx.ui_rect(pos, size, color, layer);
    }

    /// Plays a sound effect immediately (bypassing the command queue).
    pub fn play_sound(&mut self, sound: SoundHandle, volume: f32) {
        self.inner.audio.play(sound, volume);
    }

    /// Plays a sound effect through a named mix bus. The ordinary SFX volume
    /// still applies, while the bus provides an extra per-category control.
    pub fn play_sound_on_bus(&mut self, sound: SoundHandle, volume: f32, bus: &str) {
        self.inner.audio.play_on_bus(sound, volume, bus);
    }

    /// Starts looping music immediately, replacing any currently playing music.
    pub fn play_music(&mut self, sound: SoundHandle, volume: f32) {
        self.inner.audio.play_music(sound, volume);
    }

    /// Starts looping music with a fade from silence, replacing current music.
    pub fn play_music_fade_in(&mut self, sound: SoundHandle, volume: f32, duration_seconds: f32) {
        self.inner
            .audio
            .play_music_fade_in(sound, volume, duration_seconds);
    }

    /// Stops currently playing music.
    pub fn stop_music(&mut self) {
        self.inner.audio.stop_music();
    }

    /// Blends the currently playing music into `sound` over `duration_seconds`.
    pub fn crossfade_music(&mut self, sound: SoundHandle, volume: f32, duration_seconds: f32) {
        self.inner
            .audio
            .crossfade_music(sound, volume, duration_seconds);
    }

    pub fn pause_music(&mut self) {
        self.inner.audio.pause_music();
    }

    pub fn resume_music(&mut self) {
        self.inner.audio.resume_music();
    }

    pub fn set_master_volume(&mut self, volume: f32) {
        self.inner.audio.set_master_volume(volume);
    }

    pub fn set_sfx_volume(&mut self, volume: f32) {
        self.inner.audio.set_sfx_volume(volume);
    }

    pub fn set_music_volume(&mut self, volume: f32) {
        self.inner.audio.set_music_volume(volume);
    }

    pub fn set_bus_volume(&mut self, bus: &str, volume: f32) {
        self.inner.audio.set_bus_volume(bus, volume);
    }

    pub fn fade_music_to(&mut self, volume: f32, duration_seconds: f32) {
        self.inner.audio.fade_music_to(volume, duration_seconds);
    }

    /// Beginner-friendly named audio operations.
    pub fn audio(&mut self) -> AudioOps<'_, 'a, 'w> {
        AudioOps::new(self)
    }

    /// Plays a registered sound effect by key at the standard beginner volume.
    ///
    /// A missing key is reported with the same known-key diagnostic as
    /// author-time asset lookups; it does not crash a running game.
    pub fn play_sound_named(&mut self, key: &str) {
        let sound = self
            .resource::<AssetLookup>()
            .and_then(|lookup| lookup.sound(key));
        match sound {
            Some(sound) => self.play_sound(sound, 1.0),
            None => self.report_missing_sound(key),
        }
    }

    /// Starts registered music by key at the standard beginner volume.
    pub fn play_music_named(&mut self, key: &str) {
        let sound = self
            .resource::<AssetLookup>()
            .and_then(|lookup| lookup.sound(key));
        match sound {
            Some(sound) => self.play_music(sound, 1.0),
            None => self.report_missing_sound(key),
        }
    }

    fn report_missing_sound(&self, key: &str) {
        if let Some(lookup) = self.resource::<AssetLookup>() {
            eprintln!("{}", lookup.sound_error(key));
        } else {
            eprintln!("Unknown sound asset '{key}'.\n\nNo asset lookup is installed.");
        }
    }

    pub(crate) fn named_sound(&self, key: &str) -> Option<SoundHandle> {
        self.resource::<AssetLookup>()
            .and_then(|lookup| lookup.sound(key))
    }

    pub(crate) fn report_missing_named_sound(&self, key: &str) {
        self.report_missing_sound(key);
    }

    /// Deferred runtime commands applied after the current step: despawn, play
    /// sound, spawn prefab, and map flow commands.
    pub fn commands(&mut self) -> Commands<'_> {
        Commands {
            queue: self.inner.world.resource_or_insert_with(CommandQueue::new),
        }
    }

    /// Reads a world resource.
    pub fn resource<T: 'static>(&self) -> Option<&T> {
        self.inner.world.get_resource::<T>()
    }

    /// Mutates a world resource.
    pub fn resource_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.inner.world.get_resource_mut::<T>()
    }

    /// Inserts a world resource, returning any previous value.
    pub fn insert_resource<T: 'static>(&mut self, value: T) -> Option<T> {
        self.inner.world.insert_resource(value)
    }

    /// Reads a world resource, inserting a default first if absent.
    pub fn resource_or_insert_with<T: 'static>(&mut self, create: impl FnOnce() -> T) -> &mut T {
        self.inner.world.resource_or_insert_with(create)
    }

    /// Live entity ids carrying component `T`.
    pub fn entities_with<T: Component>(&self) -> Vec<EntityId> {
        self.inner.world.ids_with::<T>()
    }

    /// Live entity ids carrying component `T` and matching `predicate`.
    pub fn entities_where<T: Component>(
        &self,
        mut predicate: impl FnMut(EntityId, &T) -> bool,
    ) -> Vec<EntityId> {
        self.entities_with::<T>()
            .into_iter()
            .filter(|id| {
                self.component::<T>(*id)
                    .is_some_and(|component| predicate(*id, component))
            })
            .collect()
    }

    /// Reads component `T` from `id`.
    pub fn component<T: Component>(&self, id: EntityId) -> Option<&T> {
        self.inner.world.get::<T>(id)
    }

    /// Mutates component `T` on `id`.
    pub fn component_mut<T: Component>(&mut self, id: EntityId) -> Option<&mut T> {
        self.inner.world.get_mut::<T>(id)
    }

    /// Inserts or replaces an internal beginner component for a state change
    /// such as a projectile entering an impact animation.
    pub(crate) fn insert_component<T: Component>(&mut self, id: EntityId, component: T) {
        self.inner.world.insert(id, component);
    }

    /// True when `id` has component `T`.
    pub fn has<T: Component>(&self, id: EntityId) -> bool {
        self.inner.world.has::<T>(id)
    }

    /// Entity world position.
    pub fn position(&self, id: EntityId) -> Option<Vec2> {
        self.component::<Transform>(id)
            .map(|transform| transform.pos)
    }

    pub fn cell_center(&self, cell: MapCell) -> Vec2 {
        self.inner.map.cell_center(cell.col(), cell.row())
    }

    pub fn first_floor_center(&self) -> Option<Vec2> {
        for row in 0..self.inner.map.height() {
            for col in 0..self.inner.map.width() {
                if !self.inner.map.is_wall(col as i32, row as i32) {
                    return Some(self.inner.map.cell_center(col, row));
                }
            }
        }
        None
    }

    pub fn nearest_by_position<T: Component>(
        &self,
        origin: Vec2,
        max_distance: f32,
        mut predicate: impl FnMut(EntityId) -> bool,
    ) -> Option<EntityId> {
        let max_distance_sq = max_distance * max_distance;

        self.entities_with::<T>()
            .into_iter()
            .filter(|id| predicate(*id))
            .filter_map(|id| {
                let position = self.position(id)?;
                let dist_sq = position.distance_squared(origin);
                (dist_sq <= max_distance_sq).then_some((id, dist_sq))
            })
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(id, _)| id)
    }

    pub fn living_entities_with<T: Component>(&self) -> Vec<EntityId> {
        self.entities_with::<T>()
            .into_iter()
            .filter(|id| !self.is_dead(*id))
            .collect()
    }

    pub fn nearest_living_with<T: Component>(
        &self,
        origin: Vec2,
        max_distance: f32,
    ) -> Option<EntityId> {
        self.nearest_by_position::<T>(origin, max_distance, |id| !self.is_dead(id))
    }

    pub fn first_entity_with<T: Component>(&self) -> Option<EntityId> {
        self.entities_with::<T>().into_iter().next()
    }

    pub fn first_position<T: Component>(&self) -> Option<Vec2> {
        self.entities_with::<T>()
            .into_iter()
            .find_map(|id| self.position(id))
    }

    pub fn for_each<T: Component>(&self, mut f: impl FnMut(EntityId, &T)) {
        for id in self.entities_with::<T>() {
            if let Some(component) = self.component::<T>(id) {
                f(id, component);
            }
        }
    }

    pub fn each<T: Component>(&self, f: impl FnMut(EntityId, &T)) {
        self.for_each::<T>(f);
    }

    pub fn for_each2<A: Component, B: Component>(&self, mut f: impl FnMut(EntityId, &A, &B)) {
        for id in self.entities_with::<A>() {
            let Some(a) = self.component::<A>(id) else {
                continue;
            };
            let Some(b) = self.component::<B>(id) else {
                continue;
            };
            f(id, a, b);
        }
    }

    pub fn each2<A: Component, B: Component>(&self, f: impl FnMut(EntityId, &A, &B)) {
        self.for_each2::<A, B>(f);
    }

    pub fn for_each1_mut<T: Component>(&mut self, mut f: impl FnMut(EntityId, &mut T)) {
        for id in self.entities_with::<T>() {
            if let Some(component) = self.component_mut::<T>(id) {
                f(id, component);
            }
        }
    }

    pub fn for_each2_copy_mut<A, B>(&mut self, mut f: impl FnMut(EntityId, A, &mut B))
    where
        A: Component + Copy,
        B: Component,
    {
        for id in self.entities_with::<A>() {
            let Some(a) = self.component::<A>(id).copied() else {
                continue;
            };
            let Some(b) = self.component_mut::<B>(id) else {
                continue;
            };
            f(id, a, b);
        }
    }

    pub fn for_each3_copy_mut<A, B, C>(&mut self, mut f: impl FnMut(EntityId, A, B, &mut C))
    where
        A: Component + Copy,
        B: Component + Copy,
        C: Component,
    {
        for id in self.entities_with::<A>() {
            let Some(a) = self.component::<A>(id).copied() else {
                continue;
            };
            let Some(b) = self.component::<B>(id).copied() else {
                continue;
            };
            let Some(c) = self.component_mut::<C>(id) else {
                continue;
            };
            f(id, a, b, c);
        }
    }

    /// Drives entities with input component `C`, speed component `S`, and
    /// velocity using the content-defined movement traits.
    pub fn drive_input<C, S>(&mut self)
    where
        C: Component + Copy + InputDriven,
        S: Component + Copy + MovementSpeed,
    {
        let input = self.input().clone();
        self.for_each3_copy_mut::<C, S, Velocity>(|_, controller, speed, velocity| {
            velocity.0 = input.axis2d(controller.movement_axis()) * speed.units_per_second();
        });
    }

    pub fn move_and_collide(&mut self, dt: f32) {
        game_physics::movement_system(self.inner.world, self.inner.map, dt);
    }

    pub fn run_patrol(&mut self, dt: f32) {
        game_ai::patrol_system(self.inner.world, dt);
    }

    pub fn chase_target(&mut self, target: Option<Vec2>, dt: f32) {
        game_ai::chase_system(self.inner.world, self.inner.nav, target, dt);
    }

    pub fn chase_first<T: Component>(&mut self, dt: f32) {
        let target = self.first_position::<T>();
        self.chase_target(target, dt);
    }

    pub fn camera_follow_first<T: Component>(&mut self) {
        if let Some(position) = self.first_position::<T>() {
            self.camera_mut().set_center(position);
        }
    }

    pub fn zoom_camera_from_actions(&mut self, zoom_in: ActionId, zoom_out: ActionId, dt: f32) {
        let zoom_in = self.down(zoom_in);
        let zoom_out = self.down(zoom_out);
        if zoom_in == zoom_out {
            return;
        }

        let zoom_step = 1.0 + 2.0 * dt;
        let mut zoom = self.camera().zoom();
        if zoom_in {
            zoom *= zoom_step;
        } else {
            zoom /= zoom_step;
        }
        self.camera_mut().set_zoom(zoom.clamp(0.25, 6.0));
    }

    pub fn stop_all_velocity(&mut self) {
        self.for_each1_mut::<Velocity>(|_, velocity| {
            velocity.0 = Vec2::ZERO;
        });
    }

    pub fn damage(&mut self, id: EntityId, amount: i32) -> bool {
        game_combat::apply_damage(self.inner.world, id, amount)
    }

    pub fn melee_attack_mut(&mut self, id: EntityId) -> Option<&mut MeleeAttack> {
        self.component_mut::<MeleeAttack>(id)
    }

    pub fn is_dead(&self, id: EntityId) -> bool {
        self.component::<Health>(id).is_some_and(Health::is_dead)
    }

    pub fn faction(&self, id: EntityId) -> Option<FactionId> {
        self.component::<Faction>(id).map(|faction| faction.0)
    }

    pub fn entities_in_faction(&self, faction: FactionId) -> Vec<EntityId> {
        self.entities_with::<Faction>()
            .into_iter()
            .filter(|id| self.faction(*id) == Some(faction))
            .collect()
    }

    /// (Re)spawns the start map's objects, clearing the world first.
    pub fn spawn_start_map(&mut self) -> Result<()> {
        reset_to_start_map_world(self.inner.world)
    }

    /// Resets the world to the start map (clears entities and queued commands, then
    /// respawns map objects).
    pub fn reset_to_start_map(&mut self) -> Result<()> {
        self.restart_start_map()
    }

    /// Resets to the start map and logs any failure. Production content can use
    /// this when it wants an infallible runtime action while keeping reset errors
    /// visible.
    pub fn reset_to_start_map_or_log(&mut self) {
        self.restart_start_map_or_log();
    }

    pub fn restart_level(&mut self) {
        self.restart_map_or_log();
    }

    /// Requests that the running game close after the current update.
    pub fn quit(&mut self) {
        self.commands().quit();
    }

    /// Reparses the active text map and queues its collision-map replacement.
    /// This is intended for the development reload action, not regular gameplay.
    pub fn reload_current_map(&mut self) -> Result<()> {
        let map_name = self
            .inner
            .world
            .get_resource::<ContentRuntime>()
            .map(|runtime| runtime.current_map_name().to_owned())
            .unwrap_or_else(|| "current map".to_owned());
        let (map, data) = self
            .inner
            .world
            .get_resource_mut::<ContentRuntime>()
            .ok_or_else(|| {
                anyhow::anyhow!("content runtime missing; was the game-kit plugin used?")
            })?
            .reload_current_text_map()?;
        restart_current_map_world(self.inner.world)?;
        self.inner.world.insert_resource(MapReload { map, data });
        if let Some(iteration) = self.inner.world.get_resource_mut::<DebugIterationInfo>() {
            iteration.last_reload = format!("ok ({map_name})");
        }
        self.commands().reload_map(map);
        Ok(())
    }

    /// Reloads the active text map, logging a useful error when the map source
    /// cannot be parsed or the current map is not text-backed.
    pub fn reload_current_map_or_log(&mut self) {
        if let Err(error) = self.reload_current_map() {
            if let Some(iteration) = self.inner.world.get_resource_mut::<DebugIterationInfo>() {
                iteration.last_reload = format!(
                    "failed: {}",
                    error.to_string().lines().next().unwrap_or("unknown error")
                );
            }
            log::error!("failed to reload current map: {error:#}");
        }
    }

    /// Re-reads the tuning file registered with [`GameApp::tuning_from_file`](crate::GameApp::tuning_from_file).
    ///
    /// This updates the values used by future authoring/reload-aware spawns;
    /// values already copied into existing entity components intentionally stay
    /// unchanged until those entities are recreated.
    pub fn reload_tuning(&mut self) -> Result<()> {
        let tuning = self.resource_mut::<TuningFile>().ok_or_else(|| {
            anyhow::anyhow!(
                "no tuning file is registered. Call game.tuning_from_file(\"tuning/game.ron\") during setup first."
            )
        })?;
        tuning.reload()
    }

    /// Reloads tuning and logs a useful error instead of interrupting gameplay.
    pub fn reload_tuning_or_log(&mut self) {
        if let Err(error) = self.reload_tuning() {
            log::error!("failed to reload tuning: {error:#}");
        }
    }

    /// Reloads tuning when this game registered a tuning file, without logging
    /// anything for games that use only literal prefab values.
    ///
    /// This lets the standard development reload action serve both text maps
    /// and optional tuning files while preserving its quiet behavior for games
    /// that never opted into tuning.
    pub fn reload_tuning_if_configured_or_log(&mut self) -> bool {
        if self.resource::<TuningFile>().is_none() {
            return false;
        }
        self.reload_tuning_or_log();
        true
    }

    pub fn change_map(&mut self, map: &str) -> Result<()> {
        let map_id = change_to_map_world(self.inner.world, map)?;
        self.commands().change_map(map_id);
        Ok(())
    }

    pub fn change_map_or_log(&mut self, map: &str) {
        if let Err(err) = self.change_map(map) {
            log::error!("failed to change map to '{map}': {err:?}");
        }
    }

    pub fn restart_map(&mut self) -> Result<()> {
        restart_current_map_world(self.inner.world)?;
        self.commands().restart_map();
        Ok(())
    }

    pub fn restart_map_or_log(&mut self) {
        if let Err(err) = self.restart_map() {
            log::error!("failed to restart current map: {err:?}");
        }
    }

    pub fn restart_start_map(&mut self) -> Result<()> {
        restart_start_map_world(self.inner.world)?;
        self.commands().restart_start_map();
        Ok(())
    }

    pub fn restart_start_map_or_log(&mut self) {
        if let Err(err) = self.restart_start_map() {
            log::error!("failed to restart start map: {err:?}");
        }
    }

    pub fn spawn_prefab_at(&mut self, prefab: &str, position: Vec2) -> Result<()> {
        self.spawn_prefab_with_properties(prefab, position, PropertyBag::default())
    }

    pub(crate) fn spawn_prefab_with_properties(
        &mut self,
        prefab: &str,
        position: Vec2,
        properties: PropertyBag,
    ) -> Result<()> {
        let prefab_id = self
            .inner
            .world
            .get_resource::<ContentRuntime>()
            .and_then(|runtime| runtime.prefab_id(prefab))
            .ok_or_else(|| anyhow::anyhow!("unknown prefab '{prefab}'"))?;
        self.commands()
            .spawn_prefab(prefab_id, position, properties);
        Ok(())
    }

    pub fn spawn_prefab_or_log(&mut self, prefab: &str, position: Vec2) {
        if let Err(err) = self.spawn_prefab_at(prefab, position) {
            log::error!("failed to queue spawn for prefab '{prefab}': {err:?}");
        }
    }

    pub(crate) fn spawn_prefab_with_properties_or_log(
        &mut self,
        prefab: &str,
        position: Vec2,
        properties: PropertyBag,
    ) {
        if let Err(err) = self.spawn_prefab_with_properties(prefab, position, properties) {
            log::error!("failed to queue spawn for prefab '{prefab}': {err:?}");
        }
    }

    pub fn reset_start_map_and_resource<T: Default + 'static>(&mut self) -> Result<()> {
        self.restart_start_map()?;
        self.insert_resource(T::default());
        Ok(())
    }

    /// The author name of the map currently spawned, if the content runtime is
    /// installed.
    pub fn current_map_name(&self) -> Option<String> {
        self.inner
            .world
            .get_resource::<ContentRuntime>()
            .map(|runtime| runtime.current_map_name().to_owned())
    }
}

/// Deferred runtime commands. Only operations the engine actually supports are
/// exposed.
pub struct Commands<'a> {
    queue: &'a mut CommandQueue,
}

impl Commands<'_> {
    /// Despawns `entity` after the current step.
    pub fn despawn(&mut self, entity: EntityId) {
        self.queue.despawn(entity);
    }

    /// Plays `sound` after the current step.
    pub fn play_sound(&mut self, sound: SoundHandle) {
        self.queue.play_sound(sound);
    }

    pub fn spawn_prefab(&mut self, prefab: PrefabId, position: Vec2, properties: PropertyBag) {
        self.queue.spawn_prefab(prefab, position, properties);
    }

    pub fn change_map(&mut self, map: game_core::builder::MapId) {
        self.queue.change_map(map);
    }

    pub fn quit(&mut self) {
        self.queue.quit();
    }

    pub fn reload_map(&mut self, map: game_core::builder::MapId) {
        self.queue.reload_map(map);
    }

    pub fn restart_map(&mut self) {
        self.queue.restart_map();
    }

    pub fn restart_start_map(&mut self) {
        self.queue.restart_start_map();
    }
}

/// Context handed to startup systems. Has the world (and map services) but no
/// per-frame input/camera/render state.
pub struct StartupGameCtx<'a, 'w> {
    inner: &'a mut StartCtx<'w>,
}

impl<'a, 'w> StartupGameCtx<'a, 'w> {
    pub(crate) fn new(inner: &'a mut StartCtx<'w>) -> Self {
        Self { inner }
    }

    /// Inserts a world resource, returning any previous value.
    pub fn insert_resource<T: 'static>(&mut self, value: T) -> Option<T> {
        self.inner.world.insert_resource(value)
    }

    /// Reads a world resource, inserting a default first if absent.
    pub fn resource_or_insert_with<T: 'static>(&mut self, create: impl FnOnce() -> T) -> &mut T {
        self.inner.world.resource_or_insert_with(create)
    }

    pub fn init_resource<T: Default + 'static>(&mut self) {
        self.resource_or_insert_with(T::default);
    }

    /// Reads a world resource.
    pub fn resource<T: 'static>(&self) -> Option<&T> {
        self.inner.world.get_resource::<T>()
    }

    /// Spawns the start map's objects (clearing the world first).
    pub fn spawn_start_map(&mut self) -> Result<()> {
        reset_to_start_map_world(self.inner.world)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs;
    use std::rc::Rc;

    use game_core::app::{Ctx, RenderFrame};
    use game_core::assets::AssetRegistry;
    use game_core::audio::{Audio, AudioCommands};
    use game_core::backend::AudioCommand;
    use game_core::builder::{MapId, PrefabRegistry, PropertyBag};
    use game_core::camera::Camera2D;
    use game_core::commands::{Command, CommandQueue};
    use game_core::gfx::Gfx;
    use game_core::input::Input;
    use game_core::nav::NavGrid;
    use game_core::tilemap::TileMap;
    use game_core::world::{Entity, Velocity, World};
    use game_map::MapBuilder;
    use glam::{Vec2, vec2};

    use super::GameCtx;
    use crate::assets::AssetLookup;
    use crate::beginner::tuning::TuningFile;
    use crate::map::ContentRuntime;

    #[derive(Clone, Copy)]
    struct Marker;

    #[derive(Clone, Copy)]
    struct Speed(f32);

    fn with_game_ctx<R>(world: &mut World, f: impl FnOnce(&mut GameCtx<'_, '_>) -> R) -> R {
        let map = TileMap::from_rows(&["..."], 16.0);
        let nav = NavGrid::from_tilemap(&map);
        let input = Input::default();
        let mut camera = Camera2D::new(Vec2::ZERO, 1.0);
        let mut frame = RenderFrame::new(camera);
        let mut audio_commands = AudioCommands::default();
        let mut ctx = Ctx {
            world,
            map: &map,
            nav: &nav,
            input: &input,
            camera: &mut camera,
            gfx: Gfx::new(&mut frame),
            audio: Audio::new(&mut audio_commands),
        };
        let mut game = GameCtx::new(&mut ctx);
        f(&mut game)
    }

    fn with_game_ctx_audio<R>(
        world: &mut World,
        f: impl FnOnce(&mut GameCtx<'_, '_>) -> R,
    ) -> (R, Vec<AudioCommand>) {
        let map = TileMap::from_rows(&["..."], 16.0);
        let nav = NavGrid::from_tilemap(&map);
        let input = Input::default();
        let mut camera = Camera2D::new(Vec2::ZERO, 1.0);
        let mut frame = RenderFrame::new(camera);
        let mut audio_commands = AudioCommands::default();
        let mut ctx = Ctx {
            world,
            map: &map,
            nav: &nav,
            input: &input,
            camera: &mut camera,
            gfx: Gfx::new(&mut frame),
            audio: Audio::new(&mut audio_commands),
        };
        let result = {
            let mut game = GameCtx::new(&mut ctx);
            f(&mut game)
        };
        (result, audio_commands.drain().collect())
    }

    fn empty_game_map(name: &str) -> game_map::GameMap {
        MapBuilder::new(name, 16.0)
            .try_tile_layer("collision", &["."])
            .unwrap()
            .finish()
    }

    #[test]
    fn first_position_returns_marker_entity_transform_position() {
        let mut world = World::new();
        world.spawn(Entity::new(vec2(3.0, 4.0)).with(Marker));

        let position = with_game_ctx(&mut world, |game| game.first_position::<Marker>());

        assert_eq!(position, Some(vec2(3.0, 4.0)));
    }

    #[test]
    fn for_each2_copy_mut_reads_speed_and_mutates_velocity() {
        let mut world = World::new();
        let id = world.spawn(Entity::new(Vec2::ZERO).with(Speed(7.0)));

        with_game_ctx(&mut world, |game| {
            game.for_each2_copy_mut::<Speed, Velocity>(|_, speed, velocity| {
                velocity.0 = vec2(speed.0, 0.0);
            });
        });

        assert_eq!(world.get::<Velocity>(id).unwrap().0, vec2(7.0, 0.0));
    }

    #[test]
    fn for_each1_mut_mutates_velocity() {
        let mut world = World::new();
        let id = world.spawn(Entity::new(Vec2::ZERO));

        with_game_ctx(&mut world, |game| {
            game.for_each1_mut::<Velocity>(|_, velocity| {
                velocity.0 = vec2(1.0, 2.0);
            });
        });

        assert_eq!(world.get::<Velocity>(id).unwrap().0, vec2(1.0, 2.0));
    }

    #[test]
    fn spawn_prefab_at_queues_resolved_prefab() {
        let mut prefabs = PrefabRegistry::new();
        let prefab = prefabs.register("marker", |world, position, _| {
            Ok(world.spawn(Entity::new(position)))
        });

        let mut world = World::new();
        world.insert_resource(ContentRuntime::new(
            Rc::new(prefabs),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            "test".to_owned(),
        ));

        with_game_ctx(&mut world, |game| {
            game.spawn_prefab_at("marker", vec2(5.0, 6.0)).unwrap();
        });

        let commands = world
            .get_resource_mut::<CommandQueue>()
            .unwrap()
            .drain()
            .collect::<Vec<_>>();
        assert_eq!(
            commands,
            vec![Command::SpawnPrefab {
                prefab,
                position: vec2(5.0, 6.0),
                properties: PropertyBag::default(),
            }]
        );
    }

    #[test]
    fn change_map_switches_content_runtime_and_queues_active_map_command() {
        let mut maps = HashMap::new();
        maps.insert("first".to_owned(), empty_game_map("first"));
        maps.insert("second".to_owned(), empty_game_map("second"));
        let map_ids = HashMap::from([
            ("first".to_owned(), MapId(0)),
            ("second".to_owned(), MapId(1)),
        ]);

        let mut world = World::new();
        world.insert_resource(ContentRuntime::new(
            Rc::new(PrefabRegistry::new()),
            maps,
            map_ids,
            HashMap::new(),
            "first".to_owned(),
        ));

        with_game_ctx(&mut world, |game| {
            game.change_map("second").unwrap();
        });

        assert_eq!(
            world
                .get_resource::<ContentRuntime>()
                .unwrap()
                .current_map_name(),
            "second"
        );
        let commands = world
            .get_resource_mut::<CommandQueue>()
            .unwrap()
            .drain()
            .collect::<Vec<_>>();
        assert_eq!(commands, vec![Command::ChangeMap(MapId(1))]);
    }

    #[test]
    fn reload_tuning_replaces_the_registered_values() {
        let path = std::env::temp_dir().join(format!(
            "game-kit-context-tuning-{}-{}.ron",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::write(&path, "{ \"slime.health\": 40.0 }").unwrap();

        let mut world = World::new();
        world.insert_resource(TuningFile::from_file(&path).unwrap());
        fs::write(&path, "{ \"slime.health\": 75.0 }").unwrap();

        with_game_ctx(&mut world, |game| game.reload_tuning().unwrap());

        assert_eq!(
            world
                .get_resource::<TuningFile>()
                .unwrap()
                .int("slime.health")
                .initial(),
            75
        );
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn optional_tuning_reload_is_quiet_without_tuning_and_reloads_when_configured() {
        let mut world = World::new();
        assert!(!with_game_ctx(&mut world, |game| {
            game.reload_tuning_if_configured_or_log()
        }));

        let path = std::env::temp_dir().join(format!(
            "game-kit-context-optional-tuning-{}-{}.ron",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::write(&path, "{ \"slime.health\": 40.0 }").unwrap();
        world.insert_resource(TuningFile::from_file(&path).unwrap());
        fs::write(&path, "{ \"slime.health\": 75.0 }").unwrap();

        assert!(with_game_ctx(&mut world, |game| {
            game.reload_tuning_if_configured_or_log()
        }));
        assert_eq!(
            world
                .get_resource::<TuningFile>()
                .unwrap()
                .int("slime.health")
                .initial(),
            75
        );
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn named_audio_ops_emit_bus_music_crossfade_and_fade_commands() {
        let mut registry = AssetRegistry::new();
        registry.try_sound_file("hit", "sounds/hit.wav").unwrap();
        registry.try_sound_file("theme", "music/theme.wav").unwrap();

        let mut world = World::new();
        world.insert_resource(AssetLookup::from_registry(&registry));
        let (_, commands) = with_game_ctx_audio(&mut world, |game| {
            game.audio().play_sound("hit").bus("ambience");
            game.audio().play_music("theme").volume(0.4).fade_in(1.5);
            let mut audio = game.audio();
            audio.set_master_volume(0.8);
            audio.set_sfx_volume(0.7);
            audio.set_music_volume(0.6);
            audio.bus("ambience").volume(0.5);
            audio.crossfade_music("theme", 0.75);
            audio.fade_music_to(0.0, 1.0);
            audio.pause_music();
            audio.resume_music();
        });

        assert_eq!(
            commands,
            vec![
                AudioCommand::Play {
                    sound: game_core::backend::SoundHandle(0),
                    volume: 1.0,
                    looping: false,
                    bus: Some("ambience".to_owned()),
                },
                AudioCommand::PlayMusic {
                    sound: game_core::backend::SoundHandle(1),
                    volume: 0.4,
                    fade_in_seconds: Some(1.5),
                },
                AudioCommand::SetMasterVolume { volume: 0.8 },
                AudioCommand::SetSfxVolume { volume: 0.7 },
                AudioCommand::SetMusicVolume { volume: 0.6 },
                AudioCommand::SetBusVolume {
                    bus: "ambience".to_owned(),
                    volume: 0.5,
                },
                AudioCommand::CrossfadeMusic {
                    sound: game_core::backend::SoundHandle(1),
                    volume: 1.0,
                    duration_seconds: 0.75,
                },
                AudioCommand::FadeMusicTo {
                    volume: 0.0,
                    duration_seconds: 1.0,
                },
                AudioCommand::PauseMusic,
                AudioCommand::ResumeMusic,
            ]
        );
    }
}

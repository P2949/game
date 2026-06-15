//! Content-facing system contexts (Phase 8).
//!
//! [`GameCtx`] wraps the engine's per-step `Ctx` and [`StartupGameCtx`] wraps the
//! startup `StartCtx`, exposing readable helpers for input, camera, world,
//! resources, UI text, audio, commands, and map services — so content systems
//! never name `Ctx`, `StartCtx`, raw `World`, or `CommandQueue`.

use anyhow::Result;
use game_core::app::{Ctx, StartCtx};
use game_core::backend::SoundHandle;
use game_core::camera::Camera2D;
use game_core::commands::CommandQueue;
use game_core::input::Input;
use game_core::nav::NavGrid;
use game_core::tilemap::TileMap;
use game_core::world::{Component, EntityId, World};
use glam::{Vec2, Vec4};

use crate::map::{ContentRuntime, reset_to_start_map_world};

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

    /// The current map's collision tilemap.
    pub fn map(&self) -> &TileMap {
        self.inner.map
    }

    /// The current map's navigation grid.
    pub fn nav(&self) -> &NavGrid {
        self.inner.nav
    }

    /// Read-only access to the entity world.
    pub fn world(&self) -> &World {
        self.inner.world
    }

    /// Mutable access to the entity world.
    pub fn world_mut(&mut self) -> &mut World {
        self.inner.world
    }

    /// World and input together (disjoint borrows), for systems that drive
    /// entities from input: `let (world, input) = game.world_and_input();`.
    pub fn world_and_input(&mut self) -> (&mut World, &Input) {
        (&mut *self.inner.world, self.inner.input)
    }

    /// World and collision tilemap together (disjoint borrows), for movement.
    pub fn world_and_map(&mut self) -> (&mut World, &TileMap) {
        (&mut *self.inner.world, self.inner.map)
    }

    /// World and navigation grid together (disjoint borrows), for pathfinding.
    pub fn world_and_nav(&mut self) -> (&mut World, &NavGrid) {
        (&mut *self.inner.world, self.inner.nav)
    }

    /// Queues UI text for this frame.
    pub fn text(&mut self, text: &str, pos: Vec2, color: Vec4) {
        self.inner.gfx.text(text, pos, color);
    }

    /// Plays a sound effect immediately (bypassing the command queue).
    pub fn play_sound(&mut self, sound: SoundHandle, volume: f32) {
        self.inner.audio.play(sound, volume);
    }

    /// Deferred world commands (despawn, play sound) applied after the step.
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
    pub fn ids_with<T: Component>(&self) -> Vec<EntityId> {
        self.inner.world.ids_with::<T>()
    }

    /// (Re)spawns the start map's objects, clearing the world first.
    pub fn spawn_start_map(&mut self) -> Result<()> {
        reset_to_start_map_world(self.inner.world)
    }

    /// Resets the world to the start map (clears entities and queued commands, then
    /// respawns map objects).
    pub fn reset_to_start_map(&mut self) -> Result<()> {
        reset_to_start_map_world(self.inner.world)
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

/// Deferred world commands. Only operations the engine actually supports are
/// exposed (despawn, play sound).
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

    /// Read-only access to the entity world.
    pub fn world(&self) -> &World {
        self.inner.world
    }

    /// Mutable access to the entity world.
    pub fn world_mut(&mut self) -> &mut World {
        self.inner.world
    }

    /// Inserts a world resource, returning any previous value.
    pub fn insert_resource<T: 'static>(&mut self, value: T) -> Option<T> {
        self.inner.world.insert_resource(value)
    }

    /// Reads a world resource, inserting a default first if absent.
    pub fn resource_or_insert_with<T: 'static>(&mut self, create: impl FnOnce() -> T) -> &mut T {
        self.inner.world.resource_or_insert_with(create)
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

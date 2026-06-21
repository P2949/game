//! Prefab authoring (Phases 4 & 5).
//!
//! [`PrefabAuthor`] registers an entity composition under a name and declares the
//! components a valid spawn must carry — without content touching `PrefabRegistry`
//! or `PrefabValidator`. Reached through [`GameApp::prefab`].

use anyhow::Result;
use game_core::builder::{PrefabRegistry, PropertyBag};
use game_core::input::Axis2dId;
use game_core::world::{Component, World};
use glam::Vec2;

use crate::app::PrefabRequirement;
use crate::beginner::actors::PrefabName;
use crate::bundle::Bundle;

/// A name used to refer to authored game content, such as a prefab or map.
///
/// This intentionally accepts only ordinary strings. Its main job is to make
/// an accidental non-string argument point back to the content name expected
/// by the authoring API.
#[diagnostic::on_unimplemented(
    message = "this argument needs a game-content name such as `\"player\"`",
    label = "this is not a content name"
)]
pub trait IntoContentName {
    /// Returns the content name used by the runtime registry.
    fn into_content_name(self) -> String;
}

impl IntoContentName for String {
    fn into_content_name(self) -> String {
        self
    }
}

impl IntoContentName for &str {
    fn into_content_name(self) -> String {
        self.to_owned()
    }
}

/// A two-dimensional input axis used for player movement.
#[diagnostic::on_unimplemented(
    message = "`.moves_with(...)` needs an input axis, usually `controls.movement` from `game.input(...)`",
    label = "this is not a movement axis"
)]
pub trait IntoMovementAxis {
    /// Returns the input axis used to drive movement.
    fn into_movement_axis(self) -> Axis2dId;
}

impl IntoMovementAxis for Axis2dId {
    fn into_movement_axis(self) -> Axis2dId {
        self
    }
}

/// Defines one prefab: a spawn recipe (a bundle built from the map-object
/// position) plus the components a valid spawn must include.
pub struct PrefabAuthor<'a> {
    name: String,
    prefabs: &'a mut PrefabRegistry,
    requirements: &'a mut Vec<PrefabRequirement>,
}

impl<'a> PrefabAuthor<'a> {
    pub(crate) fn new(
        name: String,
        prefabs: &'a mut PrefabRegistry,
        requirements: &'a mut Vec<PrefabRequirement>,
    ) -> Self {
        Self {
            name,
            prefabs,
            requirements,
        }
    }

    /// Registers how this prefab spawns: `build` receives the map-object position
    /// and returns a tuple [`Bundle`] of components.
    pub fn spawn<B, F>(&mut self, build: F) -> Result<&mut Self>
    where
        B: Bundle,
        F: Fn(Vec2) -> B + 'static,
    {
        self.spawn_with_properties(move |position, _properties| build(position))
    }

    /// Registers a spawn recipe that can consume deferred authoring properties.
    /// Beginner helpers use this internally for configured projectile direction
    /// and ownership while ordinary content can continue using [`Self::spawn`].
    pub fn spawn_with_properties<B, F>(&mut self, build: F) -> Result<&mut Self>
    where
        B: Bundle,
        F: Fn(Vec2, &PropertyBag) -> B + 'static,
    {
        let prefab_name = self.name.clone();
        self.prefabs
            .try_register(self.name.clone(), move |world, position, properties| {
                let entity = world.spawn(build(position, properties).build());
                world.insert(entity, PrefabName(prefab_name.clone()));
                Ok(entity)
            })?;
        Ok(self)
    }

    /// Registers a spawn recipe that can inspect the world at spawn time.
    ///
    /// Beginner prefab authors use this for reload-aware values such as tuning
    /// keys. Ordinary content should continue to prefer [`Self::spawn`].
    pub(crate) fn spawn_with_world<B, F>(&mut self, build: F) -> Result<&mut Self>
    where
        B: Bundle,
        F: Fn(&World, Vec2) -> B + 'static,
    {
        let prefab_name = self.name.clone();
        self.prefabs
            .try_register(self.name.clone(), move |world, position, _properties| {
                let entity = world.spawn(build(world, position).build());
                world.insert(entity, PrefabName(prefab_name.clone()));
                Ok(entity)
            })?;
        Ok(self)
    }

    /// Declares that a valid spawn of this prefab must carry component `T`. Checked
    /// once, before the runtime enters its loop (see [`GameApp::finish`]).
    pub fn require<T: Component>(&mut self) -> &mut Self {
        let name = self.name.clone();
        self.requirements.push(Box::new(move |validator| {
            validator.require_component::<T>(name.clone());
        }));
        self
    }
}

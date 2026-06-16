//! Prefab authoring (Phases 4 & 5).
//!
//! [`PrefabAuthor`] registers an entity composition under a name and declares the
//! components a valid spawn must carry — without content touching `PrefabRegistry`
//! or `PrefabValidator`. Reached through [`GameApp::prefab`].

use anyhow::Result;
use game_core::builder::PrefabRegistry;
use game_core::world::Component;
use glam::Vec2;

use crate::app::PrefabRequirement;
use crate::bundle::Bundle;

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
        self.prefabs
            .try_register(self.name.clone(), move |world, position, _properties| {
                Ok(world.spawn(build(position).build()))
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

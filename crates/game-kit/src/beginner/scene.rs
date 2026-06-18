//! Minimal beginner scene state.

use anyhow::Result;

use crate::context::{GameCtx, StartupGameCtx};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SceneRegistry {
    names: Vec<String>,
}

impl SceneRegistry {
    pub fn new(names: Vec<String>) -> Self {
        Self { names }
    }

    pub fn contains(&self, name: &str) -> bool {
        self.names.iter().any(|scene| scene == name)
    }

    pub fn names(&self) -> &[String] {
        &self.names
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SceneState {
    current: String,
}

impl SceneState {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            current: name.into(),
        }
    }

    pub fn current(&self) -> &str {
        &self.current
    }
}

impl<'a, 'w> GameCtx<'a, 'w> {
    pub fn change_scene(&mut self, scene: &str) -> Result<()> {
        if let Some(registry) = self.resource::<SceneRegistry>() {
            if !registry.contains(scene) {
                anyhow::bail!("unknown scene '{scene}'");
            }
        }
        self.insert_resource(SceneState::new(scene));
        Ok(())
    }

    pub fn change_scene_or_log(&mut self, scene: &str) {
        if let Err(err) = self.change_scene(scene) {
            log::error!("failed to change scene to '{scene}': {err:?}");
        }
    }

    pub fn current_scene_name(&self) -> Option<String> {
        self.resource::<SceneState>()
            .map(|scene| scene.current().to_owned())
    }
}

impl<'a, 'w> StartupGameCtx<'a, 'w> {
    pub fn start_scene(&mut self, scene: &str) {
        self.insert_resource(SceneState::new(scene));
    }
}

#[cfg(test)]
mod tests {
    use super::{SceneRegistry, SceneState};

    #[test]
    fn scene_registry_checks_names() {
        let registry = SceneRegistry::new(vec!["menu".to_owned(), "game".to_owned()]);

        assert!(registry.contains("menu"));
        assert!(!registry.contains("credits"));
    }

    #[test]
    fn scene_state_tracks_current_name() {
        let state = SceneState::new("game");

        assert_eq!(state.current(), "game");
    }
}

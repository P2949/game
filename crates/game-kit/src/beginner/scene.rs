//! Minimal beginner scene state.

use anyhow::Result;
use game_core::input::ActionId;
use glam::{vec2, vec4};

use crate::app::GameApp;
use crate::beginner::state::SimpleGameState;
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

pub struct SimpleSceneFlowAuthor<'a, 'app> {
    app: &'a mut GameApp<'app>,
    menu: Option<String>,
    game: Option<String>,
    game_over: Option<String>,
    start_on: Option<ActionId>,
    restart_on: Option<ActionId>,
}

impl<'a, 'app> SimpleSceneFlowAuthor<'a, 'app> {
    pub(crate) fn new(app: &'a mut GameApp<'app>) -> Self {
        Self {
            app,
            menu: None,
            game: None,
            game_over: None,
            start_on: None,
            restart_on: None,
        }
    }

    pub fn menu(mut self, name: impl Into<String>) -> Self {
        self.menu = Some(name.into());
        self
    }

    pub fn game(mut self, name: impl Into<String>) -> Self {
        self.game = Some(name.into());
        self
    }

    pub fn level(self, name: impl Into<String>) -> Self {
        self.game(name)
    }

    pub fn game_over(mut self, name: impl Into<String>) -> Self {
        self.game_over = Some(name.into());
        self
    }

    pub fn start_on(mut self, action: ActionId) -> Self {
        self.start_on = Some(action);
        self
    }

    pub fn restart_on(mut self, action: ActionId) -> Self {
        self.restart_on = Some(action);
        self
    }

    pub fn build(self) {
        let menu = self.menu.unwrap_or_else(|| "menu".to_owned());
        let game_scene = self.game.unwrap_or_else(|| "game".to_owned());
        let game_over = self.game_over.unwrap_or_else(|| "game_over".to_owned());
        let app = self.app;

        app.menu_scene(menu.clone())
            .level_scene(game_scene.clone())
            .game_over_scene(game_over.clone())
            .start_scene(menu.clone());

        app.on_start(|game: &mut StartupGameCtx<'_, '_>| {
            game.init_resource::<SimpleGameState>();
            game.spawn_start_map()
        });

        if let Some(start_on) = self.start_on {
            let target_scene = game_scene.clone();
            let target_map = game_scene.clone();
            app.on_scene(menu.clone(), move |game: &mut GameCtx<'_, '_>, _dt| {
                if game.pressed(start_on) {
                    start_scene_map(game, &target_scene, &target_map);
                }
            });
        }

        {
            let game_over_scene = game_over.clone();
            let game_over_map = game_over.clone();
            app.on_scene(
                game_scene.clone(),
                move |game: &mut GameCtx<'_, '_>, _dt| {
                    if game.player_is_dead() {
                        start_scene_map(game, &game_over_scene, &game_over_map);
                    }
                },
            );
        }

        if let Some(restart_on) = self.restart_on {
            let target_scene = game_scene.clone();
            let target_map = game_scene.clone();
            let game_scene_filter = game_scene.clone();
            let game_over_filter = game_over.clone();
            app.every_frame(move |game: &mut GameCtx<'_, '_>, _dt| {
                let current = game.current_scene_name();
                let can_restart = current.as_deref() == Some(game_scene_filter.as_str())
                    || current.as_deref() == Some(game_over_filter.as_str());
                if can_restart && game.pressed(restart_on) {
                    start_scene_map(game, &target_scene, &target_map);
                }
            });
        }

        app.draw_ui(move |game: &mut GameCtx<'_, '_>, _dt| {
            let current = game.current_scene_name();
            if current.as_deref() == Some(menu.as_str()) {
                game.text("Press start", vec2(24.0, 24.0), vec4(1.0, 0.95, 0.75, 1.0));
            } else if current.as_deref() == Some(game_over.as_str()) {
                game.text("Game Over", vec2(24.0, 24.0), vec4(1.0, 0.35, 0.25, 1.0));
                game.text("Press reset", vec2(24.0, 48.0), vec4(1.0, 0.95, 0.75, 1.0));
            }
        });
    }
}

fn start_scene_map(game: &mut GameCtx<'_, '_>, scene: &str, map: &str) {
    game.change_scene_or_log(scene);
    game.change_map_or_log(map);
    game.insert_resource(SimpleGameState::default());
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

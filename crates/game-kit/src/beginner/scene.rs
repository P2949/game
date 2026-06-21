//! Minimal beginner scene state.

use crate::app::GameApp;
use crate::beginner::actors::{Enemy, Pickup};
use crate::beginner::state::SimpleGameState;
use crate::context::{GameCtx, StartupGameCtx};
use anyhow::Result;
use game_core::input::ActionId;

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
    win: Option<String>,
    menu_text: Option<String>,
    menu_button: Option<(String, String)>,
    game_over_text: Option<String>,
    game_over_button: Option<String>,
    win_text: Option<String>,
    start_on: Option<ActionId>,
    restart_on: Option<ActionId>,
    win_when_all_pickups_collected: bool,
    win_when_all_enemies_dead: bool,
}

impl<'a, 'app> SimpleSceneFlowAuthor<'a, 'app> {
    pub(crate) fn new(app: &'a mut GameApp<'app>) -> Self {
        Self {
            app,
            menu: None,
            game: None,
            game_over: None,
            win: None,
            menu_text: None,
            menu_button: None,
            game_over_text: None,
            game_over_button: None,
            win_text: None,
            start_on: None,
            restart_on: None,
            win_when_all_pickups_collected: false,
            win_when_all_enemies_dead: false,
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

    pub fn win(mut self, name: impl Into<String>) -> Self {
        self.win = Some(name.into());
        self
    }

    pub fn menu_text(mut self, text: impl Into<String>) -> Self {
        self.menu_text = Some(text.into());
        self
    }

    /// Adds a mouse-clickable menu button that opens `map` in the configured
    /// game scene. Keyboard/controller `start_on` remains available too.
    pub fn menu_button(mut self, label: impl Into<String>, map: impl Into<String>) -> Self {
        self.menu_button = Some((label.into(), map.into()));
        self
    }

    pub fn game_over_text(mut self, text: impl Into<String>) -> Self {
        self.game_over_text = Some(text.into());
        self
    }

    /// Adds a mouse-clickable restart button to the game-over scene.
    pub fn game_over_button(mut self, label: impl Into<String>) -> Self {
        self.game_over_button = Some(label.into());
        self
    }

    pub fn win_text(mut self, text: impl Into<String>) -> Self {
        self.win_text = Some(text.into());
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

    pub fn win_when_all_pickups_collected(mut self) -> Self {
        self.win_when_all_pickups_collected = true;
        self
    }

    pub fn win_when_all_enemies_dead(mut self) -> Self {
        self.win_when_all_enemies_dead = true;
        self
    }

    pub fn build(self) {
        let menu = self.menu.unwrap_or_else(|| "menu".to_owned());
        let game_scene = self.game.unwrap_or_else(|| "game".to_owned());
        let game_over = self.game_over.unwrap_or_else(|| "game_over".to_owned());
        let win = self.win;
        let menu_text = self.menu_text.unwrap_or_else(|| "Press start".to_owned());
        let menu_button = self.menu_button;
        let game_over_text = self
            .game_over_text
            .unwrap_or_else(|| "Game Over".to_owned());
        let game_over_button = self.game_over_button;
        let win_text = self.win_text.unwrap_or_else(|| "You win!".to_owned());
        let app = self.app;

        app.menu_scene(menu.clone())
            .level_scene(game_scene.clone())
            .game_over_scene(game_over.clone())
            .start_scene(menu.clone());
        if let Some(win_scene) = &win {
            app.scene(win_scene.clone());
        }

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

        if let Some(win_scene) = win.clone() {
            let win_map = win_scene.clone();
            let require_pickups = self.win_when_all_pickups_collected;
            let require_enemies = self.win_when_all_enemies_dead;
            if require_pickups || require_enemies {
                app.on_scene(
                    game_scene.clone(),
                    move |game: &mut GameCtx<'_, '_>, _dt| {
                        let pickups_done =
                            !require_pickups || game.entities_with::<Pickup>().is_empty();
                        let enemies_done = !require_enemies
                            || game
                                .entities_with::<Enemy>()
                                .into_iter()
                                .all(|enemy| game.is_dead(enemy));
                        if pickups_done && enemies_done {
                            start_scene_map(game, &win_scene, &win_map);
                        }
                    },
                );
            }
        }

        app.draw_ui(move |game: &mut GameCtx<'_, '_>, _dt| {
            let current = game.current_scene_name();
            if current.as_deref() == Some(menu.as_str()) {
                let panel_position = scene_panel_position(game);
                game.ui().panel("Menu").line(&menu_text).at(panel_position);
                if let Some((label, map)) = &menu_button {
                    let target_scene = game_scene.clone();
                    let target_map = map.clone();
                    let button_position = scene_button_position(game);
                    game.ui()
                        .button(label)
                        .at_screen(button_position)
                        .on_click(move |game| {
                            start_scene_map(game, &target_scene, &target_map);
                        });
                }
            } else if current.as_deref() == Some(game_over.as_str()) {
                let panel_position = scene_panel_position(game);
                game.ui()
                    .panel("Game Over")
                    .line(&game_over_text)
                    .at(panel_position);
                if let Some(label) = &game_over_button {
                    let target_scene = game_scene.clone();
                    let target_map = game_scene.clone();
                    let button_position = scene_button_position(game);
                    game.ui()
                        .button(label)
                        .at_screen(button_position)
                        .on_click(move |game| {
                            start_scene_map(game, &target_scene, &target_map);
                        });
                }
            } else if current.as_deref() == win.as_deref() {
                game.ui().panel("You Win!").line(&win_text).center();
            }
        });
    }
}

fn scene_button_position(game: &GameCtx<'_, '_>) -> glam::Vec2 {
    let viewport = game.input().viewport_size();
    let center = if viewport.x > 0.0 && viewport.y > 0.0 {
        viewport * 0.5
    } else {
        glam::vec2(400.0, 300.0)
    };
    center + glam::vec2(0.0, 56.0)
}

fn scene_panel_position(game: &GameCtx<'_, '_>) -> glam::Vec2 {
    let viewport = game.input().viewport_size();
    let center = if viewport.x > 0.0 && viewport.y > 0.0 {
        viewport * 0.5
    } else {
        glam::vec2(400.0, 300.0)
    };
    center - glam::vec2(0.0, 48.0)
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

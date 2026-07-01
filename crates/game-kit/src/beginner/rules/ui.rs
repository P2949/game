use super::shared::*;

#[derive(Clone, Copy)]
struct HighLevelUiOptions {
    show_score: bool,
    show_enemy_count: bool,
    show_player_health: bool,
    show_menu: bool,
    show_pause_menu: bool,
    show_game_over_panel: bool,
    show_win_panel: bool,
}

fn high_level_ui_system(game: &mut GameCtx<'_, '_>, options: HighLevelUiOptions) {
    let mut ui = game.ui().top_left();
    if options.show_score {
        ui = ui.score_label();
    }
    if options.show_enemy_count {
        ui = ui.enemy_count_label();
    }
    if options.show_player_health {
        ui = ui.player_health_bar();
    }
    ui.build();

    let scene = game.current_scene_name();
    let state = game
        .resource::<SimpleGameState>()
        .copied()
        .unwrap_or_default();
    if options.show_menu && scene.as_deref() == Some("menu") {
        game.ui()
            .panel("Menu")
            .line("Press Space, Enter, or South to Start")
            .center();
    }
    if options.show_pause_menu && state.paused {
        game.ui()
            .panel("Paused")
            .line("Press P or Escape to Resume")
            .center();
    }
    if options.show_game_over_panel && (state.player_dead || scene.as_deref() == Some("game_over"))
    {
        game.ui()
            .panel("Game Over")
            .line("Press R to Restart")
            .center();
    }
    if options.show_win_panel && scene.as_deref() == Some("win") {
        game.ui().panel("You Win!").line("Great work!").center();
    }
}

/// Draws the selected high-level UI elements and panels.
pub struct HighLevelUiBehavior {
    pub show_score: bool,
    pub show_enemy_count: bool,
    pub show_player_health: bool,
    pub show_menu: bool,
    pub show_pause_menu: bool,
    pub show_game_over_panel: bool,
    pub show_win_panel: bool,
}

impl GamePlugin for HighLevelUiBehavior {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        let options = HighLevelUiOptions {
            show_score: self.show_score,
            show_enemy_count: self.show_enemy_count,
            show_player_health: self.show_player_health,
            show_menu: self.show_menu,
            show_pause_menu: self.show_pause_menu,
            show_game_over_panel: self.show_game_over_panel,
            show_win_panel: self.show_win_panel,
        };
        game.ui(move |game: &mut GameCtx<'_, '_>, _dt| {
            high_level_ui_system(game, options);
        });
        Ok(())
    }
}

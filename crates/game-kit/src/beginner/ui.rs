//! High-level beginner UI helpers.

use game_core::input::{ActionId, MouseButton};
use glam::{Vec2, vec2, vec4};

use crate::context::GameCtx;

const MARGIN: f32 = 24.0;
const LINE_HEIGHT: f32 = 28.0;
const ESTIMATED_GLYPH_WIDTH: f32 = 16.0;
const PANEL_PADDING: Vec2 = vec2(22.0, 18.0);
const BUTTON_PADDING: Vec2 = vec2(20.0, 10.0);
const TEXT_COLOR: glam::Vec4 = vec4(1.0, 0.95, 0.75, 1.0);
const PANEL_BACKGROUND: glam::Vec4 = vec4(0.055, 0.075, 0.12, 0.94);
const PANEL_BORDER: glam::Vec4 = vec4(0.92, 0.72, 0.26, 1.0);
const BUTTON_BACKGROUND: glam::Vec4 = vec4(0.15, 0.2, 0.32, 0.98);
const BUTTON_HOVER: glam::Vec4 = vec4(0.3, 0.38, 0.58, 1.0);
const BUTTON_FOCUSED: glam::Vec4 = vec4(0.42, 0.5, 0.74, 1.0);

#[derive(Clone, Copy)]
enum HorizontalLayout {
    Left,
    Center,
}

#[derive(Clone, Copy)]
enum StackDirection {
    Vertical,
    Horizontal,
}

/// Persistent selection for a beginner menu. It is updated automatically by
/// [`UiMenu`] using the standard top-down menu controls.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UiFocus {
    pub selected_index: usize,
}

#[derive(Clone, Copy)]
pub(crate) struct UiNavigation {
    up: ActionId,
    down: ActionId,
    accept: ActionId,
}

impl UiNavigation {
    pub(crate) fn new(up: ActionId, down: ActionId, accept: ActionId) -> Self {
        Self { up, down, accept }
    }
}

/// Immediate-mode helpers for common beginner labels.
pub struct UiOps<'g, 'a, 'w> {
    game: &'g mut GameCtx<'a, 'w>,
    next_line: Vec2,
    horizontal: HorizontalLayout,
    direction: StackDirection,
}

impl<'g, 'a, 'w> UiOps<'g, 'a, 'w> {
    pub(crate) fn new(game: &'g mut GameCtx<'a, 'w>) -> Self {
        Self {
            game,
            next_line: vec2(MARGIN, MARGIN),
            horizontal: HorizontalLayout::Left,
            direction: StackDirection::Vertical,
        }
    }

    pub fn top_left(mut self) -> Self {
        self.next_line = vec2(MARGIN, MARGIN);
        self.horizontal = HorizontalLayout::Left;
        self.direction = StackDirection::Vertical;
        self
    }

    /// Places subsequent label helpers along the top centre of the viewport.
    pub fn top_center(mut self) -> Self {
        self.next_line = vec2(self.viewport().x * 0.5, MARGIN);
        self.horizontal = HorizontalLayout::Center;
        self.direction = StackDirection::Vertical;
        self
    }

    /// Places subsequent label helpers along the bottom-left of the viewport.
    pub fn bottom_left(mut self) -> Self {
        self.next_line = vec2(MARGIN, self.viewport().y - MARGIN - LINE_HEIGHT * 2.0);
        self.horizontal = HorizontalLayout::Left;
        self.direction = StackDirection::Vertical;
        self
    }

    /// Stacks subsequent label helpers from top to bottom (the default).
    pub fn vertical(mut self) -> Self {
        self.direction = StackDirection::Vertical;
        self
    }

    /// Alias for [`Self::vertical`] that reads naturally for beginner UI code.
    pub fn column(self) -> Self {
        self.vertical()
    }

    /// Places subsequent label helpers on one row.
    pub fn horizontal(mut self) -> Self {
        self.direction = StackDirection::Horizontal;
        self
    }

    /// Alias for [`Self::horizontal`] that reads naturally for beginner UI code.
    pub fn row(self) -> Self {
        self.horizontal()
    }

    /// Leaves empty space in the current row or column.
    pub fn spacer(mut self, amount: f32) -> Self {
        match self.direction {
            StackDirection::Vertical => self.next_line.y += amount.max(0.0),
            StackDirection::Horizontal => self.next_line.x += amount.max(0.0),
        }
        self
    }

    pub fn text(self, text: impl Into<String>) -> UiText<'g, 'a, 'w> {
        UiText {
            game: self.game,
            text: text.into(),
            color: TEXT_COLOR,
            width: None,
        }
    }

    /// Begins a text box. Set its approximate pixel width with
    /// [`UiText::width`] before positioning it with [`UiText::at`].
    pub fn text_box(self, text: impl Into<String>) -> UiText<'g, 'a, 'w> {
        self.text(text)
    }

    pub fn top_left_text(mut self, text: impl AsRef<str>) -> Self {
        self.draw_line(text.as_ref(), TEXT_COLOR);
        self
    }

    pub fn center_text(mut self, text: impl AsRef<str>) -> Self {
        self.draw_centered(text.as_ref(), self.viewport().y * 0.5, TEXT_COLOR);
        self
    }

    /// Draws a centred panel title. Use [`Self::panel`] when the panel needs
    /// more than one line.
    pub fn center_panel(mut self, title: impl AsRef<str>) -> Self {
        self.draw_panel(&[title.as_ref().to_owned()], self.viewport() * 0.5);
        self
    }

    /// Begins a compact rectangle panel.
    pub fn panel(self, title: impl AsRef<str>) -> UiPanel<'g, 'a, 'w> {
        UiPanel {
            game: self.game,
            lines: vec![title.as_ref().to_owned()],
        }
    }

    /// Begins a conventional centred dialog box.
    pub fn dialog(self, speaker: impl AsRef<str>) -> UiPanel<'g, 'a, 'w> {
        self.panel(speaker)
    }

    /// Begins a centred menu with focus-aware buttons. Standard top-down
    /// controls bind Up/Down and a controller D-pad to selection, and
    /// Space/Enter/controller South to activation.
    pub fn menu(self, title: impl AsRef<str>) -> UiMenu<'g, 'a, 'w> {
        UiMenu {
            game: self.game,
            title: title.as_ref().to_owned(),
            buttons: Vec::new(),
        }
    }

    /// Begins a compact, conventional status panel for score, health, and
    /// remaining enemies.
    pub fn status_panel(self) -> UiStatusPanel<'g, 'a, 'w> {
        UiStatusPanel {
            game: self.game,
            score: false,
            player_health: false,
            enemy_count: false,
        }
    }

    /// Begins a clickable, screen-space button.
    pub fn button(self, label: impl AsRef<str>) -> UiButton<'g, 'a, 'w> {
        UiButton {
            game: self.game,
            label: label.as_ref().to_owned(),
            center: Some(self.next_line),
        }
    }

    /// Draws a conventional keyboard/controller start instruction.
    pub fn press_to_start(mut self, _action: game_core::input::ActionId) -> Self {
        self.draw_panel(
            &["Press Space, Enter, or South to Start".to_owned()],
            self.viewport() * 0.5,
        );
        self
    }

    /// Draws a conventional reset instruction.
    pub fn press_to_restart(mut self, _action: game_core::input::ActionId) -> Self {
        self.draw_panel(&["Press R to Restart".to_owned()], self.viewport() * 0.5);
        self
    }

    /// Draws a game-over panel with the normal restart binding.
    pub fn game_over_panel(mut self, _action: game_core::input::ActionId) -> Self {
        self.draw_panel(
            &["Game Over".to_owned(), "Press R to Restart".to_owned()],
            self.viewport() * 0.5,
        );
        self
    }

    /// Draws a paused-state panel with the normal pause binding.
    pub fn pause_panel(mut self, _action: game_core::input::ActionId) -> Self {
        self.draw_panel(
            &[
                "Paused".to_owned(),
                "Press P or Escape to Resume".to_owned(),
            ],
            self.viewport() * 0.5,
        );
        self
    }

    pub fn score_label(mut self) -> Self {
        let score = self.game.score().value();
        self.draw_line(&format!("Score: {score}"), vec4(1.0, 0.95, 0.35, 1.0));
        self
    }

    pub fn enemy_count_label(mut self) -> Self {
        let count = self.game.enemies().alive().count();
        self.draw_line(&format!("Enemies: {count}"), TEXT_COLOR);
        self
    }

    pub fn player_health_text(mut self) -> Self {
        let health = self.game.player().health().unwrap_or_default();
        self.draw_line(&format!("Health: {health}"), TEXT_COLOR);
        self
    }

    pub fn player_health_bar(mut self) -> Self {
        let label = self
            .game
            .player_id()
            .and_then(|id| self.game.component::<game_combat::Health>(id))
            .map(|health| {
                let slots = 10usize;
                let filled = ((health.current.max(0) as f32 / health.max.max(1) as f32)
                    * slots as f32)
                    .round()
                    .clamp(0.0, slots as f32) as usize;
                format!(
                    "Health: [{}{}] {}/{}",
                    "#".repeat(filled),
                    "-".repeat(slots - filled),
                    health.current,
                    health.max
                )
            })
            .unwrap_or_else(|| "Health: [----------]".to_owned());
        self.draw_line(&label, TEXT_COLOR);
        self
    }

    /// Completes an immediate-mode chain; helpers draw as they are called.
    pub fn build(self) {}

    fn draw_line(&mut self, text: &str, color: glam::Vec4) {
        let x = match self.horizontal {
            HorizontalLayout::Left => self.next_line.x,
            HorizontalLayout::Center => self.next_line.x - estimated_width(text) * 0.5,
        };
        self.game.text(text, vec2(x, self.next_line.y), color);
        match self.direction {
            StackDirection::Vertical => self.next_line.y += LINE_HEIGHT,
            StackDirection::Horizontal => self.next_line.x += estimated_width(text) + MARGIN,
        }
    }

    fn viewport(&self) -> Vec2 {
        let viewport = self.game.input().viewport_size();
        if viewport.x > 0.0 && viewport.y > 0.0 {
            viewport
        } else {
            vec2(800.0, 600.0)
        }
    }

    fn draw_centered(&mut self, text: &str, y: f32, color: glam::Vec4) {
        let x = self.viewport().x * 0.5 - estimated_width(text) * 0.5;
        self.game.text(text, vec2(x, y), color);
    }

    fn draw_panel(&mut self, lines: &[String], center: Vec2) {
        draw_text_panel(self.game, lines, center);
    }
}

/// A small rectangle-panel builder for text and instructions.
pub struct UiPanel<'g, 'a, 'w> {
    game: &'g mut GameCtx<'a, 'w>,
    lines: Vec<String>,
}

/// A lightweight screen-space button for beginner menus.
pub struct UiButton<'g, 'a, 'w> {
    game: &'g mut GameCtx<'a, 'w>,
    label: String,
    center: Option<Vec2>,
}

impl<'g, 'a, 'w> UiButton<'g, 'a, 'w> {
    /// Positions the button at the centre of the current viewport.
    pub fn at_center(mut self) -> Self {
        let viewport = self.game.input().viewport_size();
        self.center = Some(if viewport.x > 0.0 && viewport.y > 0.0 {
            viewport * 0.5
        } else {
            vec2(400.0, 300.0)
        });
        self
    }

    /// Alias for [`Self::at_center`], which reads naturally in immediate-mode
    /// conditionals: `if game.ui().button("Play").center().clicked() { ... }`.
    pub fn center(self) -> Self {
        self.at_center()
    }

    /// Places the button at a custom screen-space centre position.
    pub fn at(mut self, center: Vec2) -> Self {
        self.center = Some(center);
        self
    }

    /// Explicit spelling of [`Self::at`] for menu code.
    pub fn at_screen(self, center: Vec2) -> Self {
        self.at(center)
    }

    /// Draws the button and reports whether it received a left-click edge.
    pub fn clicked(mut self) -> bool {
        self.draw_and_hit_test()
    }

    /// Draws the button and invokes `f` when it receives a left-click edge.
    pub fn on_click(mut self, f: impl FnOnce(&mut GameCtx<'a, 'w>)) {
        if self.draw_and_hit_test() {
            f(self.game);
        }
    }

    fn draw_and_hit_test(&mut self) -> bool {
        let center = self.center.unwrap_or_else(|| vec2(MARGIN, MARGIN));
        draw_button(self.game, &self.label, center, false).1
    }
}

fn draw_button(
    game: &mut GameCtx<'_, '_>,
    label: &str,
    center: Vec2,
    focused: bool,
) -> (bool, bool) {
    let size = vec2(
        estimated_width(label) + BUTTON_PADDING.x * 2.0,
        LINE_HEIGHT + BUTTON_PADDING.y * 2.0,
    );
    let origin = center - size * 0.5;
    let mouse = game.mouse_position();
    let hovered = mouse.x >= origin.x
        && mouse.x <= origin.x + size.x
        && mouse.y >= origin.y
        && mouse.y <= origin.y + size.y;
    let background = if hovered {
        BUTTON_HOVER
    } else if focused {
        BUTTON_FOCUSED
    } else {
        BUTTON_BACKGROUND
    };
    let border = if focused {
        vec4(1.0, 0.9, 0.4, 1.0)
    } else {
        PANEL_BORDER
    };
    game.ui_rect_at_layer(
        origin - Vec2::splat(2.0),
        size + Vec2::splat(4.0),
        border,
        9_910,
    );
    game.ui_rect_at_layer(origin, size, background, 9_911);
    game.text(
        label,
        origin + vec2(BUTTON_PADDING.x, BUTTON_PADDING.y + 4.0),
        if hovered {
            vec4(1.0, 1.0, 0.6, 1.0)
        } else {
            TEXT_COLOR
        },
    );
    (hovered, hovered && game.mouse_pressed(MouseButton::Left))
}

enum MenuAction {
    Scene(String),
    Quit,
}

struct MenuButton {
    label: String,
    action: MenuAction,
}

/// A focused vertical menu. Build one inside `game.draw_ui(...)`.
pub struct UiMenu<'g, 'a, 'w> {
    game: &'g mut GameCtx<'a, 'w>,
    title: String,
    buttons: Vec<MenuButton>,
}

/// The action half of a [`UiMenu`] button declaration.
pub struct UiMenuButton<'g, 'a, 'w> {
    menu: UiMenu<'g, 'a, 'w>,
    label: String,
}

impl<'g, 'a, 'w> UiMenu<'g, 'a, 'w> {
    pub fn button(self, label: impl AsRef<str>) -> UiMenuButton<'g, 'a, 'w> {
        UiMenuButton {
            menu: self,
            label: label.as_ref().to_owned(),
        }
    }

    pub fn build(self) {
        if self.buttons.is_empty() {
            self.game.ui().panel(&self.title).center();
            return;
        }

        let navigation = self.game.resource::<UiNavigation>().copied();
        let mut focus = self.game.resource::<UiFocus>().copied().unwrap_or_default();
        if let Some(navigation) = navigation {
            if self.game.pressed(navigation.up) {
                focus.selected_index = if focus.selected_index == 0 {
                    self.buttons.len() - 1
                } else {
                    focus.selected_index - 1
                };
            }
            if self.game.pressed(navigation.down) {
                focus.selected_index = (focus.selected_index + 1) % self.buttons.len();
            }
        }
        focus.selected_index %= self.buttons.len();

        let viewport = self.game.input().viewport_size();
        let viewport = if viewport.x > 0.0 && viewport.y > 0.0 {
            viewport
        } else {
            vec2(800.0, 600.0)
        };
        let title_center = viewport * 0.5 - vec2(0.0, 72.0);
        draw_text_panel(self.game, std::slice::from_ref(&self.title), title_center);

        let first_y = viewport.y * 0.5 - (self.buttons.len() as f32 - 1.0) * 28.0;
        let activate = navigation.is_some_and(|navigation| self.game.pressed(navigation.accept));
        let mut selected_action = None;
        for (index, button) in self.buttons.iter().enumerate() {
            let center = vec2(viewport.x * 0.5, first_y + index as f32 * 56.0);
            let (hovered, clicked) = draw_button(
                self.game,
                &button.label,
                center,
                index == focus.selected_index,
            );
            if hovered {
                focus.selected_index = index;
            }
            if clicked || (activate && index == focus.selected_index) {
                selected_action = Some(match &button.action {
                    MenuAction::Scene(scene) => MenuAction::Scene(scene.clone()),
                    MenuAction::Quit => MenuAction::Quit,
                });
            }
        }
        self.game.insert_resource(focus);
        match selected_action {
            Some(MenuAction::Scene(scene)) => self.game.change_scene_or_log(&scene),
            Some(MenuAction::Quit) => self.game.quit(),
            None => {}
        }
    }
}

impl<'g, 'a, 'w> UiMenuButton<'g, 'a, 'w> {
    pub fn go_to_scene(mut self, scene: impl Into<String>) -> UiMenu<'g, 'a, 'w> {
        self.menu.buttons.push(MenuButton {
            label: self.label,
            action: MenuAction::Scene(scene.into()),
        });
        self.menu
    }

    pub fn quit(mut self) -> UiMenu<'g, 'a, 'w> {
        self.menu.buttons.push(MenuButton {
            label: self.label,
            action: MenuAction::Quit,
        });
        self.menu
    }
}

/// Builder for a standard score/health/enemy-count panel.
pub struct UiStatusPanel<'g, 'a, 'w> {
    game: &'g mut GameCtx<'a, 'w>,
    score: bool,
    player_health: bool,
    enemy_count: bool,
}

impl<'g, 'a, 'w> UiStatusPanel<'g, 'a, 'w> {
    pub fn score(mut self) -> Self {
        self.score = true;
        self
    }

    pub fn player_health(mut self) -> Self {
        self.player_health = true;
        self
    }

    pub fn enemy_count(mut self) -> Self {
        self.enemy_count = true;
        self
    }

    pub fn build(self) {
        let mut lines = Vec::new();
        if self.score {
            lines.push(format!("Score: {}", self.game.score().value()));
        }
        if self.player_health {
            let health = self.game.player().health().unwrap_or_default();
            lines.push(format!("Health: {health}"));
        }
        if self.enemy_count {
            lines.push(format!("Enemies: {}", self.game.enemies().alive().count()));
        }
        if lines.is_empty() {
            return;
        }
        draw_text_panel(self.game, &lines, vec2(140.0, 70.0));
    }
}

impl<'g, 'a, 'w> UiPanel<'g, 'a, 'w> {
    pub fn line(mut self, text: impl AsRef<str>) -> Self {
        self.lines.push(text.as_ref().to_owned());
        self
    }

    /// Draws the panel in the centre of the current viewport.
    pub fn center(self) {
        let viewport = self.game.input().viewport_size();
        let center = if viewport.x > 0.0 && viewport.y > 0.0 {
            viewport * 0.5
        } else {
            vec2(400.0, 300.0)
        };
        draw_text_panel(self.game, &self.lines, center);
    }

    /// Draws the panel at a custom screen-space centre position.
    pub fn at(self, center: Vec2) {
        draw_text_panel(self.game, &self.lines, center);
    }

    /// Draws a dialog/panel in the centre of the current viewport.
    pub fn build(self) {
        self.center();
    }
}

fn draw_text_panel(game: &mut GameCtx<'_, '_>, lines: &[String], center: Vec2) {
    let content_width = lines
        .iter()
        .map(|line| estimated_width(line))
        .fold(0.0_f32, f32::max);
    let size = vec2(
        content_width + PANEL_PADDING.x * 2.0,
        lines.len().max(1) as f32 * LINE_HEIGHT + PANEL_PADDING.y * 2.0,
    );
    let origin = center - size * 0.5;
    game.ui_rect_at_layer(
        origin - Vec2::splat(2.0),
        size + Vec2::splat(4.0),
        PANEL_BORDER,
        9_900,
    );
    game.ui_rect_at_layer(origin, size, PANEL_BACKGROUND, 9_901);
    for (index, line) in lines.iter().enumerate() {
        let position = origin + PANEL_PADDING + vec2(0.0, index as f32 * LINE_HEIGHT + 4.0);
        game.text(line, position, TEXT_COLOR);
    }
}

fn estimated_width(text: &str) -> f32 {
    text.chars().count() as f32 * ESTIMATED_GLYPH_WIDTH
}

/// One custom text label returned by [`UiOps::text`].
pub struct UiText<'g, 'a, 'w> {
    game: &'g mut GameCtx<'a, 'w>,
    text: String,
    color: glam::Vec4,
    width: Option<f32>,
}

impl<'g, 'a, 'w> UiText<'g, 'a, 'w> {
    pub fn color(mut self, color: glam::Vec4) -> Self {
        self.color = color;
        self
    }

    /// Wraps text using the bitmap UI's fixed-width approximation.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width.max(ESTIMATED_GLYPH_WIDTH));
        self
    }

    pub fn at(self, position: Vec2) {
        let lines = self
            .width
            .map(|width| wrap_text(&self.text, width))
            .unwrap_or_else(|| vec![self.text]);
        for (index, line) in lines.iter().enumerate() {
            self.game.text(
                line,
                position + vec2(0.0, index as f32 * LINE_HEIGHT),
                self.color,
            );
        }
    }
}

fn wrap_text(text: &str, width: f32) -> Vec<String> {
    let max_chars = (width / ESTIMATED_GLYPH_WIDTH).floor().max(1.0) as usize;
    let mut lines = Vec::new();
    for paragraph in text.split('\n') {
        let mut line = String::new();
        for word in paragraph.split_whitespace() {
            let separator = usize::from(!line.is_empty());
            if line.chars().count() + separator + word.chars().count() > max_chars
                && !line.is_empty()
            {
                lines.push(std::mem::take(&mut line));
            }
            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(word);
        }
        lines.push(line);
    }
    lines
}

impl<'a, 'w> GameCtx<'a, 'w> {
    /// Starts a high-level beginner UI chain.
    pub fn ui(&mut self) -> UiOps<'_, 'a, 'w> {
        UiOps::new(self)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use game_core::backend::TextureHandle;
    use glam::vec2;

    use super::{ESTIMATED_GLYPH_WIDTH, UiFocus, wrap_text};
    use crate::app::{GameApp, GamePlugin};
    use crate::harness::GameTestHarness;

    struct UiPlugin {
        clicks: Rc<Cell<u32>>,
    }

    impl GamePlugin for UiPlugin {
        fn build(&self, game: &mut GameApp<'_>) -> anyhow::Result<()> {
            let controls = game.input(|input| input.top_down_controls())?;
            game.map("ui")
                .tiles(["..."])
                .simple_theme(TextureHandle(0), TextureHandle(0))
                .start();
            game.on_start(|game| game.spawn_start_map());

            let clicks = Rc::clone(&self.clicks);
            game.draw_ui(move |game, _dt| {
                game.ui()
                    .top_center()
                    .top_left_text("Top centre")
                    .bottom_left()
                    .top_left_text("Bottom left")
                    .build();
                game.ui()
                    .top_left()
                    .row()
                    .top_left_text("Horizontal one")
                    .top_left_text("Horizontal two")
                    .build();
                game.ui()
                    .top_left()
                    .column()
                    .spacer(16.0)
                    .text_box("Wrapped beginner UI text")
                    .width(128.0)
                    .at(vec2(24.0, 180.0));
                game.ui().center_panel("Centre panel").build();
                game.ui()
                    .panel("Custom panel")
                    .line("A second line")
                    .center();
                game.ui()
                    .dialog("Old Man")
                    .line("Welcome to the arena.")
                    .build();
                game.ui()
                    .status_panel()
                    .score()
                    .player_health()
                    .enemy_count()
                    .build();
                game.ui()
                    .press_to_start(controls.attack)
                    .press_to_restart(controls.reset)
                    .game_over_panel(controls.reset)
                    .pause_panel(controls.pause)
                    .build();
                let clicks = Rc::clone(&clicks);
                game.ui()
                    .button("Restart")
                    .at_center()
                    .on_click(move |_| clicks.set(clicks.get() + 1));
                let _ = game
                    .ui()
                    .button("Clicked API")
                    .at(vec2(400.0, 390.0))
                    .clicked();
            });
            Ok(())
        }
    }

    #[test]
    fn text_panels_layout_helpers_and_buttons_draw_and_click() {
        let clicks = Rc::new(Cell::new(0));
        let mut game = GameTestHarness::from_plugin(UiPlugin {
            clicks: Rc::clone(&clicks),
        })
        .unwrap()
        .click_mouse_left_at(vec2(400.0, 300.0), vec2(800.0, 600.0));

        game.frame(0.0);
        for expected in [
            "Top centre",
            "Bottom left",
            "Horizontal one",
            "Horizontal two",
            "Wrapped",
            "beginner",
            "UI text",
            "Centre panel",
            "Custom panel",
            "Old Man",
            "Welcome to the arena.",
            "Score: 0",
            "Health: 0",
            "Enemies: 0",
            "Press Space, Enter, or South to Start",
            "Press R to Restart",
            "Game Over",
            "Paused",
            "Restart",
        ] {
            game.assert_ui_contains(expected);
        }
        assert_eq!(clicks.get(), 1);
    }

    struct MenuPlugin;

    impl GamePlugin for MenuPlugin {
        fn build(&self, game: &mut GameApp<'_>) -> anyhow::Result<()> {
            let controls = game.input(|input| input.top_down_controls())?;
            game.map("menu")
                .tiles(["..."])
                .simple_theme(TextureHandle(0), TextureHandle(0))
                .start();
            game.start_scene("menu").scene("game");
            game.use_top_down_game().controls(controls).build();
            game.draw_ui(|game, _dt| {
                if game.current_scene_name().as_deref() == Some("menu") {
                    game.ui()
                        .menu("Main Menu")
                        .button("Start")
                        .go_to_scene("game")
                        .button("Quit")
                        .quit()
                        .build();
                }
            });
            Ok(())
        }
    }

    #[test]
    fn focused_menu_uses_standard_navigation_and_accept_controls() {
        let mut game = GameTestHarness::from_plugin(MenuPlugin).unwrap();
        game.frame(0.0);
        game.assert_ui_contains("Main Menu");
        assert_eq!(
            game.world()
                .get_resource::<UiFocus>()
                .unwrap()
                .selected_index,
            0
        );

        game = game.press_action("menu_down");
        game.frame(0.0);
        game.clear_input();
        assert_eq!(
            game.world()
                .get_resource::<UiFocus>()
                .unwrap()
                .selected_index,
            1
        );

        game = game.press_action("menu_up");
        game.frame(0.0);
        game.clear_input();
        game = game.press_action("menu_accept");
        game.frame(0.0);

        assert_eq!(game.current_scene().as_deref(), Some("game"));
    }

    #[test]
    fn text_wrapping_uses_the_bitmap_width_approximation() {
        assert_eq!(
            wrap_text("one two three", ESTIMATED_GLYPH_WIDTH * 7.0),
            ["one two".to_owned(), "three".to_owned()]
        );
        assert_eq!(
            wrap_text("one\ntwo", 100.0),
            ["one".to_owned(), "two".to_owned()]
        );
    }
}

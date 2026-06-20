//! High-level beginner UI helpers.

use glam::{Vec2, vec2, vec4};

use crate::context::GameCtx;

const MARGIN: f32 = 24.0;
const LINE_HEIGHT: f32 = 28.0;
const TEXT_COLOR: glam::Vec4 = vec4(1.0, 0.95, 0.75, 1.0);

/// Immediate-mode helpers for common beginner labels.
pub struct UiOps<'g, 'a, 'w> {
    game: &'g mut GameCtx<'a, 'w>,
    next_line: Vec2,
}

impl<'g, 'a, 'w> UiOps<'g, 'a, 'w> {
    pub(crate) fn new(game: &'g mut GameCtx<'a, 'w>) -> Self {
        Self {
            game,
            next_line: vec2(MARGIN, MARGIN),
        }
    }

    pub fn top_left(mut self) -> Self {
        self.next_line = vec2(MARGIN, MARGIN);
        self
    }

    pub fn text(self, text: impl Into<String>) -> UiText<'g, 'a, 'w> {
        UiText {
            game: self.game,
            text: text.into(),
            color: TEXT_COLOR,
        }
    }

    pub fn top_left_text(mut self, text: impl AsRef<str>) -> Self {
        self.draw_line(text.as_ref(), TEXT_COLOR);
        self
    }

    pub fn center_text(self, text: impl AsRef<str>) -> Self {
        let viewport = self.game.input().viewport_size();
        let position = if viewport.x > 0.0 && viewport.y > 0.0 {
            viewport * 0.5
        } else {
            vec2(400.0, 300.0)
        };
        self.game.text(text.as_ref(), position, TEXT_COLOR);
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
        self.game.text(text, self.next_line, color);
        self.next_line.y += LINE_HEIGHT;
    }
}

/// One custom text label returned by [`UiOps::text`].
pub struct UiText<'g, 'a, 'w> {
    game: &'g mut GameCtx<'a, 'w>,
    text: String,
    color: glam::Vec4,
}

impl<'g, 'a, 'w> UiText<'g, 'a, 'w> {
    pub fn color(mut self, color: glam::Vec4) -> Self {
        self.color = color;
        self
    }

    pub fn at(self, position: Vec2) {
        self.game.text(&self.text, position, self.color);
    }
}

impl<'a, 'w> GameCtx<'a, 'w> {
    /// Starts a high-level beginner UI chain.
    pub fn ui(&mut self) -> UiOps<'_, 'a, 'w> {
        UiOps::new(self)
    }
}

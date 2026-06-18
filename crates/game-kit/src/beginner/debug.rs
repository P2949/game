//! Beginner debug overlay helpers.

use glam::{vec2, vec4};

use crate::beginner::actors::{Enemy, Name, Player};
use crate::context::GameCtx;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DebugOverlay {
    pub enabled: bool,
    pub show_colliders: bool,
    pub show_nav: bool,
    pub show_names: bool,
    pub show_fps: bool,
}

impl DebugOverlay {
    pub fn enabled() -> Self {
        Self {
            enabled: true,
            show_colliders: false,
            show_nav: false,
            show_names: false,
            show_fps: true,
        }
    }
}

impl Default for DebugOverlay {
    fn default() -> Self {
        Self {
            enabled: false,
            show_colliders: false,
            show_nav: false,
            show_names: false,
            show_fps: true,
        }
    }
}

pub fn draw_debug_overlay(game: &mut GameCtx<'_, '_>, dt: f32) {
    let overlay = game.resource::<DebugOverlay>().copied().unwrap_or_default();
    if !overlay.enabled {
        return;
    }

    let mut lines = Vec::new();
    lines.push(format!(
        "map: {}",
        game.current_map_name()
            .unwrap_or_else(|| "<none>".to_owned())
    ));
    lines.push(format!(
        "entities: {}",
        game.entities_with::<game_core::world::Transform>().len()
    ));
    lines.push(format!("enemies: {}", game.entities_with::<Enemy>().len()));

    if let Some(player) = game.first_entity_with::<Player>() {
        if let Some(health) = game.component::<game_combat::Health>(player) {
            lines.push(format!("player hp: {}/{}", health.current, health.max));
        }
    }

    if overlay.show_fps && dt > 0.0 {
        lines.push(format!("fps: {:.0}", 1.0 / dt));
    }
    if overlay.show_colliders {
        lines.push("colliders: on".to_owned());
    }
    if overlay.show_nav {
        lines.push("nav: on".to_owned());
    }
    if overlay.show_names {
        let names = game
            .entities_with::<Name>()
            .into_iter()
            .filter_map(|id| game.component::<Name>(id))
            .map(|name| name.as_str().to_owned())
            .take(6)
            .collect::<Vec<_>>();
        if !names.is_empty() {
            lines.push(format!("names: {}", names.join(", ")));
        }
    }

    game.text(
        &lines.join("\n"),
        vec2(16.0, 16.0),
        vec4(0.78, 1.0, 0.70, 1.0),
    );
}

impl<'a, 'w> GameCtx<'a, 'w> {
    pub fn toggle_debug_overlay(&mut self) {
        let mut overlay = self.resource::<DebugOverlay>().copied().unwrap_or_default();
        overlay.enabled = !overlay.enabled;
        self.insert_resource(overlay);
    }
}

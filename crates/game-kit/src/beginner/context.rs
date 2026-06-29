//! Beginner-friendly context wrappers.

use anyhow::Result;
use game_core::input::{ActionId, Axis2dId};
use glam::Vec2;

use crate::beginner::audio::AudioOps;
use crate::beginner::collections::{
    CameraOps, EnemyCollection, PickupCollection, PlayerActor, ScoreOps, TaggedActors,
};
use crate::beginner::spawn::SpawnAuthor;
use crate::beginner::ui::UiOps;
use crate::context::{GameCtx, StartupGameCtx};
use crate::data::BeginnerRuleUiText;

/// Seconds elapsed in a beginner callback.
pub type Seconds = f32;

/// Limited, game-shaped context handed to beginner callbacks.
///
/// This wrapper intentionally exposes player/enemy/pickup/map/scene/audio/UI
/// verbs instead of raw entity/component/resource access. Advanced systems can
/// keep using [`GameCtx`] through `game_kit::advanced::prelude::*`.
pub struct Game<'g, 'a, 'w> {
    ctx: &'g mut GameCtx<'a, 'w>,
}

impl<'g, 'a, 'w> Game<'g, 'a, 'w> {
    pub(crate) fn new(ctx: &'g mut GameCtx<'a, 'w>) -> Self {
        Self { ctx }
    }

    /// True if `action` was pressed this frame.
    pub fn pressed(&self, action: ActionId) -> bool {
        self.ctx.pressed(action)
    }

    /// True while `action` is held.
    pub fn down(&self, action: ActionId) -> bool {
        self.ctx.down(action)
    }

    /// Current value of a logical 2D axis.
    pub fn axis2d(&self, axis: Axis2dId) -> Vec2 {
        self.ctx.axis2d(axis)
    }

    /// High-level player actor operations.
    pub fn player(&mut self) -> PlayerActor<'_, 'a, 'w> {
        self.ctx.player()
    }

    /// High-level enemy collection operations.
    pub fn enemies(&mut self) -> EnemyCollection<'_, 'a, 'w> {
        self.ctx.enemies()
    }

    /// High-level pickup collection operations.
    pub fn pickups(&mut self) -> PickupCollection<'_, 'a, 'w> {
        self.ctx.pickups()
    }

    /// Actors carrying a content-authored tag.
    pub fn actors_tagged(&mut self, tag: &str) -> TaggedActors<'_, 'a, 'w> {
        self.ctx.actors_tagged(tag)
    }

    /// Begins a beginner prefab spawn operation.
    pub fn spawn(&mut self, prefab: impl Into<String>) -> SpawnAuthor<'_, 'a, 'w> {
        self.ctx.spawn(prefab)
    }

    /// High-level score operations.
    pub fn score(&mut self) -> ScoreOps<'_, 'a, 'w> {
        self.ctx.score()
    }

    /// High-level camera operations.
    pub fn camera(&mut self) -> CameraOps<'_, 'a, 'w> {
        self.ctx.camera2d()
    }

    /// Backwards-compatible alias for [`Self::camera`].
    pub fn camera2d(&mut self) -> CameraOps<'_, 'a, 'w> {
        self.camera()
    }

    /// Named audio operations.
    pub fn audio(&mut self) -> AudioOps<'_, 'a, 'w> {
        self.ctx.audio()
    }

    /// Immediate-mode beginner UI operations.
    pub fn ui(&mut self) -> UiOps<'_, 'a, 'w> {
        self.ctx.ui()
    }

    /// Plays a registered sound effect by key.
    pub fn play_sound_named(&mut self, key: &str) {
        self.ctx.play_sound_named(key);
    }

    pub(crate) fn show_rule_text(&mut self, text: &str) {
        let text = text.trim();
        if text.is_empty() {
            return;
        }
        let lines = &mut self
            .ctx
            .resource_or_insert_with(BeginnerRuleUiText::default)
            .lines;
        if !lines.iter().any(|line| line == text) {
            lines.push(text.to_owned());
        }
    }

    /// Starts registered music by key.
    pub fn play_music_named(&mut self, key: &str) {
        self.ctx.play_music_named(key);
    }

    /// The active scene name, if scenes are configured.
    pub fn current_scene_name(&self) -> Option<String> {
        self.ctx.current_scene_name()
    }

    /// Changes to a named scene.
    pub fn change_scene(&mut self, scene: &str) -> Result<()> {
        self.ctx.change_scene(scene)
    }

    /// Changes to a named scene, logging any failure.
    pub fn change_scene_or_log(&mut self, scene: &str) {
        self.ctx.change_scene_or_log(scene);
    }

    /// Changes to a named map.
    pub fn change_map(&mut self, map: &str) -> Result<()> {
        self.ctx.change_map(map)
    }

    /// Changes to a named map, logging any failure.
    pub fn change_map_or_log(&mut self, map: &str) {
        self.ctx.change_map_or_log(map);
    }

    /// Restarts the currently active map, logging any failure.
    pub fn restart_current_map_or_log(&mut self) {
        self.ctx.restart_map_or_log();
    }

    /// Restarts the currently active map, logging any failure.
    pub fn restart_map_or_log(&mut self) {
        self.restart_current_map_or_log();
    }

    /// Restarts the current level, logging any failure.
    pub fn restart_level(&mut self) {
        self.ctx.restart_level();
    }

    /// Requests that the running game close after the current update.
    pub fn quit(&mut self) {
        self.ctx.quit();
    }

    /// The author name of the currently spawned map, if available.
    pub fn current_map_name(&self) -> Option<String> {
        self.ctx.current_map_name()
    }
}

/// Limited startup context handed to beginner startup callbacks.
pub struct StartupGame<'g, 'a, 'w> {
    ctx: &'g mut StartupGameCtx<'a, 'w>,
}

impl<'g, 'a, 'w> StartupGame<'g, 'a, 'w> {
    pub(crate) fn new(ctx: &'g mut StartupGameCtx<'a, 'w>) -> Self {
        Self { ctx }
    }

    /// Spawns the start map's objects.
    pub fn spawn_start_map(&mut self) -> Result<()> {
        self.ctx.spawn_start_map()
    }

    /// Sets the current scene before the first frame.
    pub fn start_scene(&mut self, scene: &str) {
        self.ctx.start_scene(scene);
    }
}

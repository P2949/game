//! Named, beginner-friendly audio controls.

use crate::context::GameCtx;

/// Audio controls reached through [`GameCtx::audio`]. Sound and music names are
/// declared through `game.asset_bag()` so gameplay never needs a raw handle.
/// WAV is always available; OGG Vorbis and MP3 assets are optional runtime
/// features. All supported formats play through the same named methods.
pub struct AudioOps<'game, 'ctx, 'world> {
    game: &'game mut GameCtx<'ctx, 'world>,
}

impl<'game, 'ctx, 'world> AudioOps<'game, 'ctx, 'world> {
    pub(crate) fn new(game: &'game mut GameCtx<'ctx, 'world>) -> Self {
        Self { game }
    }

    /// Prepares a sound effect at normal volume. Calling `.bus("ambience")`
    /// routes it through a named sub-mix; using the returned value as a
    /// statement keeps the normal SFX path.
    pub fn play_sound(self, key: &str) -> SoundPlayback<'game, 'ctx, 'world> {
        let sound = self.game.named_sound(key);
        if sound.is_none() {
            self.game.report_missing_named_sound(key);
        }
        SoundPlayback {
            game: self.game,
            sound,
            volume: 1.0,
            bus: None,
        }
    }

    /// Prepares looping music. Calling `.volume(0.5)` sets its starting gain;
    /// using the returned value as a statement starts it at normal volume.
    pub fn play_music(self, key: &str) -> MusicPlayback<'game, 'ctx, 'world> {
        let sound = self.game.named_sound(key);
        if sound.is_none() {
            self.game.report_missing_named_sound(key);
        }
        MusicPlayback {
            game: self.game,
            sound,
            volume: 1.0,
            fade_in_seconds: None,
        }
    }

    pub fn stop_music(&mut self) {
        self.game.stop_music();
    }

    /// Replaces the current music while blending the old and new tracks over
    /// `duration_seconds`.
    pub fn crossfade_music(&mut self, key: &str, duration_seconds: f32) {
        match self.game.named_sound(key) {
            Some(sound) => self.game.crossfade_music(sound, 1.0, duration_seconds),
            None => self.game.report_missing_named_sound(key),
        }
    }

    pub fn pause_music(&mut self) {
        self.game.pause_music();
    }

    pub fn resume_music(&mut self) {
        self.game.resume_music();
    }

    pub fn set_master_volume(&mut self, volume: f32) {
        self.game.set_master_volume(volume);
    }

    pub fn set_sfx_volume(&mut self, volume: f32) {
        self.game.set_sfx_volume(volume);
    }

    pub fn set_music_volume(&mut self, volume: f32) {
        self.game.set_music_volume(volume);
    }

    /// Begins configuring a named sound-effect bus. Custom bus levels multiply
    /// the standard SFX group, so master and SFX sliders keep working as usual.
    pub fn bus(&mut self, name: &str) -> AudioBus<'_, 'ctx, 'world> {
        AudioBus {
            game: self.game,
            name: name.to_owned(),
        }
    }

    pub fn fade_music_to(&mut self, volume: f32, duration_seconds: f32) {
        self.game.fade_music_to(volume, duration_seconds);
    }

    pub fn fade_music_in(&mut self, key: &str, duration_seconds: f32) {
        match self.game.named_sound(key) {
            Some(sound) => self.game.play_music_fade_in(sound, 1.0, duration_seconds),
            None => self.game.report_missing_named_sound(key),
        }
    }
}

/// Deferred sound-effect configuration returned by [`AudioOps::play_sound`].
/// It submits the request when the expression ends, allowing both ordinary
/// `play_sound("hit");` calls and `.bus("ambience")` routing.
pub struct SoundPlayback<'game, 'ctx, 'world> {
    game: &'game mut GameCtx<'ctx, 'world>,
    sound: Option<game_core::backend::SoundHandle>,
    volume: f32,
    bus: Option<String>,
}

impl SoundPlayback<'_, '_, '_> {
    pub fn volume(mut self, volume: f32) -> Self {
        self.volume = volume;
        self
    }

    pub fn bus(mut self, name: impl Into<String>) -> Self {
        self.bus = Some(name.into());
        self
    }
}

impl Drop for SoundPlayback<'_, '_, '_> {
    fn drop(&mut self) {
        let Some(sound) = self.sound else {
            return;
        };
        match self.bus.as_deref() {
            Some(bus) => self.game.play_sound_on_bus(sound, self.volume, bus),
            None => self.game.play_sound(sound, self.volume),
        }
    }
}

/// A named sound-effect bus reached through [`AudioOps::bus`].
pub struct AudioBus<'game, 'ctx, 'world> {
    game: &'game mut GameCtx<'ctx, 'world>,
    name: String,
}

impl AudioBus<'_, '_, '_> {
    pub fn volume(self, volume: f32) {
        self.game.set_bus_volume(&self.name, volume);
    }
}

/// Deferred music configuration returned by [`AudioOps::play_music`]. It submits
/// the request when the expression ends, so both `play_music("theme");` and
/// `play_music("theme").volume(0.4);` work naturally.
pub struct MusicPlayback<'game, 'ctx, 'world> {
    game: &'game mut GameCtx<'ctx, 'world>,
    sound: Option<game_core::backend::SoundHandle>,
    volume: f32,
    fade_in_seconds: Option<f32>,
}

impl MusicPlayback<'_, '_, '_> {
    pub fn volume(mut self, volume: f32) -> Self {
        self.volume = volume;
        self
    }

    pub fn fade_in(mut self, duration_seconds: f32) -> Self {
        self.fade_in_seconds = Some(duration_seconds);
        self
    }
}

impl Drop for MusicPlayback<'_, '_, '_> {
    fn drop(&mut self) {
        let Some(sound) = self.sound else {
            return;
        };
        match self.fade_in_seconds {
            Some(duration_seconds) => {
                self.game
                    .play_music_fade_in(sound, self.volume, duration_seconds);
            }
            None => self.game.play_music(sound, self.volume),
        }
    }
}

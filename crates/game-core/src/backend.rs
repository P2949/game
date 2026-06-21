//! Backend-facing handles, load requests, commands, and future backend traits.
//!
//! The runtime currently wires concrete SDL/audio/Vulkan crates directly. The
//! `RenderBackend`, `AudioBackend`, and `PlatformBackend` traits below document a
//! possible future seam for headless tests or alternate backends; they are not
//! used to drive the current runtime, and `game-kit` does not expose them to
//! content crates.

use crate::app::RenderFrame;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TextureHandle(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SoundHandle(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FontHandle(pub u32);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextureLoadRequest {
    pub path: String,
}

/// How a logical sound is produced.
///
/// File-backed WAV sounds are always loaded by the current audio runtime. OGG
/// Vorbis files are also supported when the runtime's optional `ogg` feature is
/// enabled. Both formats are normalized to the mixer output format at load time.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SoundLoadRequest {
    /// A runtime-synthesized sound effect, identified by a content-chosen name.
    Generated { name: String },
    /// A sound loaded from a supported audio file under the asset root.
    File { path: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FontLoadRequest {
    pub path: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderOutcome {
    Presented,
    Skipped,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AudioCommand {
    Play {
        sound: SoundHandle,
        volume: f32,
        looping: bool,
        /// An optional named sound-effect bus. `None` uses the standard SFX
        /// group, preserving the simple default path.
        bus: Option<String>,
    },
    PlayMusic {
        sound: SoundHandle,
        volume: f32,
        fade_in_seconds: Option<f32>,
    },
    /// Replaces music by fading the old track out while fading this one in.
    CrossfadeMusic {
        sound: SoundHandle,
        volume: f32,
        duration_seconds: f32,
    },
    StopMusic,
    PauseMusic,
    ResumeMusic,
    SetMasterVolume {
        volume: f32,
    },
    SetSfxVolume {
        volume: f32,
    },
    SetMusicVolume {
        volume: f32,
    },
    SetBusVolume {
        bus: String,
        volume: f32,
    },
    FadeMusicTo {
        volume: f32,
        duration_seconds: f32,
    },
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PlatformEvents {
    pub should_quit: bool,
}

/// Future renderer abstraction. Current runtime uses `game-renderer-vulkan`
/// directly.
pub trait RenderBackend {
    fn load_texture(&mut self, request: TextureLoadRequest) -> anyhow::Result<TextureHandle>;
    fn render(&mut self, frame: RenderFrame) -> anyhow::Result<RenderOutcome>;
}

/// Future audio abstraction. Current runtime uses `game-audio` directly.
pub trait AudioBackend {
    fn load_sound(&mut self, request: SoundLoadRequest) -> anyhow::Result<SoundHandle>;
    fn submit(&self, command: AudioCommand);
    fn poll_diagnostics(&self);
}

/// Future platform abstraction. Current runtime uses `game-platform-sdl`
/// directly.
pub trait PlatformBackend {
    fn pump_events(&mut self) -> PlatformEvents;
    fn drawable_size(&self) -> glam::UVec2;
    fn should_quit(&self) -> bool;
}

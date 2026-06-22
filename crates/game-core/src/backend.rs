//! Backend-facing handles, load requests, commands, and runtime backend traits.
//!
//! The runtime loop is generic over the three traits in this module. Concrete
//! SDL/Vulkan/audio implementations live in their respective backend crates,
//! while a lightweight headless implementation can exercise the same loop in
//! tests. Content crates do not need to name these traits.

use std::path::Path;

use crate::app::RenderFrame;
use crate::input::InputState;

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
    /// A long music track read in bounded chunks rather than decoded into memory
    /// at startup. The current streaming backend supports 48 kHz stereo PCM16
    /// WAV files; static `.music(...)` remains the simple path for all normal
    /// supported file formats.
    StreamedFile { path: String },
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

/// Rendering operations the runtime needs after a renderer has been created.
///
/// Asset registration assigns stable texture handles before the backend starts,
/// so runtime reloads replace known handles instead of inventing backend-owned
/// handles midway through a game.
pub trait RenderBackend {
    fn reload_textures(&mut self, textures: &[(TextureHandle, String)]) -> anyhow::Result<usize>;
    fn request_resize(&mut self);
    fn render(
        &mut self,
        drawable_size: glam::UVec2,
        frame: RenderFrame,
    ) -> anyhow::Result<RenderOutcome>;
}

/// Audio operations the runtime performs after startup.
pub trait AudioBackend {
    fn reload_sounds(
        &mut self,
        asset_root: &Path,
        sounds: &[(SoundHandle, SoundLoadRequest)],
    ) -> anyhow::Result<usize>;
    fn submit(&self, command: AudioCommand);
    fn poll_diagnostics(&self);
}

/// Window/event/input operations the runtime loop needs from its platform.
pub trait PlatformBackend {
    fn pump_events(&mut self) -> PlatformEvents;
    fn input(&self) -> &InputState;
    fn drawable_size(&self) -> glam::UVec2;
    fn take_stable_resize_request(&mut self) -> bool;
    fn should_quit(&self) -> bool;
    fn request_quit(&mut self);
}

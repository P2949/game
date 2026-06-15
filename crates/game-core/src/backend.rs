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

/// How a logical sound is produced. Audio is generated-only today, so most
/// content registers [`SoundLoadRequest::Generated`]; [`SoundLoadRequest::File`]
/// describes file-backed loading that the runtime does not implement yet but the
/// asset layer already models honestly (file sounds are validated on disk,
/// generated sounds are not).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SoundLoadRequest {
    /// A runtime-synthesized sound effect, identified by a content-chosen name.
    Generated { name: String },
    /// A sound loaded from a file under the asset root. Not yet played from the
    /// file by the runtime, but validated to exist.
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

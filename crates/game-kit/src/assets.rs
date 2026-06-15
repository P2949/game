//! Asset authoring (Phase 3).
//!
//! [`AssetAuthor`] names the textures/fonts/sounds a game uses without exposing
//! the engine's `AssetRegistry`. Reached through [`GameApp::assets`].

use game_core::assets::AssetRegistry;
use game_core::backend::{FontHandle, SoundHandle, TextureHandle};

/// Declares the assets a game depends on, returning stable handles content stores
/// in its own asset struct.
pub struct AssetAuthor<'a> {
    registry: &'a mut AssetRegistry,
}

impl<'a> AssetAuthor<'a> {
    pub(crate) fn new(registry: &'a mut AssetRegistry) -> Self {
        Self { registry }
    }

    /// Creates an `AssetAuthor` wrapping a raw `AssetRegistry`. Mainly useful for
    /// unit tests that create assets without a full `GameApp`.
    pub fn from_registry(registry: &'a mut AssetRegistry) -> Self {
        Self::new(registry)
    }

    /// A texture loaded from `path` (relative to the asset root), addressed by the
    /// content-chosen `key`.
    pub fn texture(&mut self, key: impl Into<String>, path: impl Into<String>) -> TextureHandle {
        self.registry.texture(key, path)
    }

    /// A font loaded from `path` (relative to the asset root).
    pub fn font(&mut self, key: impl Into<String>, path: impl Into<String>) -> FontHandle {
        self.registry.font(key, path)
    }

    /// A runtime-synthesized sound effect. Audio is generated-only today, so this
    /// is the sound API content reaches for.
    pub fn generated_sound(&mut self, key: impl Into<String>) -> SoundHandle {
        self.registry.generated_sound(key)
    }

    /// A file-backed sound. Validated on disk, but not yet played from the file by
    /// the runtime (audio is generated-only) — prefer [`Self::generated_sound`].
    pub fn sound_file(&mut self, key: impl Into<String>, path: impl Into<String>) -> SoundHandle {
        self.registry.sound_file(key, path)
    }
}

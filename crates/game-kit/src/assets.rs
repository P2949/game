//! Asset authoring (Phase 3).
//!
//! [`AssetAuthor`] names the textures/fonts/sounds a game uses without exposing
//! the engine's `AssetRegistry`. Reached through [`GameApp::assets`].

use anyhow::Result;
use game_core::assets::AssetRegistry;
use game_core::backend::{FontHandle, SoundHandle, TextureHandle};

use crate::beginner::animation::SpriteSheet;

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
    pub fn texture(
        &mut self,
        key: impl Into<String>,
        path: impl Into<String>,
    ) -> Result<TextureHandle> {
        self.registry.try_texture(key, path)
    }

    pub fn spritesheet(
        &mut self,
        key: impl Into<String>,
        path: impl Into<String>,
        columns: u32,
        rows: u32,
    ) -> Result<SpriteSheet> {
        let texture = self.texture(key, path)?;
        Ok(SpriteSheet::new(texture, columns, rows))
    }

    /// A font loaded from `path` (relative to the asset root).
    pub fn font(&mut self, key: impl Into<String>, path: impl Into<String>) -> Result<FontHandle> {
        self.registry.try_font(key, path)
    }

    /// A sound loaded from `path` (relative to the asset root).
    pub fn sound(
        &mut self,
        key: impl Into<String>,
        path: impl Into<String>,
    ) -> Result<SoundHandle> {
        self.registry.try_sound_file(key, path)
    }

    /// A runtime-synthesized sound effect.
    pub fn generated_sound(&mut self, key: impl Into<String>) -> Result<SoundHandle> {
        self.registry.try_generated_sound(key)
    }
}

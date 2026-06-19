//! Asset authoring (Phase 3).
//!
//! [`AssetAuthor`] names the textures/fonts/sounds a game uses without exposing
//! the engine's `AssetRegistry`. Reached through [`GameApp::assets`].

use std::collections::HashMap;

use anyhow::Result;
use game_core::assets::AssetRegistry;
use game_core::backend::{FontHandle, SoundHandle, TextureHandle};

use crate::beginner::animation::SpriteSheet;

/// Declares the assets a game depends on, returning stable handles content stores
/// in its own asset struct.
pub struct AssetAuthor<'a> {
    registry: &'a mut AssetRegistry,
}

#[derive(Clone, Debug, Default)]
pub struct AssetBag {
    textures: HashMap<String, TextureHandle>,
    sounds: HashMap<String, SoundHandle>,
    fonts: HashMap<String, FontHandle>,
    sheets: HashMap<String, SpriteSheet>,
}

impl AssetBag {
    pub fn texture(&self, key: &str) -> TextureHandle {
        self.try_texture(key)
            .unwrap_or_else(|| panic!("unknown texture asset '{key}'"))
    }

    pub fn sound(&self, key: &str) -> SoundHandle {
        self.try_sound(key)
            .unwrap_or_else(|| panic!("unknown sound asset '{key}'"))
    }

    pub fn font(&self, key: &str) -> FontHandle {
        self.try_font(key)
            .unwrap_or_else(|| panic!("unknown font asset '{key}'"))
    }

    pub fn spritesheet(&self, key: &str) -> SpriteSheet {
        self.try_spritesheet(key)
            .unwrap_or_else(|| panic!("unknown spritesheet asset '{key}'"))
    }

    pub fn try_texture(&self, key: &str) -> Option<TextureHandle> {
        self.textures.get(key).copied()
    }

    pub fn try_sound(&self, key: &str) -> Option<SoundHandle> {
        self.sounds.get(key).copied()
    }

    pub fn try_font(&self, key: &str) -> Option<FontHandle> {
        self.fonts.get(key).copied()
    }

    pub fn try_spritesheet(&self, key: &str) -> Option<SpriteSheet> {
        self.sheets.get(key).copied()
    }
}

pub struct AssetBagAuthor<'a> {
    author: AssetAuthor<'a>,
    bag: AssetBag,
}

impl<'a> AssetBagAuthor<'a> {
    pub(crate) fn new(author: AssetAuthor<'a>) -> Self {
        Self {
            author,
            bag: AssetBag::default(),
        }
    }

    pub fn texture(mut self, key: impl Into<String>, path: impl Into<String>) -> Result<Self> {
        let key = key.into();
        let handle = self.author.texture(key.clone(), path)?;
        self.bag.textures.insert(key, handle);
        Ok(self)
    }

    pub fn sound(mut self, key: impl Into<String>, path: impl Into<String>) -> Result<Self> {
        let key = key.into();
        let handle = self.author.sound(key.clone(), path)?;
        self.bag.sounds.insert(key, handle);
        Ok(self)
    }

    pub fn music(mut self, key: impl Into<String>, path: impl Into<String>) -> Result<Self> {
        let key = key.into();
        let handle = self.author.music(key.clone(), path)?;
        self.bag.sounds.insert(key, handle);
        Ok(self)
    }

    pub fn generated_sound(mut self, key: impl Into<String>) -> Result<Self> {
        let key = key.into();
        let handle = self.author.generated_sound(key.clone())?;
        self.bag.sounds.insert(key, handle);
        Ok(self)
    }

    pub fn font(mut self, key: impl Into<String>, path: impl Into<String>) -> Result<Self> {
        let key = key.into();
        let handle = self.author.font(key.clone(), path)?;
        self.bag.fonts.insert(key, handle);
        Ok(self)
    }

    pub fn spritesheet(
        mut self,
        key: impl Into<String>,
        path: impl Into<String>,
        columns: u32,
        rows: u32,
    ) -> Result<Self> {
        let key = key.into();
        let sheet = self.author.spritesheet(key.clone(), path, columns, rows)?;
        self.bag.sheets.insert(key, sheet);
        Ok(self)
    }

    pub fn build(self) -> AssetBag {
        self.bag
    }
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

    /// Music loaded from `path` (relative to the asset root). This returns a
    /// normal sound handle intended for `game.play_music(...)`.
    pub fn music(
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

#[cfg(test)]
mod tests {
    use game_core::assets::AssetRegistry;

    use super::{AssetAuthor, AssetBagAuthor};

    #[test]
    fn asset_bag_collects_declared_handles_by_key() {
        let mut registry = AssetRegistry::new();
        let bag = AssetBagAuthor::new(AssetAuthor::new(&mut registry))
            .texture("player", "textures/player.png")
            .unwrap()
            .sound("hit", "sounds/hit.wav")
            .unwrap()
            .music("theme", "music/theme.wav")
            .unwrap()
            .spritesheet("hero", "textures/hero.png", 4, 2)
            .unwrap()
            .build();

        assert!(bag.try_texture("player").is_some());
        assert!(bag.try_sound("hit").is_some());
        assert!(bag.try_sound("theme").is_some());
        assert_eq!(bag.spritesheet("hero").columns, 4);
        assert!(bag.try_texture("missing").is_none());
    }
}

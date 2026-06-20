//! Asset authoring (Phase 3).
//!
//! [`AssetAuthor`] names the textures/fonts/sounds a game uses without exposing
//! the engine's `AssetRegistry`. Reached through [`GameApp::assets`].

use std::collections::HashMap;

use anyhow::{Result, anyhow};
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

/// A texture supplied directly as a handle or deferred by its registered asset
/// key. Beginner builders resolve keys while they are built, producing the same
/// helpful diagnostics as [`AssetBag`].
#[derive(Clone, Debug)]
pub enum TextureRef {
    Handle(TextureHandle),
    Key(String),
}

impl From<TextureHandle> for TextureRef {
    fn from(handle: TextureHandle) -> Self {
        Self::Handle(handle)
    }
}

impl From<&str> for TextureRef {
    fn from(key: &str) -> Self {
        Self::Key(key.to_owned())
    }
}

impl From<String> for TextureRef {
    fn from(key: String) -> Self {
        Self::Key(key)
    }
}

/// A sound supplied directly as a handle or deferred by its registered asset
/// key. Sound and music share the same registry and handle type.
#[derive(Clone, Debug)]
pub enum SoundRef {
    Handle(SoundHandle),
    Key(String),
}

impl From<SoundHandle> for SoundRef {
    fn from(handle: SoundHandle) -> Self {
        Self::Handle(handle)
    }
}

impl From<&str> for SoundRef {
    fn from(key: &str) -> Self {
        Self::Key(key.to_owned())
    }
}

impl From<String> for SoundRef {
    fn from(key: String) -> Self {
        Self::Key(key)
    }
}

/// Runtime key-to-handle lookup installed by [`crate::GameApp`]. It lets
/// beginner systems play registered sounds by name without exposing the asset
/// registry or carrying an `AssetBag` into every callback.
#[derive(Clone, Debug, Default)]
pub(crate) struct AssetLookup {
    sounds: HashMap<String, SoundHandle>,
}

impl AssetLookup {
    pub(crate) fn from_registry(registry: &AssetRegistry) -> Self {
        Self {
            sounds: registry
                .sound_keys()
                .filter_map(|key| {
                    registry
                        .sound_handle(key)
                        .map(|handle| (key.to_owned(), handle))
                })
                .collect(),
        }
    }

    pub(crate) fn sound(&self, key: &str) -> Option<SoundHandle> {
        self.sounds.get(key).copied()
    }

    pub(crate) fn sound_error(&self, key: &str) -> anyhow::Error {
        missing_asset_error("sound", key, self.sounds.keys().map(String::as_str))
    }
}

impl AssetBag {
    /// Returns a texture by key.
    ///
    /// This is a convenience method that panics for a missing key. Beginner
    /// templates should prefer name-based prefab methods or [`Self::texture_result`]
    /// when a recoverable authoring diagnostic is useful.
    pub fn texture(&self, key: &str) -> TextureHandle {
        self.texture_result(key)
            .unwrap_or_else(|error| panic!("{error}"))
    }

    /// Returns a sound by key.
    ///
    /// This is a convenience method that panics for a missing key; use
    /// [`Self::sound_result`] for a recoverable authoring diagnostic.
    pub fn sound(&self, key: &str) -> SoundHandle {
        self.sound_result(key)
            .unwrap_or_else(|error| panic!("{error}"))
    }

    /// Returns a font by key.
    ///
    /// This is a convenience method that panics for a missing key; use
    /// [`Self::font_result`] for a recoverable authoring diagnostic.
    pub fn font(&self, key: &str) -> FontHandle {
        self.font_result(key)
            .unwrap_or_else(|error| panic!("{error}"))
    }

    /// Returns a spritesheet by key.
    ///
    /// This is a convenience method that panics for a missing key; use
    /// [`Self::spritesheet_result`] for a recoverable authoring diagnostic.
    pub fn spritesheet(&self, key: &str) -> SpriteSheet {
        self.spritesheet_result(key)
            .unwrap_or_else(|error| panic!("{error}"))
    }

    /// Looks up a texture and reports the registered keys plus a likely typo
    /// correction when it is missing.
    pub fn texture_result(&self, key: &str) -> Result<TextureHandle> {
        self.try_texture(key).ok_or_else(|| {
            missing_asset_error("texture", key, self.textures.keys().map(String::as_str))
        })
    }

    /// Looks up a sound and reports the registered keys plus a likely typo
    /// correction when it is missing.
    pub fn sound_result(&self, key: &str) -> Result<SoundHandle> {
        self.try_sound(key).ok_or_else(|| {
            missing_asset_error("sound", key, self.sounds.keys().map(String::as_str))
        })
    }

    /// Looks up a font and reports the registered keys plus a likely typo
    /// correction when it is missing.
    pub fn font_result(&self, key: &str) -> Result<FontHandle> {
        self.try_font(key)
            .ok_or_else(|| missing_asset_error("font", key, self.fonts.keys().map(String::as_str)))
    }

    /// Looks up a spritesheet and reports the registered keys plus a likely typo
    /// correction when it is missing.
    pub fn spritesheet_result(&self, key: &str) -> Result<SpriteSheet> {
        self.try_spritesheet(key).ok_or_else(|| {
            missing_asset_error("spritesheet", key, self.sheets.keys().map(String::as_str))
        })
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

pub(crate) fn missing_asset_error<'a>(
    kind: &str,
    key: &str,
    keys: impl Iterator<Item = &'a str>,
) -> anyhow::Error {
    let mut keys = keys.collect::<Vec<_>>();
    keys.sort_unstable();

    let known = if keys.is_empty() {
        "(none registered)".to_owned()
    } else {
        keys.iter()
            .map(|known| format!("- {known}"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let suggestion = closest_key(key, keys.iter().copied())
        .map(|candidate| format!("\n\nDid you mean '{candidate}'?"))
        .unwrap_or_default();

    anyhow!("Unknown {kind} asset '{key}'.\n\nKnown {kind} assets:\n{known}{suggestion}")
}

fn closest_key<'a>(needle: &str, keys: impl Iterator<Item = &'a str>) -> Option<&'a str> {
    let candidate = keys.min_by_key(|key| edit_distance(needle, key))?;
    let distance = edit_distance(needle, candidate);
    let threshold = (needle.chars().count().max(candidate.chars().count()) / 3).max(2);
    (distance <= threshold).then_some(candidate)
}

fn edit_distance(left: &str, right: &str) -> usize {
    let right = right.chars().collect::<Vec<_>>();
    let mut previous = (0..=right.len()).collect::<Vec<_>>();

    for (left_index, left_char) in left.chars().enumerate() {
        let mut current = vec![left_index + 1];
        for (right_index, right_char) in right.iter().enumerate() {
            let replace = previous[right_index] + usize::from(left_char != *right_char);
            let insert = current[right_index] + 1;
            let delete = previous[right_index + 1] + 1;
            current.push(replace.min(insert).min(delete));
        }
        previous = current;
    }

    previous[right.len()]
}

pub struct AssetBagAuthor<'a> {
    author: AssetAuthor<'a>,
    bag: AssetBag,
}

/// Registers common beginner asset folders using the asset key as the filename.
/// `textures(["player"])` maps to `assets/textures/player.png`, while
/// `sounds(["hit"])` maps to `assets/sounds/hit.wav`.
pub struct AssetFolderAuthor<'a> {
    bag: AssetBagAuthor<'a>,
}

impl<'a> AssetFolderAuthor<'a> {
    pub(crate) fn new(bag: AssetBagAuthor<'a>) -> Self {
        Self { bag }
    }

    pub fn textures<const N: usize>(mut self, keys: [&str; N]) -> Result<Self> {
        for key in keys {
            self.bag = self.bag.texture_auto(key)?;
        }
        Ok(self)
    }

    pub fn sounds<const N: usize>(mut self, keys: [&str; N]) -> Result<Self> {
        for key in keys {
            self.bag = self.bag.sound_auto(key)?;
        }
        Ok(self)
    }

    pub fn music<const N: usize>(mut self, keys: [&str; N]) -> Result<Self> {
        for key in keys {
            self.bag = self.bag.music_auto(key)?;
        }
        Ok(self)
    }

    pub fn build(self) -> AssetBag {
        self.bag.build()
    }
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

    /// Registers `assets/textures/<key>.png` under `key`.
    pub fn texture_auto(self, key: impl Into<String>) -> Result<Self> {
        let key = key.into();
        self.texture(key.clone(), format!("textures/{key}.png"))
    }

    pub fn sound(mut self, key: impl Into<String>, path: impl Into<String>) -> Result<Self> {
        let key = key.into();
        let handle = self.author.sound(key.clone(), path)?;
        self.bag.sounds.insert(key, handle);
        Ok(self)
    }

    /// Registers `assets/sounds/<key>.wav` under `key`.
    pub fn sound_auto(self, key: impl Into<String>) -> Result<Self> {
        let key = key.into();
        self.sound(key.clone(), format!("sounds/{key}.wav"))
    }

    pub fn music(mut self, key: impl Into<String>, path: impl Into<String>) -> Result<Self> {
        let key = key.into();
        let handle = self.author.music(key.clone(), path)?;
        self.bag.sounds.insert(key, handle);
        Ok(self)
    }

    /// Registers `assets/music/<key>.wav` under `key`.
    pub fn music_auto(self, key: impl Into<String>) -> Result<Self> {
        let key = key.into();
        self.music(key.clone(), format!("music/{key}.wav"))
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

    use super::{AssetAuthor, AssetBagAuthor, AssetFolderAuthor};

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

    #[test]
    fn asset_bag_reports_unknown_keys_with_registered_keys_and_suggestion() {
        let mut registry = AssetRegistry::new();
        let bag = AssetBagAuthor::new(AssetAuthor::new(&mut registry))
            .texture("player", "textures/player.png")
            .unwrap()
            .texture("floor", "textures/floor.png")
            .unwrap()
            .sound("hit", "sounds/hit.wav")
            .unwrap()
            .spritesheet("hero", "textures/hero.png", 4, 2)
            .unwrap()
            .build();

        assert!(bag.texture_result("player").is_ok());
        assert!(bag.sound_result("hit").is_ok());
        assert!(bag.spritesheet_result("hero").is_ok());

        let error = bag.texture_result("plaeyr").unwrap_err().to_string();
        assert!(error.contains("Unknown texture asset 'plaeyr'"));
        assert!(error.contains("Known texture assets:"));
        assert!(error.contains("- player"));
        assert!(error.contains("Did you mean 'player'?"));
    }

    #[test]
    fn conventional_asset_helpers_use_beginner_folder_paths() {
        let mut registry = AssetRegistry::new();
        let bag = AssetFolderAuthor::new(AssetBagAuthor::new(AssetAuthor::new(&mut registry)))
            .textures(["player", "floor"])
            .unwrap()
            .sounds(["hit"])
            .unwrap()
            .music(["theme"])
            .unwrap()
            .build();

        assert!(bag.try_texture("player").is_some());
        assert!(bag.try_texture("floor").is_some());
        assert!(bag.try_sound("hit").is_some());
        assert!(bag.try_sound("theme").is_some());
        assert_eq!(
            registry.texture_request("player").unwrap().path,
            "textures/player.png"
        );
        assert!(matches!(
            registry.sound_request("hit"),
            Some(game_core::backend::SoundLoadRequest::File { path }) if path == "sounds/hit.wav"
        ));
    }
}

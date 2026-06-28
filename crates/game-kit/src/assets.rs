//! Asset authoring (Phase 3).
//!
//! [`AssetAuthor`] names the textures/fonts/sounds a game uses without exposing
//! the engine's `AssetRegistry`. Reached through [`GameApp::assets`].

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
use game_core::assets::AssetRegistry;
use game_core::backend::{FontHandle, SoundHandle, TextureHandle};
use serde::Deserialize;

use crate::beginner::animation::{AnimationClip, AnimationSheet, SpriteSheet};

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
    animation_sheets: HashMap<String, AnimationSheet>,
}

/// A texture supplied directly as a handle or deferred by its registered asset
/// key. Beginner builders resolve keys while they are built, producing the same
/// helpful diagnostics as [`AssetBag`].
#[derive(Clone, Debug)]
pub enum TextureRef {
    Handle(TextureHandle),
    Key(String),
}

/// Values accepted by beginner `.sprite(...)` methods.
#[diagnostic::on_unimplemented(
    message = "`.sprite(...)` needs a texture name registered with `.texture(\"name\", \"path\")` or a `TextureHandle`",
    label = "this is not a texture reference"
)]
pub trait IntoTextureRef {
    fn into_texture_ref(self) -> TextureRef;
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

impl IntoTextureRef for TextureRef {
    fn into_texture_ref(self) -> TextureRef {
        self
    }
}

impl IntoTextureRef for TextureHandle {
    fn into_texture_ref(self) -> TextureRef {
        self.into()
    }
}

impl IntoTextureRef for &str {
    fn into_texture_ref(self) -> TextureRef {
        self.into()
    }
}

impl IntoTextureRef for String {
    fn into_texture_ref(self) -> TextureRef {
        self.into()
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

    /// Returns a spritesheet together with clips loaded from animation metadata.
    /// Pass it to `.animation_sheet(...)` on a player, enemy, or projectile
    /// prefab to avoid spelling out frame ranges in Rust.
    pub fn animation_sheet(&self, key: &str) -> AnimationSheet {
        self.animation_sheet_result(key)
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

    pub fn animation_sheet_result(&self, key: &str) -> Result<AnimationSheet> {
        self.try_animation_sheet(key).ok_or_else(|| {
            missing_asset_error(
                "animation sheet",
                key,
                self.animation_sheets.keys().map(String::as_str),
            )
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

    pub fn try_animation_sheet(&self, key: &str) -> Option<AnimationSheet> {
        self.animation_sheets.get(key).cloned()
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
/// `sounds(["hit"])` maps to `assets/sounds/hit.wav`. Explicit `.sound(...)`
/// and `.music(...)` paths may use WAV, optional OGG Vorbis, or optional MP3
/// files. `.streamed_music(...)` is for a long 48 kHz stereo PCM16 WAV track.
pub struct AssetFolderAuthor<'a> {
    bag: AssetBagAuthor<'a>,
}

impl<'a> AssetFolderAuthor<'a> {
    pub(crate) fn new(bag: AssetBagAuthor<'a>) -> Self {
        Self { bag }
    }

    /// Registers `assets/textures/<key>.png` under `key`.
    pub fn texture(mut self, key: impl Into<String>) -> Result<Self> {
        self.bag = self.bag.texture_auto(key)?;
        Ok(self)
    }

    /// Registers `assets/sounds/<key>.wav`, `<key>.ogg`, or `<key>.mp3`,
    /// preferring WAV then OGG. OGG and MP3 playback require their matching
    /// optional runtime features.
    pub fn sound(mut self, key: impl Into<String>) -> Result<Self> {
        self.bag = self.bag.sound_auto(key)?;
        Ok(self)
    }

    /// Registers `assets/music/<key>.wav`, `<key>.ogg`, or `<key>.mp3`,
    /// preferring WAV then OGG. OGG and MP3 playback require their matching
    /// optional runtime features.
    pub fn music(mut self, key: impl Into<String>) -> Result<Self> {
        self.bag = self.bag.music_auto(key)?;
        Ok(self)
    }

    /// Registers `assets/music/<key>.wav` as a bounded streamed music track.
    /// Streamed tracks intentionally use the strict WAV convention so their
    /// decoder can run on a background reader without buffering the full file.
    pub fn streamed_music(mut self, key: impl Into<String>) -> Result<Self> {
        let key = key.into();
        self.bag = self
            .bag
            .streamed_music(key.clone(), format!("music/{key}.wav"))?;
        Ok(self)
    }

    pub fn textures<const N: usize>(mut self, keys: [&str; N]) -> Result<Self> {
        for key in keys {
            self = self.texture(key)?;
        }
        Ok(self)
    }

    /// Validates and registers conventional PNG textures.
    ///
    /// Use this in a starter game when the listed files are required for the
    /// game to run. Missing files fail during setup with the path to add and a
    /// custom-path escape hatch.
    pub fn required_textures<const N: usize>(mut self, keys: [&str; N]) -> Result<Self> {
        for key in keys {
            validate_required_conventional_asset("texture", "textures", key, &["png"])?;
            self = self.texture(key)?;
        }
        Ok(self)
    }

    pub fn sounds<const N: usize>(mut self, keys: [&str; N]) -> Result<Self> {
        for key in keys {
            self = self.sound(key)?;
        }
        Ok(self)
    }

    /// Validates and registers conventional WAV, OGG, or MP3 sound effects.
    pub fn required_sounds<const N: usize>(mut self, keys: [&str; N]) -> Result<Self> {
        for key in keys {
            validate_required_conventional_asset("sound", "sounds", key, &["wav", "ogg", "mp3"])?;
            self = self.sound(key)?;
        }
        Ok(self)
    }

    pub fn music_tracks<const N: usize>(mut self, keys: [&str; N]) -> Result<Self> {
        for key in keys {
            self = self.music(key)?;
        }
        Ok(self)
    }

    /// Discovers PNG files directly under `assets/textures/` and registers each
    /// filename stem as its texture key. Prefer `required_textures(...)` in a
    /// tutorial or other setup that should fail for a missing expected file.
    pub fn discover_textures(mut self) -> Result<Self> {
        for (key, path) in discover_conventional_assets("textures", &["png"])? {
            self.bag = self.bag.texture(key, path)?;
        }
        Ok(self)
    }

    /// Discovers WAV, OGG, and MP3 files directly under `assets/sounds/`.
    /// When multiple supported formats share a filename stem, WAV wins, then
    /// OGG, then MP3, matching `.sound_auto(...)`.
    pub fn discover_sounds(mut self) -> Result<Self> {
        for (key, path) in discover_conventional_assets("sounds", &["wav", "ogg", "mp3"])? {
            self.bag = self.bag.sound(key, path)?;
        }
        Ok(self)
    }

    /// Discovers WAV, OGG, and MP3 files directly under `assets/music/`.
    /// When multiple supported formats share a filename stem, WAV wins, then
    /// OGG, then MP3, matching `.music_auto(...)`.
    pub fn discover_music(mut self) -> Result<Self> {
        for (key, path) in discover_conventional_assets("music", &["wav", "ogg", "mp3"])? {
            self.bag = self.bag.music(key, path)?;
        }
        Ok(self)
    }

    /// Registers `assets/animations/<key>.ron` as an animation sheet.
    pub fn animation_sheet_auto(mut self, key: impl Into<String>) -> Result<Self> {
        self.bag = self.bag.animation_sheet_auto(key)?;
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

    /// Registers the first conventional texture found for `key`. PNG is the
    /// current beginner convention and fallback.
    pub fn texture_auto(self, key: impl Into<String>) -> Result<Self> {
        let key = key.into();
        self.texture(
            key.clone(),
            conventional_asset_path("textures", &key, &["png"]),
        )
    }

    pub fn sound(mut self, key: impl Into<String>, path: impl Into<String>) -> Result<Self> {
        let key = key.into();
        let handle = self.author.sound(key.clone(), path)?;
        self.bag.sounds.insert(key, handle);
        Ok(self)
    }

    /// Registers the first conventional sound found for `key`: WAV first, then
    /// OGG, then MP3. Optional formats report a clear startup error until the
    /// matching runtime feature is enabled.
    pub fn sound_auto(self, key: impl Into<String>) -> Result<Self> {
        let key = key.into();
        self.sound(
            key.clone(),
            conventional_asset_path("sounds", &key, &["wav", "ogg", "mp3"]),
        )
    }

    pub fn music(mut self, key: impl Into<String>, path: impl Into<String>) -> Result<Self> {
        let key = key.into();
        let handle = self.author.music(key.clone(), path)?;
        self.bag.sounds.insert(key, handle);
        Ok(self)
    }

    /// Registers a long 48 kHz stereo PCM16 WAV track for bounded background
    /// streaming. It is played with the same `game.audio().play_music(...)`
    /// controls as ordinary music.
    pub fn streamed_music(
        mut self,
        key: impl Into<String>,
        path: impl Into<String>,
    ) -> Result<Self> {
        let key = key.into();
        let handle = self.author.streamed_music(key.clone(), path)?;
        self.bag.sounds.insert(key, handle);
        Ok(self)
    }

    /// Registers the first conventional music file found for `key`: WAV first,
    /// then OGG, then MP3. Optional formats require their matching runtime
    /// features.
    pub fn music_auto(self, key: impl Into<String>) -> Result<Self> {
        let key = key.into();
        self.music(
            key.clone(),
            conventional_asset_path("music", &key, &["wav", "ogg", "mp3"]),
        )
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

    /// Registers a spritesheet and named clips from a RON metadata document
    /// under `assets/<path>`. Use `assets.animation_sheet("player")` with a
    /// prefab's `.animation_sheet(...)` method afterwards.
    pub fn spritesheet_from_meta(
        mut self,
        key: impl Into<String>,
        path: impl Into<String>,
    ) -> Result<Self> {
        let key = key.into();
        let animation_sheet = self.author.spritesheet_from_meta(key.clone(), path)?;
        self.bag
            .sheets
            .insert(key.clone(), animation_sheet.spritesheet());
        self.bag.animation_sheets.insert(key, animation_sheet);
        Ok(self)
    }

    /// Registers `assets/animations/<key>.ron` as a spritesheet with named
    /// clips. Use `assets.animation_sheet("<key>")` with a prefab afterwards.
    pub fn animation_sheet_auto(self, key: impl Into<String>) -> Result<Self> {
        let key = key.into();
        self.spritesheet_from_meta(key.clone(), format!("animations/{key}.ron"))
    }

    pub fn build(self) -> AssetBag {
        self.bag
    }
}

fn conventional_asset_path(folder: &str, key: &str, extensions: &[&str]) -> String {
    let candidates = extensions
        .iter()
        .map(|extension| format!("{folder}/{key}.{extension}"))
        .collect::<Vec<_>>();
    select_conventional_path(&candidates, asset_path_exists)
}

fn validate_required_conventional_asset(
    kind: &str,
    folder: &str,
    key: &str,
    extensions: &[&str],
) -> Result<()> {
    let candidates = extensions
        .iter()
        .map(|extension| format!("{folder}/{key}.{extension}"))
        .collect::<Vec<_>>();
    if candidates
        .iter()
        .any(|candidate| asset_path_exists(candidate))
    {
        return Ok(());
    }

    let looked_for = candidates
        .iter()
        .map(|candidate| format!("- assets/{candidate}"))
        .collect::<Vec<_>>()
        .join("\n");
    let preferred = candidates
        .first()
        .expect("asset extensions must not be empty");
    let custom_extension = extensions
        .first()
        .expect("asset extensions must not be empty");
    anyhow::bail!(
        "Missing {kind} asset '{key}'.\n\nLooked for:\n{looked_for}\n\nFix:\n- add assets/{preferred}\n- or register a custom path:\n  game.asset_bag().{kind}(\"{key}\", \"some/path.{custom_extension}\")?"
    );
}

fn select_conventional_path(candidates: &[String], mut exists: impl FnMut(&str) -> bool) -> String {
    candidates
        .iter()
        .find(|candidate| exists(candidate))
        .cloned()
        .or_else(|| candidates.first().cloned())
        .unwrap_or_default()
}

fn asset_path_exists(relative: &str) -> bool {
    let root = std::env::var_os("GAME_ASSET_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| "assets".into());
    if root.is_absolute() {
        return root.join(relative).is_file();
    }
    let Ok(current_dir) = std::env::current_dir() else {
        return Path::new(&root).join(relative).is_file();
    };
    current_dir
        .ancestors()
        .any(|directory| directory.join(&root).join(relative).is_file())
}

fn beginner_asset_file(path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();
    if path.is_absolute() {
        return path.to_path_buf();
    }
    let root = std::env::var_os("GAME_ASSET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("assets"));
    if root.is_absolute() {
        return root.join(path);
    }
    let relative = root.join(path);
    let Ok(current_dir) = std::env::current_dir() else {
        return relative;
    };
    current_dir
        .ancestors()
        .map(|directory| directory.join(&relative))
        .find(|candidate| candidate.is_file())
        .unwrap_or_else(|| current_dir.join(relative))
}

fn beginner_asset_directory(folder: &str) -> PathBuf {
    let root = std::env::var_os("GAME_ASSET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("assets"));
    if root.is_absolute() {
        return root.join(folder);
    }
    let relative = root.join(folder);
    let Ok(current_dir) = std::env::current_dir() else {
        return relative;
    };
    current_dir
        .ancestors()
        .map(|directory| directory.join(&relative))
        .find(|candidate| candidate.is_dir())
        .unwrap_or_else(|| current_dir.join(relative))
}

fn discover_conventional_assets(
    folder: &str,
    extensions: &[&str],
) -> Result<Vec<(String, String)>> {
    let directory = beginner_asset_directory(folder);
    if !directory.is_dir() {
        return Ok(Vec::new());
    }

    let mut discovered = BTreeMap::<String, (usize, String)>::new();
    for entry in std::fs::read_dir(&directory).map_err(|error| {
        anyhow!(
            "Could not read asset folder '{}': {error}",
            directory.display()
        )
    })? {
        let entry = entry.map_err(|error| {
            anyhow!(
                "Could not inspect asset folder '{}': {error}",
                directory.display()
            )
        })?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(extension) = path.extension().and_then(|value| value.to_str()) else {
            continue;
        };
        let Some(priority) = extensions
            .iter()
            .position(|candidate| extension.eq_ignore_ascii_case(candidate))
        else {
            continue;
        };
        let Some(key) = path.file_stem().and_then(|value| value.to_str()) else {
            continue;
        };
        if key.is_empty() {
            continue;
        }
        let relative = format!("{folder}/{}", entry.file_name().to_string_lossy());
        match discovered.get(key) {
            Some((existing_priority, _)) if *existing_priority <= priority => {}
            _ => {
                discovered.insert(key.to_owned(), (priority, relative));
            }
        }
    }

    Ok(discovered
        .into_iter()
        .map(|(key, (_, path))| (key, path))
        .collect())
}

#[derive(Deserialize)]
struct AnimationSheetMetadata {
    texture: String,
    columns: u32,
    rows: u32,
    clips: HashMap<String, AnimationClipMetadata>,
}

#[derive(Deserialize)]
struct AnimationClipMetadata {
    frames: Vec<usize>,
    #[serde(default = "default_animation_fps")]
    fps: f32,
    #[serde(default = "default_animation_looping")]
    looping: bool,
}

type ParsedAnimationSheetMetadata = (String, u32, u32, Vec<(String, AnimationClip)>);

fn default_animation_fps() -> f32 {
    8.0
}

fn default_animation_looping() -> bool {
    true
}

fn parse_animation_sheet_metadata(
    path: &str,
    source: &str,
) -> Result<ParsedAnimationSheetMetadata> {
    let metadata: AnimationSheetMetadata = ron::from_str(source).map_err(|error| {
        anyhow!(
            "Animation metadata 'assets/{path}' is not valid RON: {error}.\n\nUse fields: texture, columns, rows, and clips."
        )
    })?;
    if metadata.columns == 0 || metadata.rows == 0 {
        anyhow::bail!("Animation metadata 'assets/{path}' needs non-zero columns and rows.");
    }
    let total_frames = metadata.columns as usize * metadata.rows as usize;
    let mut clips = metadata.clips.into_iter().collect::<Vec<_>>();
    clips.sort_by(|left, right| left.0.cmp(&right.0));
    let clips = clips
        .into_iter()
        .map(|(name, clip)| {
            if clip.frames.is_empty() {
                anyhow::bail!(
                    "Animation metadata 'assets/{path}' clip '{name}' has no frames."
                );
            }
            if let Some(frame) = clip.frames.iter().copied().find(|frame| *frame >= total_frames)
            {
                anyhow::bail!(
                    "Animation metadata 'assets/{path}' clip '{name}' uses frame {frame}, but the sheet has only {total_frames} frames."
                );
            }
            let animation = AnimationClip::frames(clip.frames).fps(clip.fps);
            let animation = if clip.looping {
                animation.looping()
            } else {
                animation.once()
            };
            Ok((name, animation))
        })
        .collect::<Result<Vec<_>>>()?;
    Ok((metadata.texture, metadata.columns, metadata.rows, clips))
}

fn resolve_metadata_texture_path(metadata_path: &str, texture: String) -> String {
    let texture_path = Path::new(&texture);
    if texture_path.is_absolute() {
        return texture;
    }

    let metadata_path = Path::new(metadata_path);
    if metadata_path.is_absolute()
        && let Some(asset_root) = metadata_path.parent().and_then(Path::parent)
    {
        return asset_root.join(texture_path).to_string_lossy().into_owned();
    }

    texture
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

    /// Loads `texture`, grid dimensions, and named clips from a small RON file.
    /// The metadata texture path is relative to the game asset folder, just like
    /// `.texture(...)` and `.spritesheet(...)` paths.
    pub fn spritesheet_from_meta(
        &mut self,
        key: impl Into<String>,
        path: impl Into<String>,
    ) -> Result<AnimationSheet> {
        let key = key.into();
        let path = path.into();
        let file = beginner_asset_file(&path);
        let source = std::fs::read_to_string(&file).map_err(|error| {
            anyhow!(
                "Could not read animation metadata 'assets/{path}' (looked for '{}'): {error}",
                file.display()
            )
        })?;
        let (texture, columns, rows, clips) = parse_animation_sheet_metadata(&path, &source)?;
        let texture = resolve_metadata_texture_path(&path, texture);
        let sheet = self.spritesheet(key, texture, columns, rows)?;
        Ok(AnimationSheet::new(sheet, clips))
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

    /// Registers long music for bounded, background streaming rather than
    /// decoding the whole track during startup. The current backend accepts a
    /// 48 kHz stereo PCM16 WAV file; use [`Self::music`] for ordinary WAV/OGG/
    /// MP3 tracks.
    pub fn streamed_music(
        &mut self,
        key: impl Into<String>,
        path: impl Into<String>,
    ) -> Result<SoundHandle> {
        self.registry.try_streamed_music_file(key, path)
    }

    /// A runtime-synthesized sound effect.
    pub fn generated_sound(&mut self, key: impl Into<String>) -> Result<SoundHandle> {
        self.registry.try_generated_sound(key)
    }
}

#[cfg(test)]
mod tests {
    use game_core::assets::AssetRegistry;

    use super::{
        AssetAuthor, AssetBagAuthor, AssetFolderAuthor, parse_animation_sheet_metadata,
        select_conventional_path, validate_required_conventional_asset,
    };

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
            .streamed_music("long_theme", "music/long_theme.wav")
            .unwrap()
            .spritesheet("hero", "textures/hero.png", 4, 2)
            .unwrap()
            .build();

        assert!(bag.try_texture("player").is_some());
        assert!(bag.try_sound("hit").is_some());
        assert!(bag.try_sound("theme").is_some());
        assert!(bag.try_sound("long_theme").is_some());
        assert!(matches!(
            registry.sound_request("long_theme"),
            Some(game_core::backend::SoundLoadRequest::StreamedFile { path })
                if path == "music/long_theme.wav"
        ));
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
            .music_tracks(["theme"])
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

    #[test]
    fn conventional_folder_lookup_prefers_existing_wav_then_ogg_and_has_a_stable_fallback() {
        let candidates = vec!["sounds/hit.wav".to_owned(), "sounds/hit.ogg".to_owned()];
        assert_eq!(
            select_conventional_path(&candidates, |path| path.ends_with(".ogg")),
            "sounds/hit.ogg"
        );
        assert_eq!(
            select_conventional_path(&candidates, |_| false),
            "sounds/hit.wav"
        );
    }

    #[test]
    fn singular_folder_helpers_register_named_texture_sound_and_music() {
        let mut registry = AssetRegistry::new();
        let bag = AssetFolderAuthor::new(AssetBagAuthor::new(AssetAuthor::new(&mut registry)))
            .texture("player")
            .unwrap()
            .sound("hit")
            .unwrap()
            .music("theme")
            .unwrap()
            .build();
        assert!(bag.try_texture("player").is_some());
        assert!(bag.try_sound("hit").is_some());
        assert!(bag.try_sound("theme").is_some());
    }

    #[test]
    fn discovery_helpers_register_conventional_workspace_assets_by_filename() {
        let mut registry = AssetRegistry::new();
        let bag = AssetFolderAuthor::new(AssetBagAuthor::new(AssetAuthor::new(&mut registry)))
            .discover_textures()
            .unwrap()
            .discover_sounds()
            .unwrap()
            .discover_music()
            .unwrap()
            .build();

        assert!(bag.try_texture("test").is_some());
        assert!(bag.try_sound("hit").is_some());
        assert_eq!(
            registry.texture_request("test").unwrap().path,
            "textures/test.png"
        );
        assert!(matches!(
            registry.sound_request("hit"),
            Some(game_core::backend::SoundLoadRequest::File { path }) if path == "sounds/hit.wav"
        ));
    }

    #[test]
    fn required_asset_diagnostics_name_the_conventional_path_and_escape_hatch() {
        let error = validate_required_conventional_asset(
            "texture",
            "textures",
            "missing_beginner_texture",
            &["png"],
        )
        .unwrap_err()
        .to_string();

        assert!(error.contains("Missing texture asset 'missing_beginner_texture'"));
        assert!(error.contains("assets/textures/missing_beginner_texture.png"));
        assert!(error.contains("game.asset_bag().texture"));
    }

    #[test]
    fn animation_metadata_creates_sorted_named_clips_without_rust_frame_ranges() {
        let (texture, columns, rows, clips) = parse_animation_sheet_metadata(
            "animations/player.ron",
            r#"(
                texture: "textures/player.png",
                columns: 4,
                rows: 1,
                clips: {
                    "walk_right": (frames: [3], fps: 10.0),
                    "attack_down": (frames: [1, 2], fps: 12.0, looping: false),
                },
            )"#,
        )
        .unwrap();

        assert_eq!(texture, "textures/player.png");
        assert_eq!((columns, rows), (4, 1));
        assert_eq!(clips[0].0, "attack_down");
        assert!(!clips[0].1.looping);
        assert_eq!(clips[1].0, "walk_right");
        assert!(clips[1].1.looping);
    }

    #[test]
    fn animation_metadata_explains_invalid_frame_numbers() {
        let error = parse_animation_sheet_metadata(
            "animations/player.ron",
            r#"(
                texture: "textures/player.png",
                columns: 2,
                rows: 1,
                clips: { "idle": (frames: [2]) },
            )"#,
        )
        .unwrap_err()
        .to_string();

        assert!(error.contains("clip 'idle' uses frame 2"));
        assert!(error.contains("only 2 frames"));
    }

    #[test]
    fn checked_in_animation_demo_metadata_matches_the_loader_format() {
        let (texture, columns, rows, clips) = parse_animation_sheet_metadata(
            "animations/player.ron",
            include_str!("../../../assets/animations/player.ron"),
        )
        .unwrap();

        assert_eq!(texture, "textures/test.png");
        assert_eq!((columns, rows), (4, 1));
        assert!(clips.iter().any(|(name, _)| name == "attack_right"));
    }
}

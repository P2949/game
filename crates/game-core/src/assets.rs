use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use crate::backend::{
    FontHandle, FontLoadRequest, SoundHandle, SoundLoadRequest, TextureHandle, TextureLoadRequest,
};

pub type AssetKey = String;

#[derive(Default)]
pub struct AssetRegistry {
    textures: HashMap<AssetKey, TextureLoadRequest>,
    sounds: HashMap<AssetKey, SoundLoadRequest>,
    fonts: HashMap<AssetKey, FontLoadRequest>,
    texture_handles: HashMap<AssetKey, TextureHandle>,
    sound_handles: HashMap<AssetKey, SoundHandle>,
    font_handles: HashMap<AssetKey, FontHandle>,
    texture_path_handles: HashMap<String, TextureHandle>,
    /// Deduplicates sound handles by identity (`file:<path>` / `gen:<name>`) so a
    /// single sound shared under several keys resolves to one handle.
    sound_identity_handles: HashMap<String, SoundHandle>,
    font_path_handles: HashMap<String, FontHandle>,
}

impl AssetRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Low-level convenience wrapper around [`Self::try_texture`] that panics on
    /// duplicate/conflicting keys. Content should use `game-kit::AssetAuthor`,
    /// which returns `Result`.
    pub fn texture(&mut self, key: impl Into<String>, path: impl Into<String>) -> TextureHandle {
        self.try_texture(key, path)
            .expect("texture asset keys must not be reused with different paths")
    }

    pub fn try_texture(
        &mut self,
        key: impl Into<String>,
        path: impl Into<String>,
    ) -> anyhow::Result<TextureHandle> {
        let key = key.into();
        let path = path.into();
        if let Some(request) = self.textures.get(&key) {
            if request.path != path {
                anyhow::bail!(
                    "Texture asset key '{}' already points to '{}', not '{}'.\n\nUse a different key, or reuse the same path for repeated calls to assets.texture(...).",
                    key,
                    request.path,
                    path
                );
            }
            return Ok(*self
                .texture_handles
                .get(&key)
                .expect("texture request and handle maps must stay in sync"));
        }

        let handle = if let Some(handle) = self.texture_path_handles.get(&path) {
            *handle
        } else {
            let handle = TextureHandle(self.texture_path_handles.len() as u32);
            self.texture_path_handles.insert(path.clone(), handle);
            handle
        };
        self.textures
            .insert(key.clone(), TextureLoadRequest { path });
        self.texture_handles.insert(key, handle);
        Ok(handle)
    }

    /// Low-level convenience wrapper around [`Self::try_generated_sound`] that
    /// panics on duplicate/conflicting keys. Content should use
    /// `game-kit::AssetAuthor`, which returns `Result`.
    pub fn generated_sound(&mut self, key: impl Into<String>) -> SoundHandle {
        self.try_generated_sound(key)
            .expect("sound asset keys must not be reused with a different source")
    }

    pub fn try_generated_sound(&mut self, key: impl Into<String>) -> anyhow::Result<SoundHandle> {
        let name = key.into();
        self.try_register_sound(name.clone(), SoundLoadRequest::Generated { name })
    }

    /// Low-level convenience wrapper around [`Self::try_sound_file`] that panics
    /// on duplicate/conflicting keys. Content should use
    /// `game-kit::AssetAuthor`, which returns `Result`.
    pub fn sound_file(&mut self, key: impl Into<String>, path: impl Into<String>) -> SoundHandle {
        self.try_sound_file(key, path)
            .expect("sound asset keys must not be reused with a different source")
    }

    pub fn try_sound_file(
        &mut self,
        key: impl Into<String>,
        path: impl Into<String>,
    ) -> anyhow::Result<SoundHandle> {
        self.try_register_sound(key.into(), SoundLoadRequest::File { path: path.into() })
    }

    /// Registers a long music track that the audio runtime reads in bounded
    /// chunks instead of decoding in full at startup.
    pub fn try_streamed_music_file(
        &mut self,
        key: impl Into<String>,
        path: impl Into<String>,
    ) -> anyhow::Result<SoundHandle> {
        self.try_register_sound(
            key.into(),
            SoundLoadRequest::StreamedFile { path: path.into() },
        )
    }

    fn try_register_sound(
        &mut self,
        key: String,
        request: SoundLoadRequest,
    ) -> anyhow::Result<SoundHandle> {
        if let Some(existing) = self.sounds.get(&key) {
            if existing != &request {
                anyhow::bail!(
                    "Sound asset key '{}' already points to {:?}, not {:?}.\n\nUse a different key, or reuse the same sound source for repeated calls to assets.sound(...).",
                    key,
                    existing,
                    request
                );
            }
            return Ok(*self
                .sound_handles
                .get(&key)
                .expect("sound request and handle maps must stay in sync"));
        }

        let identity = match &request {
            SoundLoadRequest::Generated { name } => format!("gen:{name}"),
            SoundLoadRequest::File { path } => format!("file:{path}"),
            SoundLoadRequest::StreamedFile { path } => format!("stream:{path}"),
        };
        let handle = if let Some(handle) = self.sound_identity_handles.get(&identity) {
            *handle
        } else {
            let handle = SoundHandle(self.sound_identity_handles.len() as u32);
            self.sound_identity_handles.insert(identity, handle);
            handle
        };
        self.sounds.insert(key.clone(), request);
        self.sound_handles.insert(key, handle);
        Ok(handle)
    }

    /// Low-level convenience wrapper around [`Self::try_font`] that panics on
    /// duplicate/conflicting keys. Content should use `game-kit::AssetAuthor`,
    /// which returns `Result`.
    pub fn font(&mut self, key: impl Into<String>, path: impl Into<String>) -> FontHandle {
        self.try_font(key, path)
            .expect("font asset keys must not be reused with different paths")
    }

    pub fn try_font(
        &mut self,
        key: impl Into<String>,
        path: impl Into<String>,
    ) -> anyhow::Result<FontHandle> {
        let key = key.into();
        let path = path.into();
        if let Some(request) = self.fonts.get(&key) {
            if request.path != path {
                anyhow::bail!(
                    "Font asset key '{}' already points to '{}', not '{}'.\n\nUse a different key, or reuse the same path for repeated calls to assets.font(...).",
                    key,
                    request.path,
                    path
                );
            }
            return Ok(*self
                .font_handles
                .get(&key)
                .expect("font request and handle maps must stay in sync"));
        }

        let handle = if let Some(handle) = self.font_path_handles.get(&path) {
            *handle
        } else {
            let handle = FontHandle(self.font_path_handles.len() as u32);
            self.font_path_handles.insert(path.clone(), handle);
            handle
        };
        self.fonts.insert(key.clone(), FontLoadRequest { path });
        self.font_handles.insert(key, handle);
        Ok(handle)
    }

    pub fn texture_request(&self, key: &str) -> Option<&TextureLoadRequest> {
        self.textures.get(key)
    }

    /// Resolves a content-facing texture key to its stable runtime handle.
    pub fn texture_handle(&self, key: &str) -> Option<TextureHandle> {
        self.texture_handles.get(key).copied()
    }

    pub fn sound_request(&self, key: &str) -> Option<&SoundLoadRequest> {
        self.sounds.get(key)
    }

    /// Resolves a content-facing sound or music key to its stable runtime handle.
    pub fn sound_handle(&self, key: &str) -> Option<SoundHandle> {
        self.sound_handles.get(key).copied()
    }

    pub fn font_request(&self, key: &str) -> Option<&FontLoadRequest> {
        self.fonts.get(key)
    }

    /// Resolves a content-facing font key to its stable runtime handle.
    pub fn font_handle(&self, key: &str) -> Option<FontHandle> {
        self.font_handles.get(key).copied()
    }

    /// Iterates over every registered texture key.
    pub fn texture_keys(&self) -> impl Iterator<Item = &str> {
        self.texture_handles.keys().map(String::as_str)
    }

    /// Iterates over every registered sound or music key.
    pub fn sound_keys(&self) -> impl Iterator<Item = &str> {
        self.sound_handles.keys().map(String::as_str)
    }

    /// Iterates over every registered font key.
    pub fn font_keys(&self) -> impl Iterator<Item = &str> {
        self.font_handles.keys().map(String::as_str)
    }

    pub fn texture_requests(&self) -> impl Iterator<Item = (&str, &TextureLoadRequest)> {
        self.textures
            .iter()
            .map(|(key, request)| (key.as_str(), request))
    }

    pub fn sound_requests(&self) -> impl Iterator<Item = (&str, &SoundLoadRequest)> {
        self.sounds
            .iter()
            .map(|(key, request)| (key.as_str(), request))
    }

    pub fn font_requests(&self) -> impl Iterator<Item = (&str, &FontLoadRequest)> {
        self.fonts
            .iter()
            .map(|(key, request)| (key.as_str(), request))
    }

    /// Every distinct texture the content needs, as `(handle, path)` pairs ordered
    /// by handle. The renderer loads these into GPU textures and records the
    /// resulting `TextureHandle -> TextureId` mapping, so handles resolve through a
    /// real lookup instead of being cast straight to renderer ids.
    pub fn texture_loads(&self) -> Vec<(TextureHandle, String)> {
        let mut loads: Vec<(TextureHandle, String)> = self
            .texture_path_handles
            .iter()
            .map(|(path, handle)| (*handle, path.clone()))
            .collect();
        loads.sort_by_key(|(handle, _)| handle.0);
        loads
    }

    pub fn sound_loads(&self) -> Vec<(SoundHandle, SoundLoadRequest)> {
        let mut loads = BTreeMap::new();
        for request in self.sounds.values() {
            let identity = match request {
                SoundLoadRequest::Generated { name } => format!("gen:{name}"),
                SoundLoadRequest::File { path } => format!("file:{path}"),
                SoundLoadRequest::StreamedFile { path } => format!("stream:{path}"),
            };
            let handle = *self
                .sound_identity_handles
                .get(&identity)
                .expect("sound request and identity maps must stay in sync");
            loads
                .entry(handle.0)
                .or_insert_with(|| (handle, request.clone()));
        }
        loads.into_values().collect()
    }
}

pub struct AssetValidator<'a> {
    registry: &'a AssetRegistry,
    root: PathBuf,
}

impl<'a> AssetValidator<'a> {
    pub fn new(registry: &'a AssetRegistry) -> Self {
        Self {
            registry,
            root: PathBuf::from("assets"),
        }
    }

    pub fn root(mut self, root: impl Into<PathBuf>) -> Self {
        self.root = root.into();
        self
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        for (key, request) in self.registry.texture_requests() {
            validate_file(&self.root, key, &request.path, "texture")?;
        }
        for (key, request) in self.registry.font_requests() {
            validate_file(&self.root, key, &request.path, "font")?;
        }
        // Generated sounds are synthesized at runtime and have no file to check;
        // only file-backed sounds are validated on disk.
        for (key, request) in self.registry.sound_requests() {
            if let SoundLoadRequest::File { path } | SoundLoadRequest::StreamedFile { path } =
                request
            {
                validate_file(&self.root, key, path, "sound")?;
            }
        }
        Ok(())
    }
}

fn validate_file(root: &Path, key: &str, path: &str, kind: &str) -> anyhow::Result<()> {
    let path = root.join(path);
    if !path.is_file() {
        let display_kind = title_case_ascii(kind);
        anyhow::bail!(
            "{} asset '{}' path '{}' does not exist.\n\nCheck the path relative to the assets directory, or register a different file with assets.{}(...).",
            display_kind,
            key,
            path.display(),
            kind
        );
    }
    Ok(())
}

fn title_case_ascii(value: &str) -> String {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    first.to_ascii_uppercase().to_string() + chars.as_str()
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::backend::SoundLoadRequest;

    use super::{AssetRegistry, AssetValidator};

    #[test]
    fn asset_registry_reuses_keys_and_records_requests() {
        let mut registry = AssetRegistry::new();
        let first = registry.texture("arena/floor", "textures/test.png");
        let second = registry.texture("arena/floor", "textures/test.png");
        let same_path = registry.texture("arena/wall", "textures/test.png");
        let other_path = registry.texture("arena/other", "textures/other.png");

        assert_eq!(first, second);
        assert_eq!(first, same_path);
        assert_ne!(first, other_path);
        assert_eq!(
            registry.texture_request("arena/floor").unwrap().path,
            "textures/test.png"
        );
    }

    #[test]
    fn asset_registry_rejects_conflicting_key_paths() {
        let mut registry = AssetRegistry::new();
        registry.texture("arena/floor", "textures/test.png");

        let err = registry
            .try_texture("arena/floor", "textures/other.png")
            .unwrap_err();

        assert!(err.to_string().contains("already points to"));
    }

    #[test]
    fn sound_and_font_handles_reuse_matching_sources() {
        let mut registry = AssetRegistry::new();
        let hit = registry.generated_sound("arena/hit");
        let hit_alias = registry.generated_sound("arena/hit");
        let ui = registry.font("ui/body", "fonts/DejaVuSans.ttf");
        let ui_alias = registry.font("ui/body_alias", "fonts/DejaVuSans.ttf");

        // Same generated identity -> same handle; distinct keys, same file -> same.
        assert_eq!(hit, hit_alias);
        assert_eq!(ui, ui_alias);
    }

    #[test]
    fn streamed_music_uses_a_distinct_bounded_reader_request() {
        let mut registry = AssetRegistry::new();
        let streamed = registry
            .try_streamed_music_file("theme", "music/theme.wav")
            .unwrap();
        let static_music = registry.sound_file("theme_static", "music/theme.wav");

        assert_ne!(streamed, static_music);
        assert!(matches!(
            registry.sound_request("theme"),
            Some(SoundLoadRequest::StreamedFile { path }) if path == "music/theme.wav"
        ));
    }

    #[test]
    fn asset_registry_resolves_handles_and_lists_registered_keys() {
        let mut registry = AssetRegistry::new();
        let texture = registry.texture("player", "textures/player.png");
        let sound = registry.generated_sound("hit");
        let font = registry.font("body", "fonts/body.ttf");

        assert_eq!(registry.texture_handle("player"), Some(texture));
        assert_eq!(registry.sound_handle("hit"), Some(sound));
        assert_eq!(registry.font_handle("body"), Some(font));
        assert_eq!(registry.texture_handle("missing"), None);

        assert_eq!(registry.texture_keys().collect::<Vec<_>>(), vec!["player"]);
        assert_eq!(registry.sound_keys().collect::<Vec<_>>(), vec!["hit"]);
        assert_eq!(registry.font_keys().collect::<Vec<_>>(), vec!["body"]);
    }

    #[test]
    fn asset_validator_checks_files_and_skips_generated_sounds() {
        let root = std::env::temp_dir().join(format!(
            "game-core-assets-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(root.join("textures")).unwrap();
        fs::write(root.join("textures/test.png"), b"fake").unwrap();

        let mut registry = AssetRegistry::new();
        registry.texture("arena/player", "textures/test.png");
        // Generated sounds have no file to validate.
        registry.generated_sound("arena/hit");

        AssetValidator::new(&registry)
            .root(&root)
            .validate()
            .unwrap();

        // A file-backed sound with a missing path must fail validation.
        registry.sound_file("arena/music", "audio/missing.wav");
        let err = AssetValidator::new(&registry)
            .root(&root)
            .validate()
            .unwrap_err();
        assert!(err.to_string().contains("audio/missing.wav"));

        fs::remove_dir_all(root).unwrap();
    }
}

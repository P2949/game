use std::collections::HashMap;
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
                    "texture asset key '{}' already points to '{}', not '{}'",
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
    /// on duplicate/conflicting keys. File-backed sounds are not exposed through
    /// `game-kit` until runtime playback exists.
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

    fn try_register_sound(
        &mut self,
        key: String,
        request: SoundLoadRequest,
    ) -> anyhow::Result<SoundHandle> {
        if let Some(existing) = self.sounds.get(&key) {
            if existing != &request {
                anyhow::bail!(
                    "sound asset key '{}' already points to {:?}, not {:?}",
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
                    "font asset key '{}' already points to '{}', not '{}'",
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

    pub fn sound_request(&self, key: &str) -> Option<&SoundLoadRequest> {
        self.sounds.get(key)
    }

    pub fn font_request(&self, key: &str) -> Option<&FontLoadRequest> {
        self.fonts.get(key)
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
            if let SoundLoadRequest::File { path } = request {
                validate_file(&self.root, key, path, "sound")?;
            }
        }
        Ok(())
    }
}

fn validate_file(root: &Path, key: &str, path: &str, kind: &str) -> anyhow::Result<()> {
    let path = root.join(path);
    if !path.is_file() {
        anyhow::bail!(
            "{kind} asset '{}' path '{}' does not exist",
            key,
            path.display()
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

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

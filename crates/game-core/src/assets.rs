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
    sound_path_handles: HashMap<String, SoundHandle>,
    font_path_handles: HashMap<String, FontHandle>,
}

impl AssetRegistry {
    pub fn new() -> Self {
        Self::default()
    }

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

    pub fn sound(&mut self, key: impl Into<String>, path: impl Into<String>) -> SoundHandle {
        self.try_sound(key, path)
            .expect("sound asset keys must not be reused with different paths")
    }

    pub fn try_sound(
        &mut self,
        key: impl Into<String>,
        path: impl Into<String>,
    ) -> anyhow::Result<SoundHandle> {
        let key = key.into();
        let path = path.into();
        if let Some(request) = self.sounds.get(&key) {
            if request.path != path {
                anyhow::bail!(
                    "sound asset key '{}' already points to '{}', not '{}'",
                    key,
                    request.path,
                    path
                );
            }
            return Ok(*self
                .sound_handles
                .get(&key)
                .expect("sound request and handle maps must stay in sync"));
        }

        let handle = if let Some(handle) = self.sound_path_handles.get(&path) {
            *handle
        } else {
            let handle = SoundHandle(self.sound_path_handles.len() as u32);
            self.sound_path_handles.insert(path.clone(), handle);
            handle
        };
        self.sounds.insert(key.clone(), SoundLoadRequest { path });
        self.sound_handles.insert(key, handle);
        Ok(handle)
    }

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
    allow_generated_sounds: bool,
}

impl<'a> AssetValidator<'a> {
    pub fn new(registry: &'a AssetRegistry) -> Self {
        Self {
            registry,
            root: PathBuf::from("assets"),
            allow_generated_sounds: false,
        }
    }

    pub fn root(mut self, root: impl Into<PathBuf>) -> Self {
        self.root = root.into();
        self
    }

    pub fn allow_generated_sounds(mut self) -> Self {
        self.allow_generated_sounds = true;
        self
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        for (key, request) in self.registry.texture_requests() {
            validate_file(&self.root, key, &request.path, "texture")?;
        }
        for (key, request) in self.registry.font_requests() {
            validate_file(&self.root, key, &request.path, "font")?;
        }
        for (key, request) in self.registry.sound_requests() {
            let path = self.root.join(&request.path);
            if !path.is_file() && !self.allow_generated_sounds {
                anyhow::bail!(
                    "sound asset '{}' path '{}' does not exist",
                    key,
                    path.display()
                );
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
    fn sound_and_font_handles_reuse_matching_paths() {
        let mut registry = AssetRegistry::new();
        let hit = registry.sound("arena/hit", "audio/hit.wav");
        let hit_alias = registry.sound("arena/hit_alias", "audio/hit.wav");
        let ui = registry.font("ui/body", "fonts/DejaVuSans.ttf");
        let ui_alias = registry.font("ui/body_alias", "fonts/DejaVuSans.ttf");

        assert_eq!(hit, hit_alias);
        assert_eq!(ui, ui_alias);
    }

    #[test]
    fn asset_validator_checks_texture_paths_and_allows_generated_sounds() {
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
        registry.sound("arena/hit", "audio/generated.wav");

        AssetValidator::new(&registry)
            .root(&root)
            .allow_generated_sounds()
            .validate()
            .unwrap();

        fs::remove_dir_all(root).unwrap();
    }
}

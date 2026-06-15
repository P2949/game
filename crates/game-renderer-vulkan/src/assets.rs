//! Loads the renderer's built-in GPU textures and font atlas. Split out of
//! `context.rs` so asset acquisition is isolated from device orchestration.
//!
//! Textures are registered into the caller's [`TextureRegistryGuard`] the moment
//! they are created, so a failure partway through can never leak an already-built
//! texture: the guard owns everything registered so far and destroys it on drop.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::Context;
use ash::vk;
use game_core::backend::TextureHandle;

use crate::texture::{Texture, TextureColorSpace, TextureUpload};
use crate::texture_registry::TextureRegistryGuard;
use crate::{FONT_TEXTURE_HANDLE, FONT_TEXTURE_ID, TextureId, text};

/// Creates and registers the renderer's built-in textures into `registry_guard`,
/// returning the font atlas metadata the renderer needs for text layout.
///
/// Each `create_and_register` call hands its freshly-created texture straight to
/// the registry, so the only state held across the fallible font-atlas build is
/// the guard itself — which already owns the test texture and will clean it up if
/// any later step (here or back in `VulkanContext::new`) fails.
/// Loads the renderer's textures into `registry_guard` and returns the font atlas
/// metadata plus the `TextureHandle -> TextureId` map that gameplay sprites are
/// resolved through. The font atlas is registered first so it owns the stable
/// [`FONT_TEXTURE_ID`]; the content textures follow in handle order. Each
/// `create_and_register` call hands its freshly-built texture straight to the
/// guard, so a failure partway through cleans up everything registered so far.
pub fn load_textures(
    registry_guard: &mut TextureRegistryGuard<'_>,
    descriptor_set_layout: vk::DescriptorSetLayout,
    queue: vk::Queue,
    upload_pool: vk::CommandPool,
    upload_fence: vk::Fence,
    content_textures: &[(TextureHandle, String)],
) -> anyhow::Result<(text::FontAtlas, HashMap<TextureHandle, TextureId>)> {
    let asset_root = asset_root()?;
    let mut handle_to_id = HashMap::new();

    let font_atlas_image =
        text::build_ascii_atlas(asset_root.join("fonts/DejaVuSans.ttf"), FONT_TEXTURE_HANDLE)?;
    let font_id = registry_guard.create_and_register(
        descriptor_set_layout,
        "font atlas",
        |device, allocator| {
            let mut upload = TextureUpload {
                device,
                allocator,
                queue,
                upload_pool,
                upload_fence,
            };
            Texture::from_rgba8(
                &mut upload,
                font_atlas_image.width,
                font_atlas_image.height,
                &font_atlas_image.pixels,
                TextureColorSpace::LinearData,
                "font atlas",
            )
        },
    )?;
    assert_eq!(
        font_id, FONT_TEXTURE_ID,
        "font atlas must be the first-registered texture so its id stays stable"
    );
    handle_to_id.insert(FONT_TEXTURE_HANDLE, font_id);

    for (handle, path) in content_textures {
        let texture_path = asset_root.join(path);
        let id = registry_guard.create_and_register(
            descriptor_set_layout,
            path.clone(),
            move |device, allocator| {
                let mut upload = TextureUpload {
                    device,
                    allocator,
                    queue,
                    upload_pool,
                    upload_fence,
                };
                Texture::from_path(
                    &mut upload,
                    texture_path,
                    TextureColorSpace::SrgbColor,
                    "content texture",
                )
            },
        )?;
        handle_to_id.insert(*handle, id);
    }

    Ok((font_atlas_image.atlas, handle_to_id))
}

pub fn asset_root() -> anyhow::Result<PathBuf> {
    if let Some(path) = std::env::var_os("GAME_ASSET_DIR") {
        return asset_root_from_override(PathBuf::from(path));
    }

    let candidates = asset_root_candidates()?;

    for candidate in &candidates {
        if candidate.path.is_dir() {
            log::info!("using asset root: {}", candidate.path.display());
            if candidate.is_debug_fallback {
                log::info!("using debug asset fallback from CARGO_MANIFEST_DIR");
            }
            return Ok(candidate.path.clone());
        }
    }

    let mut message = String::from("failed to locate runtime assets\ntried:");
    for candidate in candidates {
        message.push_str(&format!("\n- {}", candidate.description));
    }
    anyhow::bail!(message)
}

fn asset_root_from_override(path: PathBuf) -> anyhow::Result<PathBuf> {
    if path.is_dir() {
        log::info!("using asset root from GAME_ASSET_DIR: {}", path.display());
        return Ok(path);
    }

    anyhow::bail!(
        "GAME_ASSET_DIR={} is set but is not a directory",
        path.display()
    )
}

struct AssetRootCandidate {
    path: PathBuf,
    description: String,
    is_debug_fallback: bool,
}

fn asset_root_candidates() -> anyhow::Result<Vec<AssetRootCandidate>> {
    let mut candidates = Vec::new();

    let exe = std::env::current_exe().context("failed to resolve current executable path")?;
    if let Some(exe_dir) = exe.parent() {
        let path = exe_dir.join("assets");
        candidates.push(AssetRootCandidate {
            description: path.display().to_string(),
            path,
            is_debug_fallback: false,
        });
    }

    if cfg!(debug_assertions) {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
        candidates.push(AssetRootCandidate {
            description: format!("debug source fallback {}", path.display()),
            path,
            is_debug_fallback: true,
        });

        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
        candidates.push(AssetRootCandidate {
            description: format!("debug workspace fallback {}", path.display()),
            path,
            is_debug_fallback: true,
        });
    }

    Ok(candidates)
}

/// Validates renderer-owned assets that are required before the Vulkan context
/// can load built-in textures. Content assets are validated separately by the
/// runtime through `game_core::assets::AssetValidator`.
pub fn validate_builtin_assets(root: &Path) -> anyhow::Result<()> {
    let font = root.join("fonts/DejaVuSans.ttf");
    if !font.is_file() {
        anyhow::bail!(
            "renderer built-in font asset '{}' does not exist",
            font.display()
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{asset_root_from_override, validate_builtin_assets};
    use std::fs;
    use std::path::Path;

    #[test]
    fn asset_override_requires_existing_directory() {
        let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
        assert_eq!(asset_root_from_override(assets.clone()).unwrap(), assets);

        let missing = Path::new(env!("CARGO_MANIFEST_DIR")).join("does-not-exist");
        assert!(asset_root_from_override(missing).is_err());
    }

    #[test]
    fn validates_builtin_font_asset() {
        let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
        validate_builtin_assets(&assets).unwrap();
    }

    #[test]
    fn missing_builtin_font_reports_path() {
        let root =
            std::env::temp_dir().join(format!("game-renderer-missing-font-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("fonts")).unwrap();

        let err = validate_builtin_assets(&root).unwrap_err();
        assert!(err.to_string().contains("fonts/DejaVuSans.ttf"));

        fs::remove_dir_all(root).unwrap();
    }
}

//! Loads the renderer's built-in GPU textures and font atlas. Split out of
//! `context.rs` so asset acquisition is isolated from device orchestration.
//!
//! Textures are registered into the caller's [`TextureRegistryGuard`] the moment
//! they are created, so a failure partway through can never leak an already-built
//! texture: the guard owns everything registered so far and destroys it on drop.

use std::path::{Path, PathBuf};

use anyhow::Context;
use ash::vk;

use crate::renderer::texture::{Texture, TextureColorSpace, TextureUpload};
use crate::renderer::texture_registry::TextureRegistryGuard;
use crate::renderer::{FONT_TEXTURE_ID, TEST_TEXTURE_ID, text};

/// Creates and registers the renderer's built-in textures into `registry_guard`,
/// returning the font atlas metadata the renderer needs for text layout.
///
/// Each `create_and_register` call hands its freshly-created texture straight to
/// the registry, so the only state held across the fallible font-atlas build is
/// the guard itself — which already owns the test texture and will clean it up if
/// any later step (here or back in `VulkanContext::new`) fails.
pub fn load_builtin_textures(
    registry_guard: &mut TextureRegistryGuard<'_>,
    descriptor_set_layout: vk::DescriptorSetLayout,
    queue: vk::Queue,
    upload_pool: vk::CommandPool,
    upload_fence: vk::Fence,
) -> anyhow::Result<text::FontAtlas> {
    let asset_root = asset_root()?;

    let test_id = registry_guard.create_and_register(
        descriptor_set_layout,
        "test texture",
        |device, allocator| {
            let mut upload = TextureUpload {
                device,
                allocator,
                queue,
                upload_pool,
                upload_fence,
            };
            Texture::from_path(
                &mut upload,
                asset_root.join("textures/test.png"),
                TextureColorSpace::SrgbColor,
                "test texture",
            )
        },
    )?;
    assert_eq!(
        test_id, TEST_TEXTURE_ID,
        "built-in test texture must keep its stable TextureId"
    );

    let font_atlas_image =
        text::build_ascii_atlas(asset_root.join("fonts/DejaVuSans.ttf"), FONT_TEXTURE_ID)?;
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
        "built-in font texture must keep its stable TextureId"
    );

    Ok(font_atlas_image.atlas)
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

#[cfg(test)]
mod tests {
    use super::asset_root_from_override;
    use std::path::Path;

    #[test]
    fn asset_override_requires_existing_directory() {
        let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
        assert_eq!(asset_root_from_override(assets.clone()).unwrap(), assets);

        let missing = Path::new(env!("CARGO_MANIFEST_DIR")).join("does-not-exist");
        assert!(asset_root_from_override(missing).is_err());
    }
}

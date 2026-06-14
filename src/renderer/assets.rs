//! Loads the renderer's built-in GPU textures and font atlas. Split out of
//! `context.rs` so asset acquisition is isolated from device orchestration.
//!
//! Textures are registered into the caller's [`TextureRegistryGuard`] the moment
//! they are created, so a failure partway through can never leak an already-built
//! texture: the guard owns everything registered so far and destroys it on drop.

use std::path::PathBuf;

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
                asset_path("assets/textures/test.png"),
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
        text::build_ascii_atlas(asset_path("assets/fonts/DejaVuSans.ttf"), FONT_TEXTURE_ID)?;
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

/// Resolves an asset path, preferring assets shipped next to the executable (the
/// installed/distributed layout) and falling back to the crate manifest dir so
/// `cargo run` works from a source checkout where assets live in the repo root.
pub fn asset_path(relative: &str) -> PathBuf {
    if let Ok(exe) = std::env::current_exe()
        && let Some(exe_dir) = exe.parent()
    {
        let candidate = exe_dir.join(relative);
        if candidate.exists() {
            return candidate;
        }
    }

    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)
}

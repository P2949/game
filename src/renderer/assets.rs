//! Loads the renderer's GPU textures and font atlas from disk. Split out of
//! `context.rs` so asset acquisition is isolated from device orchestration.

use std::path::PathBuf;

use crate::renderer::texture::{Texture, TextureColorSpace, TextureUpload};
use crate::renderer::{FONT_TEXTURE_ID, text};

/// The renderer's loaded GPU assets.
pub struct RendererAssets {
    pub test_texture: Texture,
    pub font_texture: Texture,
    pub font_atlas: text::FontAtlas,
}

impl RendererAssets {
    /// Uploads the bundled test texture and builds the ASCII font atlas texture,
    /// using `upload` for the staging transfers.
    pub fn load(upload: &mut TextureUpload<'_>) -> anyhow::Result<Self> {
        let test_texture = Texture::from_path(
            upload,
            asset_path("assets/textures/test.png"),
            TextureColorSpace::SrgbColor,
            "test texture",
        )?;

        let font_atlas_image =
            text::build_ascii_atlas(asset_path("assets/fonts/DejaVuSans.ttf"), FONT_TEXTURE_ID)?;
        let font_texture = Texture::from_rgba8(
            upload,
            font_atlas_image.width,
            font_atlas_image.height,
            &font_atlas_image.pixels,
            TextureColorSpace::LinearData,
            "font atlas",
        )?;

        Ok(Self {
            test_texture,
            font_texture,
            font_atlas: font_atlas_image.atlas,
        })
    }
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

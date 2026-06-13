use std::collections::HashMap;
use std::path::Path;

use crate::renderer::{SpriteDraw, TextureId};

const FIRST_ASCII: u8 = 32;
const LAST_ASCII: u8 = 126;
const ATLAS_COLUMNS: u32 = 16;
const ATLAS_ROWS: u32 = 6;
const CELL_SIZE: u32 = 48;
const CELL_PADDING: u32 = 4;
const FONT_SIZE: f32 = 32.0;

pub struct GlyphInfo {
    pub uv_min: glam::Vec2,
    pub uv_max: glam::Vec2,
    pub size: glam::Vec2,
    pub bearing: glam::Vec2,
    pub advance: f32,
}

pub struct FontAtlas {
    pub texture: TextureId,
    pub glyphs: HashMap<char, GlyphInfo>,
    pub line_height: f32,
}

pub struct FontAtlasImage {
    pub atlas: FontAtlas,
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

pub fn build_ascii_atlas(
    path: impl AsRef<Path>,
    texture: TextureId,
) -> anyhow::Result<FontAtlasImage> {
    let font_bytes = std::fs::read(path)?;
    let font = fontdue::Font::from_bytes(font_bytes, fontdue::FontSettings::default())
        .map_err(|err| anyhow::anyhow!("failed to load font: {err}"))?;

    let width = ATLAS_COLUMNS * CELL_SIZE;
    let height = ATLAS_ROWS * CELL_SIZE;
    let mut pixels = vec![0_u8; width as usize * height as usize * 4];
    let mut glyphs = HashMap::new();

    let line_metrics = font
        .horizontal_line_metrics(FONT_SIZE)
        .ok_or_else(|| anyhow::anyhow!("font does not provide horizontal line metrics"))?;

    for ch in FIRST_ASCII..=LAST_ASCII {
        let index = (ch - FIRST_ASCII) as u32;
        let col = index % ATLAS_COLUMNS;
        let row = index / ATLAS_COLUMNS;

        let (metrics, bitmap) = font.rasterize(ch as char, FONT_SIZE);
        let dst_x = col * CELL_SIZE + CELL_PADDING;
        let dst_y = row * CELL_SIZE + CELL_PADDING;

        if metrics.width as u32 + CELL_PADDING * 2 > CELL_SIZE
            || metrics.height as u32 + CELL_PADDING * 2 > CELL_SIZE
        {
            anyhow::bail!(
                "glyph '{}' does not fit in {CELL_SIZE}px atlas cell",
                ch as char
            );
        }

        for y in 0..metrics.height {
            for x in 0..metrics.width {
                let alpha = bitmap[y * metrics.width + x];
                let atlas_x = dst_x as usize + x;
                let atlas_y = dst_y as usize + y;
                let offset = (atlas_y * width as usize + atlas_x) * 4;
                pixels[offset] = 255;
                pixels[offset + 1] = 255;
                pixels[offset + 2] = 255;
                pixels[offset + 3] = alpha;
            }
        }

        let uv_min = glam::vec2(dst_x as f32 / width as f32, dst_y as f32 / height as f32);
        let uv_max = glam::vec2(
            (dst_x + metrics.width as u32) as f32 / width as f32,
            (dst_y + metrics.height as u32) as f32 / height as f32,
        );
        let size = glam::vec2(metrics.width as f32, metrics.height as f32);
        let bearing = glam::vec2(
            metrics.xmin as f32,
            line_metrics.ascent - metrics.ymin as f32 - metrics.height as f32,
        );

        glyphs.insert(
            ch as char,
            GlyphInfo {
                uv_min,
                uv_max,
                size,
                bearing,
                advance: metrics.advance_width,
            },
        );
    }

    Ok(FontAtlasImage {
        atlas: FontAtlas {
            texture,
            glyphs,
            line_height: line_metrics.new_line_size,
        },
        pixels,
        width,
        height,
    })
}

pub fn draw_text(
    batch: &mut crate::renderer::sprite_batch::SpriteBatch,
    atlas: &FontAtlas,
    text: &str,
    mut pos: glam::Vec2,
    color: glam::Vec4,
    layer: i16,
) {
    let start_x = pos.x;

    for ch in text.chars() {
        if ch == '\n' {
            pos.x = start_x;
            pos.y += atlas.line_height;
            continue;
        }

        let Some(glyph) = atlas.glyphs.get(&ch) else {
            continue;
        };

        if glyph.size.x > 0.0 && glyph.size.y > 0.0 {
            let glyph_pos = glam::vec2(pos.x + glyph.bearing.x, pos.y + glyph.bearing.y);

            batch.push(SpriteDraw {
                texture: atlas.texture,
                layer,
                position: glyph_pos,
                size: glyph.size,
                uv_min: glyph.uv_min,
                uv_max: glyph.uv_max,
                color,
            });
        }

        pos.x += glyph.advance;
    }
}

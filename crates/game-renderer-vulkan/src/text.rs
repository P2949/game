use std::collections::HashMap;
use std::path::Path;

use anyhow::Context;
use game_core::backend::TextureHandle;

use crate::renderer::SpriteDraw;

// The atlas is a fixed grid of equally-sized cells covering printable ASCII.
// These values are tuned for the bundled DejaVu Sans at FONT_SIZE: every glyph in
// `FIRST_ASCII..=LAST_ASCII` must fit within CELL_SIZE (minus padding), which
// `build_ascii_atlas` asserts. Changing the font or font size may require
// retuning CELL_SIZE / the grid; a future dynamic (measured + packed) atlas would
// remove this fixed-cell assumption. Non-ASCII characters are not in the atlas
// and render via a fallback glyph at draw time (see `glyph_or_fallback`).
const FIRST_ASCII: u8 = 32;
const LAST_ASCII: u8 = 126;
const ATLAS_COLUMNS: u32 = 16;
const ATLAS_ROWS: u32 = 6;
const CELL_SIZE: u32 = 48;
const CELL_PADDING: u32 = 4;
const FONT_SIZE: f32 = 32.0;

/// Glyph substituted for any character not present in the atlas, so unsupported
/// text degrades to visible placeholders instead of silently collapsing.
const FALLBACK_CHAR: char = '?';

pub struct GlyphInfo {
    pub uv_min: glam::Vec2,
    pub uv_max: glam::Vec2,
    pub size: glam::Vec2,
    pub bearing: glam::Vec2,
    pub advance: f32,
}

pub struct FontAtlas {
    pub texture: TextureHandle,
    pub glyphs: HashMap<char, GlyphInfo>,
    pub line_height: f32,
}

impl FontAtlas {
    /// Returns the glyph for `ch`, falling back to [`FALLBACK_CHAR`] for any
    /// character the atlas does not contain. `None` only if even the fallback is
    /// missing (an atlas built without `?`).
    fn glyph_or_fallback(&self, ch: char) -> Option<&GlyphInfo> {
        self.glyphs
            .get(&ch)
            .or_else(|| self.glyphs.get(&FALLBACK_CHAR))
    }

    /// Horizontal advance used when no glyph (not even the fallback) is available,
    /// so layout still progresses. Prefers the fallback glyph's advance, then the
    /// space glyph's, then zero.
    fn missing_glyph_advance(&self) -> f32 {
        self.glyphs
            .get(&FALLBACK_CHAR)
            .or_else(|| self.glyphs.get(&' '))
            .map(|glyph| glyph.advance)
            .unwrap_or(0.0)
    }

    /// Per-character horizontal advance, accounting for the fallback glyph.
    fn advance_for(&self, ch: char) -> f32 {
        self.glyph_or_fallback(ch)
            .map(|glyph| glyph.advance)
            .unwrap_or_else(|| self.missing_glyph_advance())
    }

    /// Measures the pixel size of `text` as it would be laid out by
    /// [`draw_text`], honoring embedded newlines and the fallback glyph. Returns
    /// `(max line width, total height)`. Provided for UI layout (alignment,
    /// centering); not yet wired into gameplay rendering.
    #[allow(dead_code)]
    pub fn measure_text(&self, text: &str) -> glam::Vec2 {
        let mut max_width = 0.0_f32;
        let mut line_width = 0.0_f32;
        let mut lines = 1_u32;

        for ch in text.chars() {
            if ch == '\n' {
                max_width = max_width.max(line_width);
                line_width = 0.0;
                lines += 1;
                continue;
            }
            line_width += self.advance_for(ch);
        }
        max_width = max_width.max(line_width);

        glam::vec2(max_width, lines as f32 * self.line_height)
    }

    /// Greedily word-wraps `text` so each produced line fits within `max_width`
    /// pixels, preserving existing newlines. Whitespace is the only break point;
    /// a single word longer than `max_width` is left on its own (over-long) line.
    ///
    /// Note: word splitting uses `split_whitespace`, so runs of spaces, tabs, and
    /// other intra-line whitespace are collapsed to single spaces in the output.
    /// That suits HUD/menu text; whitespace-significant content (pre-formatted or
    /// code-like text) would need a preserving wrapper instead.
    ///
    /// Provided for UI layout; not yet wired into gameplay rendering.
    #[allow(dead_code)]
    pub fn wrap_text(&self, text: &str, max_width: f32) -> Vec<String> {
        let space_advance = self.advance_for(' ');
        let mut lines = Vec::new();

        for raw_line in text.split('\n') {
            let mut current = String::new();
            let mut current_width = 0.0_f32;

            for word in raw_line.split_whitespace() {
                let word_width: f32 = word.chars().map(|ch| self.advance_for(ch)).sum();

                if current.is_empty() {
                    current.push_str(word);
                    current_width = word_width;
                } else if current_width + space_advance + word_width <= max_width {
                    current.push(' ');
                    current.push_str(word);
                    current_width += space_advance + word_width;
                } else {
                    lines.push(std::mem::take(&mut current));
                    current.push_str(word);
                    current_width = word_width;
                }
            }

            lines.push(current);
        }

        lines
    }
}

pub struct FontAtlasImage {
    pub atlas: FontAtlas,
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct TextDrawStats {
    pub glyphs_submitted: usize,
    pub glyphs_dropped: usize,
}

pub fn build_ascii_atlas(
    path: impl AsRef<Path>,
    texture: TextureHandle,
) -> anyhow::Result<FontAtlasImage> {
    let path = path.as_ref();
    let font_bytes =
        std::fs::read(path).with_context(|| format!("failed to read font {}", path.display()))?;
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
) -> TextDrawStats {
    let mut stats = TextDrawStats::default();
    let start_x = pos.x;

    for ch in text.chars() {
        if ch == '\n' {
            pos.x = start_x;
            pos.y += atlas.line_height;
            continue;
        }

        let Some(glyph) = atlas.glyph_or_fallback(ch) else {
            // No glyph and no fallback available: advance so layout still
            // progresses instead of stacking subsequent glyphs on top.
            pos.x += atlas.missing_glyph_advance();
            continue;
        };

        if glyph.size.x > 0.0 && glyph.size.y > 0.0 {
            let glyph_pos = glam::vec2(pos.x + glyph.bearing.x, pos.y + glyph.bearing.y);

            let sprite = SpriteDraw {
                texture: atlas.texture,
                layer,
                position: glyph_pos,
                size: glyph.size,
                uv_min: glyph.uv_min,
                uv_max: glyph.uv_max,
                color,
            };

            if batch.push(sprite) {
                stats.glyphs_submitted += 1;
            } else {
                stats.glyphs_dropped += 1;
            }
        }

        pos.x += glyph.advance;
    }

    stats
}

#[cfg(test)]
mod tests {
    use super::{FontAtlas, GlyphInfo, draw_text};
    use crate::renderer::FONT_TEXTURE_HANDLE;
    use crate::renderer::sprite_batch::SpriteBatch;
    use std::collections::HashMap;

    fn glyph(advance: f32) -> GlyphInfo {
        GlyphInfo {
            uv_min: glam::Vec2::ZERO,
            uv_max: glam::Vec2::ONE,
            size: glam::vec2(advance, 10.0),
            bearing: glam::Vec2::ZERO,
            advance,
        }
    }

    fn test_atlas() -> FontAtlas {
        let mut glyphs = HashMap::new();
        glyphs.insert('A', glyph(10.0));
        glyphs.insert('B', glyph(10.0));
        glyphs.insert(' ', glyph(5.0));
        glyphs.insert('?', glyph(8.0));
        FontAtlas {
            texture: FONT_TEXTURE_HANDLE,
            glyphs,
            line_height: 12.0,
        }
    }

    #[test]
    fn measure_text_sums_advances_and_counts_lines() {
        let atlas = test_atlas();
        assert_eq!(atlas.measure_text("AB"), glam::vec2(20.0, 12.0));
        assert_eq!(atlas.measure_text("A\nB"), glam::vec2(10.0, 24.0));
    }

    #[test]
    fn measure_empty_text_is_one_line_high_and_zero_wide() {
        assert_eq!(test_atlas().measure_text(""), glam::vec2(0.0, 12.0));
    }

    #[test]
    fn unsupported_character_uses_fallback_advance() {
        // '€' is not in the atlas, so it measures as the fallback '?' (advance 8).
        assert_eq!(test_atlas().measure_text("€"), glam::vec2(8.0, 12.0));
    }

    #[test]
    fn wrap_text_breaks_on_width_and_preserves_newlines() {
        let atlas = test_atlas();
        // Each 'A' is 10 wide, space is 5: "A A" = 25 fits, a third word overflows.
        assert_eq!(atlas.wrap_text("A A A", 25.0), vec!["A A", "A"]);
        // Existing newlines are preserved as hard breaks.
        assert_eq!(atlas.wrap_text("A\nB", 100.0), vec!["A", "B"]);
    }

    #[test]
    fn draw_text_reports_submitted_glyphs() {
        let atlas = test_atlas();
        let mut batch = SpriteBatch::new();

        let stats = draw_text(
            &mut batch,
            &atlas,
            "AB",
            glam::Vec2::ZERO,
            glam::Vec4::ONE,
            0,
        );

        assert_eq!(stats.glyphs_submitted, 2);
        assert_eq!(stats.glyphs_dropped, 0);
    }

    #[test]
    fn draw_text_reports_dropped_invalid_glyph_sprites() {
        let mut atlas = test_atlas();
        atlas.glyphs.insert(
            'A',
            GlyphInfo {
                bearing: glam::vec2(f32::NAN, 0.0),
                ..glyph(10.0)
            },
        );
        let mut batch = SpriteBatch::new();

        let stats = draw_text(
            &mut batch,
            &atlas,
            "A",
            glam::Vec2::ZERO,
            glam::Vec4::ONE,
            0,
        );

        assert_eq!(stats.glyphs_submitted, 0);
        assert_eq!(stats.glyphs_dropped, 1);
    }
}

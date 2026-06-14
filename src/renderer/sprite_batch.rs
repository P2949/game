use crate::renderer::vertex::SpriteVertex;
use crate::renderer::{SpriteDraw, TextureId};

#[derive(Clone, Copy, Debug)]
pub struct SpriteBatchRange {
    pub texture: TextureId,
    pub first_vertex: u32,
    pub vertex_count: u32,
}

pub struct SpriteBatch {
    sprites: Vec<SpriteDraw>,
}

impl SpriteBatch {
    pub fn new() -> Self {
        Self {
            sprites: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.sprites.clear();
    }

    pub fn push(&mut self, sprite: SpriteDraw) {
        self.sprites.push(sprite);
    }

    /// Sorts the queued sprites in place, then appends their triangle vertices
    /// to `vertices` while recording per-texture draw ranges in `ranges`.
    ///
    /// Both output buffers are appended to (never cleared), and recorded
    /// `first_vertex` values are absolute indices into `vertices`. This lets a
    /// caller pack several batches into one shared, reused vertex buffer without
    /// any per-frame allocation once the buffers reach steady-state capacity.
    pub fn build_into(
        &mut self,
        vertices: &mut Vec<SpriteVertex>,
        ranges: &mut Vec<SpriteBatchRange>,
    ) {
        // Ordering contract:
        //   * Sprites are sorted by `layer` first, then by `texture` (to group
        //     same-texture draws and reduce descriptor binds within a layer).
        //   * Layer order therefore always wins over texture grouping: a sprite
        //     on a higher layer draws after (on top of) lower layers regardless
        //     of texture.
        //   * Within a single layer, cross-texture draw order is NOT preserved —
        //     it follows texture id, not submission order. So overlapping
        //     transparent sprites that need a strict front-to-back order must be
        //     placed on distinct layers, not left to submission order.
        // The sort is stable, so same-(layer, texture) sprites keep submission
        // order relative to each other.
        self.sprites
            .sort_by_key(|sprite| (sprite.layer, sprite.texture.0));

        let mut current_texture: Option<TextureId> = None;
        let mut current_start = vertices.len() as u32;

        for &sprite in &self.sprites {
            if current_texture != Some(sprite.texture) {
                if let Some(texture) = current_texture {
                    let vertex_count = vertices.len() as u32 - current_start;
                    if vertex_count > 0 {
                        ranges.push(SpriteBatchRange {
                            texture,
                            first_vertex: current_start,
                            vertex_count,
                        });
                    }
                }

                current_texture = Some(sprite.texture);
                current_start = vertices.len() as u32;
            }

            append_sprite_vertices(vertices, sprite);
        }

        if let Some(texture) = current_texture {
            let vertex_count = vertices.len() as u32 - current_start;
            if vertex_count > 0 {
                ranges.push(SpriteBatchRange {
                    texture,
                    first_vertex: current_start,
                    vertex_count,
                });
            }
        }
    }
}

impl Default for SpriteBatch {
    fn default() -> Self {
        Self::new()
    }
}

pub fn append_sprite_vertices(out: &mut Vec<SpriteVertex>, sprite: SpriteDraw) {
    let x = sprite.position.x;
    let y = sprite.position.y;
    let w = sprite.size.x;
    let h = sprite.size.y;

    let u0 = sprite.uv_min.x;
    let v0 = sprite.uv_min.y;
    let u1 = sprite.uv_max.x;
    let v1 = sprite.uv_max.y;

    let c = sprite.color.to_array();

    let p0 = [x, y];
    let p1 = [x + w, y];
    let p2 = [x + w, y + h];
    let p3 = [x, y + h];

    out.extend_from_slice(&[
        SpriteVertex {
            pos: p0,
            uv: [u0, v0],
            color: c,
        },
        SpriteVertex {
            pos: p1,
            uv: [u1, v0],
            color: c,
        },
        SpriteVertex {
            pos: p2,
            uv: [u1, v1],
            color: c,
        },
        SpriteVertex {
            pos: p0,
            uv: [u0, v0],
            color: c,
        },
        SpriteVertex {
            pos: p2,
            uv: [u1, v1],
            color: c,
        },
        SpriteVertex {
            pos: p3,
            uv: [u0, v1],
            color: c,
        },
    ]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::{FONT_TEXTURE_ID, TEST_TEXTURE_ID};

    fn sprite(texture: TextureId, layer: i16) -> SpriteDraw {
        SpriteDraw {
            texture,
            layer,
            position: glam::Vec2::ZERO,
            size: glam::Vec2::ONE,
            uv_min: glam::Vec2::ZERO,
            uv_max: glam::Vec2::ONE,
            color: glam::Vec4::ONE,
        }
    }

    #[test]
    fn build_into_merges_runs_and_sorts_by_layer() {
        let mut batch = SpriteBatch::new();
        // Pushed out of order and interleaved by texture; sorting by (layer,
        // texture) must group the two TEST sprites on layer 0 into one run.
        batch.push(sprite(TEST_TEXTURE_ID, 0));
        batch.push(sprite(FONT_TEXTURE_ID, 5));
        batch.push(sprite(TEST_TEXTURE_ID, 0));

        let mut vertices = Vec::new();
        let mut ranges = Vec::new();
        batch.build_into(&mut vertices, &mut ranges);

        assert_eq!(vertices.len(), 3 * 6);
        assert_eq!(ranges.len(), 2);
        assert_eq!(ranges[0].texture, TEST_TEXTURE_ID);
        assert_eq!(ranges[0].first_vertex, 0);
        assert_eq!(ranges[0].vertex_count, 12);
        assert_eq!(ranges[1].texture, FONT_TEXTURE_ID);
        assert_eq!(ranges[1].first_vertex, 12);
        assert_eq!(ranges[1].vertex_count, 6);
    }

    #[test]
    fn build_into_appends_with_absolute_offsets_across_batches() {
        let mut world = SpriteBatch::new();
        world.push(sprite(TEST_TEXTURE_ID, 0));
        let mut ui = SpriteBatch::new();
        ui.push(sprite(FONT_TEXTURE_ID, 0));

        let mut vertices = Vec::new();
        let mut ranges = Vec::new();
        world.build_into(&mut vertices, &mut ranges);
        // Second batch packs into the same buffers; its range offset must be
        // absolute (start at 6), not reset to 0.
        ui.build_into(&mut vertices, &mut ranges);

        assert_eq!(vertices.len(), 12);
        assert_eq!(ranges.len(), 2);
        assert_eq!(ranges[1].first_vertex, 6);
        assert_eq!(ranges[1].vertex_count, 6);
    }

    #[test]
    fn build_into_layer_order_wins_over_texture_id() {
        // FONT (texture id 1) on layer 0 must draw before TEST (texture id 0) on
        // layer 5: layer ordering dominates the texture secondary key.
        let mut batch = SpriteBatch::new();
        batch.push(sprite(TEST_TEXTURE_ID, 5));
        batch.push(sprite(FONT_TEXTURE_ID, 0));

        let mut vertices = Vec::new();
        let mut ranges = Vec::new();
        batch.build_into(&mut vertices, &mut ranges);

        assert_eq!(ranges.len(), 2);
        assert_eq!(ranges[0].texture, FONT_TEXTURE_ID);
        assert_eq!(ranges[1].texture, TEST_TEXTURE_ID);
    }

    #[test]
    fn build_into_on_empty_batch_records_nothing() {
        let mut batch = SpriteBatch::new();
        let mut vertices = Vec::new();
        let mut ranges = Vec::new();
        batch.build_into(&mut vertices, &mut ranges);

        assert!(vertices.is_empty());
        assert!(ranges.is_empty());
    }
}

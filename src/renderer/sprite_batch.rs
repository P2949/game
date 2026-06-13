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

    pub fn sorted_sprites(&self) -> Vec<SpriteDraw> {
        let mut sprites = self.sprites.clone();
        sprites.sort_by_key(|sprite| sprite.texture.0);
        sprites
    }

    pub fn build_vertices(&self) -> (Vec<SpriteVertex>, Vec<SpriteBatchRange>) {
        let sorted = self.sorted_sprites();
        let mut vertices = Vec::with_capacity(sorted.len() * 6);
        let mut ranges = Vec::new();
        let mut current_texture = None;
        let mut current_start = 0_u32;

        for sprite in sorted {
            if current_texture != Some(sprite.texture) {
                if let Some(texture) = current_texture {
                    ranges.push(SpriteBatchRange {
                        texture,
                        first_vertex: current_start,
                        vertex_count: vertices.len() as u32 - current_start,
                    });
                }

                current_texture = Some(sprite.texture);
                current_start = vertices.len() as u32;
            }

            append_sprite_vertices(&mut vertices, sprite);
        }

        if let Some(texture) = current_texture {
            ranges.push(SpriteBatchRange {
                texture,
                first_vertex: current_start,
                vertex_count: vertices.len() as u32 - current_start,
            });
        }

        (vertices, ranges)
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

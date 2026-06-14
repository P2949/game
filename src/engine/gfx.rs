use glam::{Vec2, Vec4};

use crate::renderer::{DrawCommands, SpriteDraw, TextureId};

#[derive(Clone, Copy, Debug)]
pub struct SpriteHandle(pub TextureId);

pub struct Gfx<'a> {
    draw: &'a mut dyn DrawCommands,
}

impl<'a> Gfx<'a> {
    pub fn new(draw: &'a mut dyn DrawCommands) -> Self {
        Self { draw }
    }

    #[allow(dead_code)]
    pub fn sprite(
        &mut self,
        handle: SpriteHandle,
        center: Vec2,
        size: Vec2,
        layer: i16,
        color: Vec4,
    ) {
        self.draw.draw_world_sprite(SpriteDraw {
            texture: handle.0,
            layer,
            position: center - size * 0.5,
            size,
            uv_min: Vec2::ZERO,
            uv_max: Vec2::ONE,
            color,
        });
    }

    pub fn text(&mut self, text: &str, pos: Vec2, color: Vec4) {
        self.draw.draw_ui_text(text, pos, color);
    }
}

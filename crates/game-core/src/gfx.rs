use glam::{Vec2, Vec4};

use crate::app::RenderFrame;
use crate::backend::TextureHandle;

#[derive(Clone, Debug)]
pub struct TextDraw {
    pub text: String,
    pub pos: Vec2,
    pub color: Vec4,
    pub layer: i16,
}

#[derive(Clone, Copy, Debug)]
pub struct SpriteDraw {
    pub texture: TextureHandle,
    pub layer: i16,
    pub position: Vec2,
    pub size: Vec2,
    pub uv_min: Vec2,
    pub uv_max: Vec2,
    pub color: Vec4,
}

pub struct Gfx<'a> {
    frame: &'a mut RenderFrame,
}

impl<'a> Gfx<'a> {
    pub fn new(frame: &'a mut RenderFrame) -> Self {
        Self { frame }
    }

    #[allow(dead_code)]
    pub fn sprite(
        &mut self,
        texture: TextureHandle,
        center: Vec2,
        size: Vec2,
        layer: i16,
        color: Vec4,
    ) {
        self.frame.world_sprites.push(SpriteDraw {
            texture,
            layer,
            position: center - size * 0.5,
            size,
            uv_min: Vec2::ZERO,
            uv_max: Vec2::ONE,
            color,
        });
    }

    pub fn text(&mut self, text: &str, pos: Vec2, color: Vec4) {
        self.frame.ui_text.push(TextDraw {
            text: text.to_owned(),
            pos,
            color,
            layer: 10_000,
        });
    }
}

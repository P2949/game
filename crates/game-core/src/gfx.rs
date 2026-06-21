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

/// A renderer-neutral, screen-space coloured rectangle used by high-level UI.
/// `position` is its top-left corner; `layer` orders it with other UI draws.
#[derive(Clone, Copy, Debug)]
pub struct UiRect {
    pub position: Vec2,
    pub size: Vec2,
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

    pub fn ui_rect(&mut self, position: Vec2, size: Vec2, color: Vec4, layer: i16) {
        if size.x <= 0.0 || size.y <= 0.0 || !position.is_finite() || !size.is_finite() {
            return;
        }
        self.frame.ui_rects.push(UiRect {
            position,
            size,
            color,
            layer,
        });
    }
}

#[cfg(test)]
mod tests {
    use glam::{Vec2, vec4};

    use super::Gfx;
    use crate::app::RenderFrame;
    use crate::camera::Camera2D;

    #[test]
    fn ui_rects_are_extracted_and_invalid_rects_are_ignored() {
        let mut frame = RenderFrame::new(Camera2D::default());
        Gfx::new(&mut frame).ui_rect(
            Vec2::new(12.0, 24.0),
            Vec2::new(50.0, 30.0),
            vec4(1.0, 0.0, 0.0, 1.0),
            42,
        );
        Gfx::new(&mut frame).ui_rect(Vec2::ZERO, Vec2::ZERO, vec4(1.0, 1.0, 1.0, 1.0), 42);

        assert_eq!(frame.ui_rects.len(), 1);
        assert_eq!(frame.ui_rects[0].position, Vec2::new(12.0, 24.0));
        assert_eq!(frame.ui_rects[0].size, Vec2::new(50.0, 30.0));
        assert_eq!(frame.ui_rects[0].layer, 42);
    }
}

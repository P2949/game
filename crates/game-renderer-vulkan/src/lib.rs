pub mod assets;
pub mod buffer;
pub mod commands;
pub mod context;
pub mod debug;
pub mod device;
pub mod frame;
pub mod instance;
pub mod owned;
pub mod pipeline;
pub mod recreate;
pub mod sprite_batch;
pub mod surface;
pub mod swapchain;
pub mod text;
pub mod texture;
pub mod texture_registry;
pub mod vertex;

/// Handle to a texture registered in the renderer's `TextureRegistry`.
///
/// Ids are assigned sequentially as textures are registered and resolved back to
/// a descriptor set at draw time; an unknown id is reported as an error rather
/// than panicking, so a stale id cannot cause unsafe access. The inner value is
/// `pub` only for the built-in constants below; richer safety (opaque,
/// registry-minted ids with generation counters) is deferred until the engine
/// grows dynamic texture lifetimes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TextureId(pub u32);

// Built-in texture handles. These are the first two registrations in the
// renderer's `TextureRegistry`, so the ids are stable and can be referenced as
// constants from gameplay code. Registration order in `assets::load_builtin_textures`
// must match these values (asserted there with `assert_eq!`).
pub const TEST_TEXTURE_ID: TextureId = TextureId(0);
pub const FONT_TEXTURE_ID: TextureId = TextureId(1);

#[derive(Clone, Copy, Debug)]
pub struct RenderCamera {
    center: glam::Vec2,
    zoom: f32,
}

impl RenderCamera {
    pub fn new(center: glam::Vec2, zoom: f32) -> Self {
        Self {
            center: if center.is_finite() {
                center
            } else {
                glam::Vec2::ZERO
            },
            zoom: sanitize_zoom(zoom),
        }
    }

    pub fn view_projection(&self, width: f32, height: f32) -> glam::Mat4 {
        let width = sanitize_dimension(width);
        let height = sanitize_dimension(height);
        let half_w = width * 0.5 / self.zoom;
        let half_h = height * 0.5 / self.zoom;

        glam::Mat4::orthographic_rh(
            self.center.x - half_w,
            self.center.x + half_w,
            self.center.y - half_h,
            self.center.y + half_h,
            -1.0,
            1.0,
        )
    }
}

fn sanitize_zoom(zoom: f32) -> f32 {
    if zoom.is_finite() && zoom > 0.0 {
        zoom.clamp(0.01, 100_000.0)
    } else {
        1.0
    }
}

fn sanitize_dimension(value: f32) -> f32 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        1.0
    }
}

pub use game_core::gfx::SpriteDraw;

#[allow(dead_code)]
pub trait DrawCommands {
    fn draw_world_sprite(&mut self, sprite: SpriteDraw);
    fn draw_ui_sprite(&mut self, sprite: SpriteDraw);
    fn draw_ui_text(&mut self, text: &str, pos: glam::Vec2, color: glam::Vec4);
}

// TEMP: compatibility module for Phase 2's mechanical move from `src/renderer`.
// Later phases can replace these old single-crate paths with direct crate paths.
pub mod renderer {
    pub use crate::{
        DrawCommands, FONT_TEXTURE_ID, RenderCamera, TEST_TEXTURE_ID, TextureId, assets, buffer,
        commands, context, debug, device, frame, instance, owned, pipeline, recreate, sprite_batch,
        surface, swapchain, text, texture, texture_registry, vertex,
    };
    pub use game_core::gfx::SpriteDraw;
}

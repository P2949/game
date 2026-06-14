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
pub struct SpriteDraw {
    pub texture: TextureId,
    pub layer: i16,
    pub position: glam::Vec2,
    pub size: glam::Vec2,
    pub uv_min: glam::Vec2,
    pub uv_max: glam::Vec2,
    pub color: glam::Vec4,
}

pub trait DrawCommands {
    fn draw_world_sprite(&mut self, sprite: SpriteDraw);
    fn draw_ui_sprite(&mut self, sprite: SpriteDraw);
    fn draw_ui_text(&mut self, text: &str, pos: glam::Vec2, color: glam::Vec4);
}

pub mod buffer;
pub mod context;
pub mod debug;
pub mod device;
pub mod frame;
pub mod pipeline;
pub mod sprite_batch;
pub mod surface;
pub mod swapchain;
pub mod text;
pub mod texture;
pub mod vertex;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TextureId(pub u32);

pub const TEST_TEXTURE_ID: TextureId = TextureId(0);

#[derive(Clone, Copy, Debug)]
pub struct SpriteDraw {
    pub texture: TextureId,
    pub position: glam::Vec2,
    pub size: glam::Vec2,
    pub uv_min: glam::Vec2,
    pub uv_max: glam::Vec2,
    pub color: glam::Vec4,
}
// pub struct Renderer {
//     // private Vulkan internals
// }

// impl Renderer {
//     pub fn new(window: &sdl3::video::Window) -> anyhow::Result<Self> {
//         todo!()
//     }

//     pub fn resize(&mut self, width: u32, height: u32) {
//         todo!()
//     }

//     pub fn draw_sprite(&mut self, sprite: SpriteDraw) {
//         todo!()
//     }

//     pub fn render(&mut self) -> anyhow::Result<()> {
//         todo!()
//     }
// }

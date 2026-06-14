use crate::engine::gfx::SpriteHandle;
use crate::renderer::TEST_TEXTURE_ID;

pub struct Assets {
    pub floor: SpriteHandle,
    pub wall: SpriteHandle,
    pub player: SpriteHandle,
    pub enemy: SpriteHandle,
}

impl Assets {
    pub fn load() -> Self {
        let test_texture = SpriteHandle(TEST_TEXTURE_ID);
        Self {
            floor: test_texture,
            wall: test_texture,
            player: test_texture,
            enemy: test_texture,
        }
    }
}

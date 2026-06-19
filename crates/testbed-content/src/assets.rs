use game_kit::advanced::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct TestbedAssets {
    pub floor: TextureHandle,
    pub wall: TextureHandle,
    pub player: TextureHandle,
    pub chaser: TextureHandle,
    pub patroller: TextureHandle,
    pub hit: SoundHandle,
}

pub fn register(assets: &mut AssetAuthor<'_>) -> Result<TestbedAssets> {
    Ok(TestbedAssets {
        floor: assets.texture("testbed/floor", "textures/test.png")?,
        wall: assets.texture("testbed/wall", "textures/test.png")?,
        player: assets.texture("testbed/player", "textures/test.png")?,
        chaser: assets.texture("testbed/chaser", "textures/test.png")?,
        patroller: assets.texture("testbed/patroller", "textures/test.png")?,
        hit: assets.sound("testbed/hit", "sounds/hit.wav")?,
    })
}

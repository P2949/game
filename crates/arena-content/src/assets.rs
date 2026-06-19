use game_kit::beginner::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct ArenaAssets {
    pub floor: TextureHandle,
    pub wall: TextureHandle,
    pub player: TextureHandle,
    pub enemy: TextureHandle,
    pub hit: SoundHandle,
}

pub fn register(assets: &mut AssetAuthor<'_>) -> Result<ArenaAssets> {
    Ok(ArenaAssets {
        floor: assets.texture("arena/floor", "textures/test.png")?,
        wall: assets.texture("arena/wall", "textures/test.png")?,
        player: assets.texture("arena/player", "textures/test.png")?,
        enemy: assets.texture("arena/enemy", "textures/test.png")?,
        hit: assets.sound("arena/hit", "sounds/hit.wav")?,
    })
}

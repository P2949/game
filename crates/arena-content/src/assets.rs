use game_kit::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct ArenaAssets {
    pub floor: TextureHandle,
    pub wall: TextureHandle,
    pub player: TextureHandle,
    pub enemy: TextureHandle,
    pub hit: SoundHandle,
}

pub fn register(assets: &mut AssetAuthor<'_>) -> ArenaAssets {
    ArenaAssets {
        floor: assets.texture("arena/floor", "textures/test.png"),
        wall: assets.texture("arena/wall", "textures/test.png"),
        player: assets.texture("arena/player", "textures/test.png"),
        enemy: assets.texture("arena/enemy", "textures/test.png"),
        hit: assets.generated_sound("arena/hit"),
    }
}

use game_core::assets::AssetRegistry;
use game_core::backend::{SoundHandle, TextureHandle};

#[derive(Clone, Copy, Debug)]
pub struct ArenaAssets {
    pub floor: TextureHandle,
    pub wall: TextureHandle,
    pub player: TextureHandle,
    pub enemy: TextureHandle,
    pub hit: SoundHandle,
}

impl ArenaAssets {
    pub fn load() -> Self {
        let mut registry = AssetRegistry::new();
        register(&mut registry)
    }
}

pub fn register(assets: &mut AssetRegistry) -> ArenaAssets {
    ArenaAssets {
        floor: assets.texture("arena/floor", "textures/test.png"),
        wall: assets.texture("arena/wall", "textures/test.png"),
        player: assets.texture("arena/player", "textures/test.png"),
        enemy: assets.texture("arena/enemy", "textures/test.png"),
        hit: assets.sound("arena/hit", "audio/hit.wav"),
    }
}

#[cfg(test)]
mod tests {
    use super::register;
    use game_core::assets::AssetRegistry;

    #[test]
    fn arena_assets_register_current_texture_and_hit_sound() {
        let mut registry = AssetRegistry::new();
        let assets = register(&mut registry);

        assert_eq!(assets.floor.0, 0);
        assert_eq!(assets.wall, assets.floor);
        assert_eq!(assets.player, assets.floor);
        assert_eq!(assets.enemy, assets.floor);
        assert_eq!(assets.hit.0, 0);
        assert_eq!(
            registry.texture_request("arena/player").unwrap().path,
            "textures/test.png"
        );
        assert_eq!(
            registry.sound_request("arena/hit").unwrap().path,
            "audio/hit.wav"
        );
    }
}

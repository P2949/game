use game_core::assets::AssetRegistry;
use game_core::backend::{SoundHandle, TextureHandle};

#[derive(Clone, Copy, Debug)]
pub struct TestbedAssets {
    pub floor: TextureHandle,
    pub wall: TextureHandle,
    pub player: TextureHandle,
    pub chaser: TextureHandle,
    pub patroller: TextureHandle,
    pub hit: SoundHandle,
}

impl TestbedAssets {
    pub fn load() -> Self {
        let mut registry = AssetRegistry::new();
        register(&mut registry)
    }
}

pub fn register(assets: &mut AssetRegistry) -> TestbedAssets {
    TestbedAssets {
        floor: assets.texture("testbed/floor", "textures/test.png"),
        wall: assets.texture("testbed/wall", "textures/test.png"),
        player: assets.texture("testbed/player", "textures/test.png"),
        chaser: assets.texture("testbed/chaser", "textures/test.png"),
        patroller: assets.texture("testbed/patroller", "textures/test.png"),
        hit: assets.sound("testbed/hit", "audio/hit.wav"),
    }
}

#[cfg(test)]
mod tests {
    use super::register;
    use game_core::assets::AssetRegistry;

    #[test]
    fn testbed_assets_register_distinct_keys() {
        let mut registry = AssetRegistry::new();
        let assets = register(&mut registry);

        // All textures reuse the built-in test texture path, so they share a handle.
        assert_eq!(assets.player, assets.floor);
        assert_eq!(assets.chaser, assets.floor);
        assert_eq!(assets.patroller, assets.floor);
        assert_eq!(
            registry.texture_request("testbed/player").unwrap().path,
            "textures/test.png"
        );
        assert_eq!(
            registry.sound_request("testbed/hit").unwrap().path,
            "audio/hit.wav"
        );
    }
}

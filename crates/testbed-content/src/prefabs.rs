use game_core::builder::{PrefabId, PrefabRegistry};

use crate::assets::TestbedAssets;
use crate::input::TestbedActions;

pub const PLAYER: &str = "testbed/player";
pub const CHASER: &str = "testbed/chaser";
pub const PATROLLER: &str = "testbed/patroller";

#[derive(Clone, Copy, Debug)]
pub struct TestbedPrefabs {
    pub player: PrefabId,
    pub chaser: PrefabId,
    pub patroller: PrefabId,
}

pub fn register(
    prefabs: &mut PrefabRegistry,
    assets: TestbedAssets,
    actions: TestbedActions,
) -> TestbedPrefabs {
    let player_assets = assets;
    let player_actions = actions;
    let player = prefabs.register(PLAYER, move |world, position, _properties| {
        Ok(crate::spawn::spawn_player(
            world,
            position,
            &player_assets,
            &player_actions,
        ))
    });

    let chaser_assets = assets;
    let chaser = prefabs.register(CHASER, move |world, position, _properties| {
        Ok(crate::spawn::spawn_chaser(world, position, &chaser_assets))
    });

    let patroller_assets = assets;
    let patroller = prefabs.register(PATROLLER, move |world, position, _properties| {
        Ok(crate::spawn::spawn_patroller(
            world,
            position,
            &patroller_assets,
        ))
    });

    TestbedPrefabs {
        player,
        chaser,
        patroller,
    }
}

#[cfg(test)]
mod tests {
    use super::register;
    use crate::assets::TestbedAssets;
    use game_core::builder::{PrefabRegistry, PropertyBag};
    use game_core::input::InputRegistry;
    use game_core::world::World;

    #[test]
    fn testbed_prefabs_spawn_all_three_actors() {
        let assets = TestbedAssets::load();
        let mut input = InputRegistry::new();
        let actions = crate::input::register(&mut input);
        let mut registry = PrefabRegistry::new();
        let prefabs = register(&mut registry, assets, actions);
        let mut world = World::new();

        for prefab in [prefabs.player, prefabs.chaser, prefabs.patroller] {
            registry
                .spawn(
                    prefab,
                    &mut world,
                    glam::Vec2::ZERO,
                    &PropertyBag::default(),
                )
                .unwrap();
        }

        assert_eq!(world.ids().count(), 3);
    }
}

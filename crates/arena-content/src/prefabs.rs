use crate::assets::ArenaAssets;
use crate::engine::builder::{PrefabId, PrefabRegistry};
use crate::input::ArenaActions;

pub const PLAYER: &str = "arena/player";
pub const SLIME: &str = "arena/slime";

#[derive(Clone, Copy, Debug)]
pub struct ArenaPrefabs {
    pub player: PrefabId,
    pub slime: PrefabId,
}

pub fn register(
    prefabs: &mut PrefabRegistry,
    assets: ArenaAssets,
    actions: ArenaActions,
) -> ArenaPrefabs {
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

    let enemy_assets = assets;
    let slime = prefabs.register(SLIME, move |world, position, _properties| {
        Ok(crate::spawn::spawn_enemy(world, position, &enemy_assets))
    });

    ArenaPrefabs { player, slime }
}

#[cfg(test)]
mod tests {
    use super::register;
    use crate::assets::ArenaAssets;
    use crate::engine::builder::{PrefabRegistry, PropertyBag};
    use crate::engine::input::InputRegistry;
    use crate::engine::world::World;

    #[test]
    fn arena_prefabs_spawn_player_and_enemy() {
        let assets = ArenaAssets::load();
        let mut input = InputRegistry::new();
        let actions = crate::input::register(&mut input);
        let mut registry = PrefabRegistry::new();
        let prefabs = register(&mut registry, assets, actions);
        let mut world = World::new();

        registry
            .spawn(
                prefabs.player,
                &mut world,
                glam::Vec2::ZERO,
                &PropertyBag::default(),
            )
            .unwrap();
        registry
            .spawn(
                prefabs.slime,
                &mut world,
                glam::vec2(32.0, 0.0),
                &PropertyBag::default(),
            )
            .unwrap();

        assert_eq!(world.ids().count(), 2);
    }
}

use game_ai::{AiController, ChaseTarget, PathFollow};
use game_combat::{Faction, Health, MeleeAttack};
use game_map::GameMap;
use game_physics::Collider;
use glam::{Vec2, Vec4};

use crate::assets::ArenaAssets;
use crate::engine::builder::PrefabRegistry;
use crate::engine::world::{Entity, EntityId, Sprite};
use crate::game::World;
use crate::game::actor::{EnemyTag, MoveSpeed, Name, PlayerController};
use crate::input::ArenaActions;

const ENEMY_REPATH_SECONDS: f32 = 0.25;

pub fn spawn_map_objects(
    world: &mut World,
    map: &GameMap,
    prefabs: &PrefabRegistry,
) -> anyhow::Result<()> {
    for object in &map.objects {
        prefabs.spawn(object.prefab, world, object.position, &object.properties)?;
    }
    Ok(())
}

pub fn spawn_player(
    world: &mut World,
    at: Vec2,
    assets: &ArenaAssets,
    actions: &ArenaActions,
) -> EntityId {
    let size = Vec2::splat(20.0);
    world.spawn(
        Entity::new(at)
            .with(Name::new("Player"))
            .with(PlayerController {
                move_axis: actions.movement,
            })
            .with(Health::new(100))
            .with(MoveSpeed(130.0))
            .with(MeleeAttack::new(30.0, 25))
            .with(Faction::player())
            .with_sprite(
                Sprite::new(assets.player, size)
                    .layer(10)
                    .tint(Vec4::new(0.4, 0.7, 1.0, 1.0)),
            )
            .with_collider(Collider::box_of(size)),
    )
}

pub fn spawn_enemy(world: &mut World, at: Vec2, assets: &ArenaAssets) -> EntityId {
    let size = Vec2::splat(22.0);
    world.spawn(
        Entity::new(at)
            .with(Name::new("Enemy"))
            .with(EnemyTag)
            .with(Health::new(40))
            .with(MoveSpeed(80.0))
            .with(Faction::enemy())
            .with(MeleeAttack::new(26.0, 6).cooldown(0.75))
            .with(AiController::chase_player())
            .with(ChaseTarget::player(
                180.0,
                26.0 * 0.8,
                80.0,
                ENEMY_REPATH_SECONDS,
            ))
            .with(PathFollow::default())
            .with_sprite(
                Sprite::new(assets.enemy, size)
                    .layer(10)
                    .tint(Vec4::new(1.0, 0.4, 0.4, 1.0)),
            )
            .with_collider(Collider::box_of(size)),
    )
}

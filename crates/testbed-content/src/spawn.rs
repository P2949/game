use game_ai::{AiController, ChaseTarget, PathFollow, Patrol};
use game_combat::{Faction, Health, MeleeAttack};
use game_core::builder::PrefabRegistry;
use game_core::world::{Entity, EntityId, Sprite, World};
use game_map::GameMap;
use game_physics::Collider;
use glam::{Vec2, Vec4};

use crate::actor::{MoveSpeed, Name, PlayerController};
use crate::assets::TestbedAssets;
use crate::input::TestbedActions;

const CHASER_REPATH_SECONDS: f32 = 0.25;
/// Half the patrol corridor length, in world units (6 tiles either side of spawn).
const PATROL_SWEEP: f32 = 6.0 * crate::level::TILE;

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
    assets: &TestbedAssets,
    actions: &TestbedActions,
) -> EntityId {
    let size = Vec2::splat(20.0);
    world.spawn(
        Entity::new(at)
            .with(Name::new("Player"))
            .with(PlayerController {
                move_axis: actions.movement,
            })
            .with(Health::new(120))
            .with(MoveSpeed(140.0))
            .with(MeleeAttack::new(30.0, 25))
            .with(Faction::player())
            .with_sprite(
                Sprite::new(assets.player, size)
                    .layer(10)
                    .tint(Vec4::new(0.5, 0.9, 0.6, 1.0)),
            )
            .with_collider(Collider::box_of(size)),
    )
}

pub fn spawn_chaser(world: &mut World, at: Vec2, assets: &TestbedAssets) -> EntityId {
    let size = Vec2::splat(22.0);
    world.spawn(
        Entity::new(at)
            .with(Name::new("Chaser"))
            .with(Health::new(40))
            .with(MoveSpeed(90.0))
            .with(Faction::enemy())
            .with(MeleeAttack::new(26.0, 6).cooldown(0.75))
            .with(AiController::chase_player())
            .with(ChaseTarget::player(
                220.0,
                22.0 * 0.8,
                90.0,
                CHASER_REPATH_SECONDS,
            ))
            .with(PathFollow::default())
            .with_sprite(
                Sprite::new(assets.chaser, size)
                    .layer(10)
                    .tint(Vec4::new(1.0, 0.4, 0.4, 1.0)),
            )
            .with_collider(Collider::box_of(size)),
    )
}

pub fn spawn_patroller(world: &mut World, at: Vec2, assets: &TestbedAssets) -> EntityId {
    let size = Vec2::splat(22.0);
    let waypoints = vec![
        at - Vec2::new(PATROL_SWEEP, 0.0),
        at + Vec2::new(PATROL_SWEEP, 0.0),
    ];
    world.spawn(
        Entity::new(at)
            .with(Name::new("Patroller"))
            .with(Health::new(30))
            .with(MoveSpeed(70.0))
            .with(Faction::enemy())
            .with(MeleeAttack::new(24.0, 4).cooldown(1.0))
            .with(Patrol::new(waypoints, 70.0))
            .with_sprite(
                Sprite::new(assets.patroller, size)
                    .layer(10)
                    .tint(Vec4::new(1.0, 0.82, 0.3, 1.0)),
            )
            .with_collider(Collider::box_of(size)),
    )
}

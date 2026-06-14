use glam::{Vec2, Vec4};

use crate::engine::assets::Assets;
use crate::engine::tilemap::TileMap;
use crate::engine::world::{Collider, Entity, EntityId, Sprite};
use crate::game::World;
use crate::game::actor::{Actor, Enemy, Health, PathFollow, Player};

pub fn spawn_markers(world: &mut World, map: &TileMap, assets: &Assets) {
    for &(marker, col, row) in &map.spawns {
        let pos = map.cell_center(col, row);
        match marker {
            'P' => {
                spawn_player(world, pos, assets);
            }
            'E' => {
                spawn_enemy(world, pos, assets);
            }
            _ => {}
        }
    }
}

pub fn spawn_player(world: &mut World, at: Vec2, assets: &Assets) -> EntityId {
    let size = Vec2::splat(20.0);
    world.spawn(
        Entity::new(
            at,
            Actor::Player(Player {
                health: Health::new(100),
                speed: 130.0,
                attack_range: 30.0,
                attack_damage: 25,
            }),
        )
        .with_sprite(
            Sprite::new(assets.player, size)
                .layer(10)
                .tint(Vec4::new(0.4, 0.7, 1.0, 1.0)),
        )
        .with_collider(Collider::box_of(size)),
    )
}

pub fn spawn_enemy(world: &mut World, at: Vec2, assets: &Assets) -> EntityId {
    let size = Vec2::splat(22.0);
    world.spawn(
        Entity::new(
            at,
            Actor::Enemy(Enemy {
                health: Health::new(40),
                speed: 80.0,
                aggro_radius: 180.0,
                attack_range: 26.0,
                attack_damage: 6,
                attack_cooldown: 0.0,
                path: PathFollow::default(),
            }),
        )
        .with_sprite(
            Sprite::new(assets.enemy, size)
                .layer(10)
                .tint(Vec4::new(1.0, 0.4, 0.4, 1.0)),
        )
        .with_collider(Collider::box_of(size)),
    )
}

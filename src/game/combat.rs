use crate::engine::audio::Audio;
use crate::engine::input::{Action, Input};
use crate::engine::world::EntityId;
use crate::game::World;
use crate::game::actor::Actor;

const ENEMY_ATTACK_COOLDOWN: f32 = 0.75;

pub fn tick(world: &mut World, input: &Input, audio: Audio<'_>, dt: f32) {
    let Some((player_id, player_pos, player_range, player_damage)) = player_snapshot(world) else {
        return;
    };

    if input.pressed(Action::Attack) {
        if let Some(target) = nearest_enemy_in_range(world, player_pos, player_range) {
            if damage_enemy(world, target, player_damage) {
                audio.hit();
            }
        }
    }

    let mut player_damage_taken = 0;
    let enemy_ids = world.ids_where(|entity| matches!(entity.user, Actor::Enemy(_)));
    for id in enemy_ids {
        let Some(entity) = world.get_mut(id) else {
            continue;
        };
        let Actor::Enemy(enemy) = &mut entity.user else {
            continue;
        };
        if enemy.health.is_dead() {
            continue;
        }

        enemy.attack_cooldown = (enemy.attack_cooldown - dt).max(0.0);
        if enemy.attack_cooldown == 0.0
            && entity.transform.pos.distance(player_pos) <= enemy.attack_range
        {
            enemy.attack_cooldown = ENEMY_ATTACK_COOLDOWN;
            player_damage_taken += enemy.attack_damage;
        }
    }

    if player_damage_taken > 0 {
        if let Some(player) = world.get_mut(player_id) {
            if let Actor::Player(player_actor) = &mut player.user {
                player_actor.health.damage(player_damage_taken);
                audio.hit();
            }
        }
    }

    for id in world
        .ids_where(|entity| matches!(entity.user, Actor::Enemy(enemy) if enemy.health.is_dead()))
    {
        world.despawn(id);
    }
}

pub fn kill_player(world: &mut World) {
    for (_, entity) in world.iter_mut() {
        if let Actor::Player(player) = &mut entity.user {
            player.health.damage(player.health.current);
            entity.velocity = glam::Vec2::ZERO;
        }
    }
}

pub fn player_is_dead(world: &World) -> bool {
    world
        .iter()
        .any(|(_, entity)| matches!(entity.user, Actor::Player(player) if player.health.is_dead()))
}

fn player_snapshot(world: &World) -> Option<(EntityId, glam::Vec2, f32, i32)> {
    world.iter().find_map(|(id, entity)| {
        if let Actor::Player(player) = entity.user {
            Some((
                id,
                entity.transform.pos,
                player.attack_range,
                player.attack_damage,
            ))
        } else {
            None
        }
    })
}

fn nearest_enemy_in_range(world: &World, player_pos: glam::Vec2, range: f32) -> Option<EntityId> {
    world
        .iter()
        .filter_map(|(id, entity)| {
            let Actor::Enemy(enemy) = entity.user else {
                return None;
            };
            if enemy.health.is_dead() {
                return None;
            }
            let dist_sq = entity.transform.pos.distance_squared(player_pos);
            (dist_sq <= range * range).then_some((id, dist_sq))
        })
        .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(id, _)| id)
}

fn damage_enemy(world: &mut World, id: EntityId, amount: i32) -> bool {
    let Some(entity) = world.get_mut(id) else {
        return false;
    };
    let Actor::Enemy(enemy) = &mut entity.user else {
        return false;
    };
    enemy.health.damage(amount);
    true
}

#[cfg(test)]
mod tests {
    use crate::engine::audio::Audio;
    use crate::engine::input::Input;
    use crate::engine::world::Entity;
    use crate::game::World;
    use crate::game::actor::{Actor, Enemy, Health, PathFollow, Player};
    use crate::platform::input::{FrameActions, InputState};

    use super::{kill_player, player_is_dead, tick};

    fn input(action_pressed: bool) -> Input {
        Input::new(
            &InputState::default(),
            FrameActions {
                action_pressed,
                ..Default::default()
            },
        )
    }

    fn world_with_player_and_enemy(enemy_pos: glam::Vec2) -> World {
        let mut world = World::new();
        world.spawn(Entity::new(
            glam::Vec2::ZERO,
            Actor::Player(Player {
                health: Health::new(100),
                speed: 10.0,
                attack_range: 20.0,
                attack_damage: 50,
            }),
        ));
        world.spawn(Entity::new(
            enemy_pos,
            Actor::Enemy(Enemy {
                health: Health::new(40),
                speed: 10.0,
                aggro_radius: 100.0,
                attack_range: 5.0,
                attack_damage: 7,
                attack_cooldown: 0.0,
                path: PathFollow::default(),
            }),
        ));
        world
    }

    #[test]
    fn player_attack_damages_and_despawns_dead_enemy() {
        let mut world = world_with_player_and_enemy(glam::vec2(10.0, 0.0));

        tick(&mut world, &input(true), Audio::new(None), 1.0 / 120.0);

        assert!(
            world
                .iter()
                .all(|(_, entity)| !matches!(entity.user, Actor::Enemy(_)))
        );
    }

    #[test]
    fn enemy_attack_damages_player() {
        let mut world = world_with_player_and_enemy(glam::vec2(4.0, 0.0));

        tick(&mut world, &input(false), Audio::new(None), 1.0 / 120.0);

        let player = world
            .iter()
            .find_map(|(_, entity)| {
                if let Actor::Player(player) = entity.user {
                    Some(player)
                } else {
                    None
                }
            })
            .unwrap();
        assert_eq!(player.health.current, 93);
    }

    #[test]
    fn kill_player_marks_player_dead() {
        let mut world = world_with_player_and_enemy(glam::vec2(100.0, 0.0));

        kill_player(&mut world);

        assert!(player_is_dead(&world));
    }
}

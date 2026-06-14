use glam::Vec2;

use crate::engine::input::Input;
use crate::engine::nav::NavGrid;
use crate::game::World;
use crate::game::actor::Actor;

const REPATH_SECONDS: f32 = 0.25;
const WAYPOINT_REACHED_DISTANCE: f32 = 4.0;

pub fn player_pos(world: &World) -> Option<Vec2> {
    world.iter().find_map(|(_, entity)| {
        matches!(entity.user, Actor::Player(_)).then_some(entity.transform.pos)
    })
}

pub fn stop_all(world: &mut World) {
    for (_, entity) in world.iter_mut() {
        entity.velocity = Vec2::ZERO;
    }
}

pub fn drive_player(world: &mut World, input: &Input) {
    for (_, entity) in world.iter_mut() {
        if let Actor::Player(player) = entity.user {
            entity.velocity = input.move_axis() * player.speed;
        }
    }
}

pub fn chase_player(world: &mut World, nav: &NavGrid, dt: f32) {
    let Some(player_pos) = player_pos(world) else {
        return;
    };

    let enemy_ids = world.ids_where(|entity| matches!(entity.user, Actor::Enemy(_)));
    for id in enemy_ids {
        let Some(entity) = world.get_mut(id) else {
            continue;
        };
        let Actor::Enemy(enemy) = &mut entity.user else {
            continue;
        };

        let to_player = player_pos - entity.transform.pos;
        let distance = to_player.length();
        if distance > enemy.aggro_radius {
            entity.velocity = Vec2::ZERO;
            enemy.path.next = None;
            enemy.path.repath_timer = 0.0;
            continue;
        }

        if distance <= enemy.attack_range * 0.8 {
            entity.velocity = Vec2::ZERO;
            enemy.path.next = None;
            continue;
        }

        if let Some(next) = enemy.path.next {
            if entity.transform.pos.distance(next) <= WAYPOINT_REACHED_DISTANCE {
                enemy.path.next = None;
            }
        }

        enemy.path.repath_timer = (enemy.path.repath_timer - dt).max(0.0);
        if enemy.path.repath_timer == 0.0 || enemy.path.next.is_none() {
            enemy.path.repath_timer = REPATH_SECONDS;
            enemy.path.next = nav
                .find_path(entity.transform.pos, player_pos)
                .and_then(|path| path.into_iter().next());
        }

        let target = enemy.path.next.unwrap_or(player_pos);
        let desired = target - entity.transform.pos;
        entity.velocity = desired.normalize_or_zero() * enemy.speed;
    }
}

#[cfg(test)]
mod tests {
    use crate::engine::nav::NavGrid;
    use crate::engine::tilemap::TileMap;
    use crate::engine::world::Entity;
    use crate::game::World;
    use crate::game::actor::{Actor, Enemy, Health, PathFollow, Player};

    use super::{chase_player, drive_player, player_pos};

    #[test]
    fn drive_player_maps_input_axis_to_velocity() {
        let mut world = World::new();
        world.spawn(Entity::new(
            glam::Vec2::ZERO,
            Actor::Player(Player {
                health: Health::new(10),
                speed: 10.0,
                attack_range: 1.0,
                attack_damage: 1,
            }),
        ));
        let mut input_state = crate::platform::input::InputState::default();
        input_state.set_move_x(1.0);
        let input = crate::engine::input::Input::new(
            &input_state,
            crate::platform::input::FrameActions::default(),
        );

        drive_player(&mut world, &input);

        assert_eq!(
            world.iter().next().unwrap().1.velocity,
            glam::vec2(10.0, 0.0)
        );
    }

    #[test]
    fn chase_player_sets_enemy_velocity_toward_path() {
        let map = TileMap::from_rows(&["....."], 10.0);
        let nav = NavGrid::from_tilemap(&map);
        let mut world = World::new();
        world.spawn(Entity::new(
            glam::vec2(5.0, 5.0),
            Actor::Player(Player {
                health: Health::new(10),
                speed: 10.0,
                attack_range: 1.0,
                attack_damage: 1,
            }),
        ));
        world.spawn(Entity::new(
            glam::vec2(35.0, 5.0),
            Actor::Enemy(Enemy {
                health: Health::new(10),
                speed: 5.0,
                aggro_radius: 100.0,
                attack_range: 1.0,
                attack_damage: 1,
                attack_cooldown: 0.0,
                path: PathFollow::default(),
            }),
        ));

        chase_player(&mut world, &nav, 1.0 / 120.0);

        let enemy = world
            .iter()
            .find(|(_, entity)| matches!(entity.user, Actor::Enemy(_)))
            .unwrap()
            .1;
        assert!(enemy.velocity.x < 0.0);
        assert_eq!(player_pos(&world), Some(glam::vec2(5.0, 5.0)));
    }
}

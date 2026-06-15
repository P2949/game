use crate::actor::{MoveSpeed, PlayerController};
use game_kit::prelude::*;

pub fn player_pos(world: &World) -> Option<Vec2> {
    world
        .ids_with::<PlayerController>()
        .into_iter()
        .find_map(|id| world.get::<Transform>(id).map(|transform| transform.pos))
}

pub fn stop_all(world: &mut World) {
    for id in world.ids_with::<Velocity>() {
        if let Some(velocity) = world.get_mut::<Velocity>(id) {
            velocity.0 = Vec2::ZERO;
        }
    }
}

pub fn drive_player(world: &mut World, input: &Input) {
    for id in world.ids_with::<PlayerController>() {
        let Some(controller) = world.get::<PlayerController>(id).copied() else {
            continue;
        };
        let Some(speed) = world.get::<MoveSpeed>(id).copied() else {
            continue;
        };
        if let Some(velocity) = world.get_mut::<Velocity>(id) {
            velocity.0 = input.axis2d(controller.move_axis) * speed.0;
        }
    }
}

pub fn chase_player(world: &mut World, nav: &NavGrid, dt: f32) {
    // Reuse the shared chase behavior from `game-ai`, resolving the arena's
    // player position as the chase target.
    let target = player_pos(world);
    chase_system(world, nav, target, dt);
}

#[cfg(test)]
mod tests {
    use crate::actor::{MoveSpeed, PlayerController};
    use game_kit::prelude::*;

    use super::{chase_player, drive_player, player_pos};

    #[test]
    fn drive_player_maps_input_axis_to_velocity() {
        let mut world = World::new();
        let id = world.spawn(
            Entity::new(glam::Vec2::ZERO)
                .with(PlayerController {
                    move_axis: Axis2dId(0),
                })
                .with(MoveSpeed(10.0)),
        );
        let input = Input::default().with_axis2d(Axis2dId(0), glam::vec2(1.0, 0.0));

        drive_player(&mut world, &input);

        assert_eq!(world.get::<Velocity>(id).unwrap().0, glam::vec2(10.0, 0.0));
    }

    #[test]
    fn chase_player_sets_enemy_velocity_toward_path() {
        let map = TileMap::from_rows(&["....."], 10.0);
        let nav = NavGrid::from_tilemap(&map);
        let mut world = World::new();
        world.spawn(Entity::new(glam::vec2(5.0, 5.0)).with(PlayerController {
            move_axis: Axis2dId(0),
        }));
        let enemy = world.spawn(
            Entity::new(glam::vec2(35.0, 5.0))
                .with(MoveSpeed(5.0))
                .with(AiController::chase_player())
                .with(ChaseTarget::player(100.0, 1.0, 5.0, 0.25))
                .with(PathFollow::default()),
        );

        chase_player(&mut world, &nav, 1.0 / 120.0);

        assert!(world.get::<Velocity>(enemy).unwrap().0.x < 0.0);
        assert_eq!(player_pos(&world), Some(glam::vec2(5.0, 5.0)));
        assert_eq!(
            world.get::<Transform>(enemy).unwrap().pos,
            glam::vec2(35.0, 5.0)
        );
    }
}

use game_kit::prelude::*;
use glam::Vec2;

use crate::actor::{MoveSpeed, PlayerController};

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

#[cfg(test)]
mod tests {
    use game_kit::prelude::*;

    use super::{drive_player, player_pos};
    use crate::actor::{MoveSpeed, PlayerController};

    #[test]
    fn drive_player_maps_input_axis_to_velocity() {
        let mut world = World::new();
        let id = world.spawn(
            Entity::new(glam::Vec2::ZERO)
                .with(PlayerController {
                    move_axis: Axis2dId(0),
                })
                .with(MoveSpeed(140.0)),
        );
        let input = Input::default().with_axis2d(Axis2dId(0), glam::vec2(1.0, 0.0));

        drive_player(&mut world, &input);

        assert_eq!(world.get::<Velocity>(id).unwrap().0, glam::vec2(140.0, 0.0));
        assert_eq!(player_pos(&world), Some(glam::Vec2::ZERO));
    }
}

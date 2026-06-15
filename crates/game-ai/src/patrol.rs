use game_core::world::{Transform, Velocity, World};
use glam::Vec2;

/// Reusable patrol behavior: an ordered, looping list of world-space waypoints the
/// entity walks between at a fixed speed. State (`index`) lives on the component so
/// patrol routes are pure data that content can place on any entity.
#[derive(Clone, Debug)]
pub struct Patrol {
    pub waypoints: Vec<Vec2>,
    pub speed: f32,
    pub index: usize,
    pub arrive_radius: f32,
}

impl Patrol {
    pub fn new(waypoints: Vec<Vec2>, speed: f32) -> Self {
        Self {
            waypoints,
            speed,
            index: 0,
            arrive_radius: 4.0,
        }
    }

    pub fn arrive_radius(mut self, radius: f32) -> Self {
        self.arrive_radius = radius.max(0.0);
        self
    }
}

/// Steers every entity with a [`Patrol`] toward its current waypoint, advancing to
/// the next (looping) once within `arrive_radius`. Sets [`Velocity`]; the physics
/// `movement_system` applies axis-separated wall collision, subject to the usual
/// discrete-step tunneling limits.
pub fn patrol_system(world: &mut World, _dt: f32) {
    for id in world.ids_with::<Patrol>() {
        let Some(pos) = world.get::<Transform>(id).map(|transform| transform.pos) else {
            continue;
        };
        let Some((index, len, speed, arrive)) = world.get::<Patrol>(id).map(|patrol| {
            (
                patrol.index,
                patrol.waypoints.len(),
                patrol.speed,
                patrol.arrive_radius,
            )
        }) else {
            continue;
        };
        if len == 0 {
            // An empty route is a stop, not a coast: clear any prior velocity so
            // the physics step does not keep carrying the entity along.
            if let Some(velocity) = world.get_mut::<Velocity>(id) {
                velocity.0 = Vec2::ZERO;
            }
            continue;
        }

        let current = world.get::<Patrol>(id).unwrap().waypoints[index % len];
        let target = if pos.distance(current) <= arrive {
            let next = (index + 1) % len;
            if let Some(patrol) = world.get_mut::<Patrol>(id) {
                patrol.index = next;
            }
            world.get::<Patrol>(id).unwrap().waypoints[next % len]
        } else {
            current
        };

        let desired = (target - pos).normalize_or_zero() * speed;
        if let Some(velocity) = world.get_mut::<Velocity>(id) {
            velocity.0 = desired;
        }
    }
}

#[cfg(test)]
mod tests {
    use game_core::world::{Entity, Velocity, World};

    use super::{Patrol, patrol_system};

    #[test]
    fn patrol_steers_toward_first_waypoint() {
        let mut world = World::new();
        let id = world.spawn(Entity::new(glam::vec2(0.0, 0.0)).with(Patrol::new(
            vec![glam::vec2(100.0, 0.0), glam::vec2(-100.0, 0.0)],
            50.0,
        )));

        patrol_system(&mut world, 1.0 / 60.0);

        assert_eq!(world.get::<Velocity>(id).unwrap().0, glam::vec2(50.0, 0.0));
    }

    #[test]
    fn patrol_with_no_waypoints_clears_velocity() {
        let mut world = World::new();
        let id = world.spawn(Entity::new(glam::Vec2::ZERO).with(Patrol::new(Vec::new(), 50.0)));
        world.get_mut::<Velocity>(id).unwrap().0 = glam::vec2(5.0, 5.0);

        patrol_system(&mut world, 1.0 / 60.0);

        assert_eq!(world.get::<Velocity>(id).unwrap().0, glam::Vec2::ZERO);
    }

    #[test]
    fn patrol_advances_to_next_waypoint_on_arrival() {
        let mut world = World::new();
        let id = world.spawn(
            Entity::new(glam::vec2(100.0, 0.0)).with(
                Patrol::new(vec![glam::vec2(100.0, 0.0), glam::vec2(-100.0, 0.0)], 50.0)
                    .arrive_radius(2.0),
            ),
        );

        patrol_system(&mut world, 1.0 / 60.0);

        // Within arrive radius of waypoint 0, so it advances and heads to waypoint 1.
        assert_eq!(world.get::<Patrol>(id).unwrap().index, 1);
        assert_eq!(world.get::<Velocity>(id).unwrap().0, glam::vec2(-50.0, 0.0));
    }
}

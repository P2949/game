use game_core::world::{Transform, Velocity, World};
use game_map::nav::NavGrid;
use glam::Vec2;

const WAYPOINT_REACHED_DISTANCE: f32 = 4.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AiBehaviorId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AiState {
    Idle,
    Chasing,
}

#[derive(Clone, Copy, Debug)]
pub struct AiController {
    pub behavior: AiBehaviorId,
    pub state: AiState,
}

impl AiController {
    pub fn chase_player() -> Self {
        Self {
            behavior: AiBehaviorId(0),
            state: AiState::Idle,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetSelector {
    Player,
}

#[derive(Clone, Copy, Debug)]
pub struct ChaseTarget {
    pub selector: TargetSelector,
    pub aggro_radius: f32,
    pub stop_distance: f32,
    pub speed: f32,
    pub repath_seconds: f32,
}

impl ChaseTarget {
    pub fn player(aggro_radius: f32, stop_distance: f32, speed: f32, repath_seconds: f32) -> Self {
        Self {
            selector: TargetSelector::Player,
            aggro_radius,
            stop_distance,
            speed,
            repath_seconds,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PathFollow {
    pub next_waypoint: Option<Vec2>,
    pub repath_timer: f32,
}

/// Reusable chase behavior: steers every entity with a [`ChaseTarget`]/[`PathFollow`]
/// toward `target_pos` along a [`NavGrid`] path, updating its [`Velocity`] and
/// [`AiController`] state. The caller resolves `target_pos` (e.g. the player's
/// position) so this system stays free of any content-specific target component.
pub fn chase_system(world: &mut World, nav: &NavGrid, target_pos: Option<Vec2>, dt: f32) {
    let Some(target_pos) = target_pos else {
        return;
    };

    for id in world.ids_with::<ChaseTarget>() {
        let Some(transform) = world.get::<Transform>(id).copied() else {
            continue;
        };

        let mut desired_velocity = Vec2::ZERO;
        {
            let Some(chase) = world.get::<ChaseTarget>(id).copied() else {
                continue;
            };
            let Some(path) = world.get_mut::<PathFollow>(id) else {
                continue;
            };

            let to_target = target_pos - transform.pos;
            let distance = to_target.length();
            if distance > chase.aggro_radius {
                path.next_waypoint = None;
                path.repath_timer = 0.0;
            } else if distance <= chase.stop_distance {
                path.next_waypoint = None;
            } else {
                if let Some(next) = path.next_waypoint {
                    if transform.pos.distance(next) <= WAYPOINT_REACHED_DISTANCE {
                        path.next_waypoint = None;
                    }
                }

                path.repath_timer = (path.repath_timer - dt).max(0.0);
                if path.repath_timer == 0.0 || path.next_waypoint.is_none() {
                    path.repath_timer = chase.repath_seconds;
                    path.next_waypoint = nav
                        .find_path(transform.pos, target_pos)
                        .and_then(|path| path.into_iter().next());
                }

                let target = path.next_waypoint.unwrap_or(target_pos);
                desired_velocity = (target - transform.pos).normalize_or_zero() * chase.speed;
            }
        }

        if let Some(velocity) = world.get_mut::<Velocity>(id) {
            velocity.0 = desired_velocity;
        }
        if let Some(controller) = world.get_mut::<AiController>(id) {
            controller.state = if desired_velocity == Vec2::ZERO {
                AiState::Idle
            } else {
                AiState::Chasing
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use game_core::world::{Entity, Velocity, World};
    use game_map::nav::NavGrid;
    use game_map::tilemap::TileMap;

    use super::{AiController, ChaseTarget, PathFollow, chase_system};

    #[test]
    fn chase_system_steers_toward_target_along_path() {
        let map = TileMap::from_rows(&["....."], 10.0);
        let nav = NavGrid::from_tilemap(&map);
        let mut world = World::new();
        let enemy = world.spawn(
            Entity::new(glam::vec2(35.0, 5.0))
                .with(AiController::chase_player())
                .with(ChaseTarget::player(100.0, 1.0, 5.0, 0.25))
                .with(PathFollow::default()),
        );

        chase_system(&mut world, &nav, Some(glam::vec2(5.0, 5.0)), 1.0 / 120.0);

        assert!(world.get::<Velocity>(enemy).unwrap().0.x < 0.0);
    }

    #[test]
    fn chase_system_idles_without_target() {
        let map = TileMap::from_rows(&["....."], 10.0);
        let nav = NavGrid::from_tilemap(&map);
        let mut world = World::new();
        let enemy = world.spawn(
            Entity::new(glam::vec2(35.0, 5.0))
                .with(AiController::chase_player())
                .with(ChaseTarget::player(100.0, 1.0, 5.0, 0.25))
                .with(PathFollow::default()),
        );

        chase_system(&mut world, &nav, None, 1.0 / 120.0);

        assert_eq!(world.get::<Velocity>(enemy).unwrap().0, glam::Vec2::ZERO);
    }
}

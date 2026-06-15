use game_kit::prelude::*;

use crate::actor::{MoveSpeed, Name, PlayerController};
use crate::assets::TestbedAssets;
use crate::input::TestbedActions;

const CHASER_REPATH_SECONDS: f32 = 0.25;
const PATROL_SWEEP: f32 = 6.0 * crate::level::TILE;

pub fn register(game: &mut GameApp, assets: TestbedAssets, actions: TestbedActions) {
    game.prefab("testbed/player", move |prefab| {
        prefab
            .spawn(move |at| {
                (
                    Name::new("Player"),
                    Transform::at(at),
                    Velocity::default(),
                    PlayerController {
                        move_axis: actions.movement,
                    },
                    Health::new(120),
                    MoveSpeed(140.0),
                    MeleeAttack::new(30.0, 25),
                    Faction::player(),
                    Sprite::new(assets.player, vec2s(20.0))
                        .layer(10)
                        .tint(vec4(0.5, 0.9, 0.6, 1.0)),
                    Collider::box_of(vec2s(20.0)),
                )
            })
            .require::<Transform>()
            .require::<Collider>()
            .require::<Sprite>()
            .require::<Health>()
            .require::<Faction>()
            .require::<PlayerController>();
    });

    game.prefab("testbed/chaser", move |prefab| {
        prefab
            .spawn(move |at| {
                (
                    Name::new("Chaser"),
                    Transform::at(at),
                    Velocity::default(),
                    Health::new(40),
                    MoveSpeed(90.0),
                    Faction::enemy(),
                    MeleeAttack::new(26.0, 6).cooldown(0.75),
                    AiController::chase_player(),
                    ChaseTarget::player(220.0, 22.0 * 0.8, 90.0, CHASER_REPATH_SECONDS),
                    PathFollow::default(),
                    Sprite::new(assets.chaser, vec2s(22.0))
                        .layer(10)
                        .tint(vec4(1.0, 0.4, 0.4, 1.0)),
                    Collider::box_of(vec2s(22.0)),
                )
            })
            .require::<Transform>()
            .require::<Collider>()
            .require::<Sprite>()
            .require::<Health>()
            .require::<Faction>()
            .require::<AiController>();
    });

    game.prefab("testbed/patroller", move |prefab| {
        prefab
            .spawn(move |at| {
                let waypoints = vec![at - vec2(PATROL_SWEEP, 0.0), at + vec2(PATROL_SWEEP, 0.0)];
                (
                    Name::new("Patroller"),
                    Transform::at(at),
                    Velocity::default(),
                    Health::new(30),
                    MoveSpeed(70.0),
                    Faction::enemy(),
                    MeleeAttack::new(24.0, 4).cooldown(1.0),
                    Patrol::new(waypoints, 70.0),
                    Sprite::new(assets.patroller, vec2s(22.0))
                        .layer(10)
                        .tint(vec4(1.0, 0.82, 0.3, 1.0)),
                    Collider::box_of(vec2s(22.0)),
                )
            })
            .require::<Transform>()
            .require::<Collider>()
            .require::<Sprite>()
            .require::<Health>()
            .require::<Faction>()
            .require::<Patrol>();
    });
}

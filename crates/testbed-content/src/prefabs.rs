use game_kit::prelude::*;

use crate::assets::TestbedAssets;
use crate::input::TestbedActions;

const CHASER_REPATH_SECONDS: f32 = 0.25;
const PATROL_SWEEP: f32 = 6.0 * crate::level::TILE;

pub fn register(game: &mut GameApp, assets: TestbedAssets, actions: TestbedActions) -> Result<()> {
    game.player_prefab("testbed/player")
        .named("Player")
        .sprite(assets.player)
        .size(20.0)
        .tint(vec4(0.5, 0.9, 0.6, 1.0))
        .health(120)
        .moves_with(actions.movement, 140.0)
        .melee(30.0, 25)
        .build()?;

    game.enemy_prefab("testbed/chaser")
        .named("Chaser")
        .sprite(assets.chaser)
        .size(22.0)
        .tint(vec4(1.0, 0.4, 0.4, 1.0))
        .health(40)
        .speed(90.0)
        .chases_player()
        .chase_range(220.0)
        .stop_distance(22.0 * 0.8)
        .repath_seconds(CHASER_REPATH_SECONDS)
        .melee(26.0, 6)
        .build()?;

    game.prefab("testbed/patroller", move |prefab| {
        prefab
            .spawn(move |at| {
                let waypoints = vec![at - vec2(PATROL_SWEEP, 0.0), at + vec2(PATROL_SWEEP, 0.0)];
                (
                    Name::new("Patroller"),
                    Transform::at(at),
                    Velocity::default(),
                    Enemy,
                    Health::new(30),
                    Speed::new(70.0),
                    Faction::enemy(),
                    MeleeAttack::new(24.0, 4).cooldown(1.0),
                    Patrol::new(waypoints, 70.0),
                    Sprite::new(assets.patroller, vec2s(22.0))
                        .layer(10)
                        .tint(vec4(1.0, 0.82, 0.3, 1.0)),
                    Collider::box_of(vec2s(22.0)),
                )
            })?
            .require::<Transform>()
            .require::<Collider>()
            .require::<Sprite>()
            .require::<Health>()
            .require::<Faction>()
            .require::<Enemy>()
            .require::<Patrol>();
        Ok(())
    })?;

    Ok(())
}

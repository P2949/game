use crate::actor::{EnemyTag, MoveSpeed, Name, PlayerController};
use crate::assets::ArenaAssets;
use crate::input::ArenaActions;
use game_kit::prelude::*;

pub const PLAYER: &str = "arena/player";
pub const SLIME: &str = "arena/slime";

pub fn register(game: &mut GameApp, assets: ArenaAssets, actions: ArenaActions) -> Result<()> {
    game.prefab(PLAYER, |prefab| {
        let size = vec2(20.0, 20.0);
        prefab
            .spawn(move |at| {
                (
                    Name::new("Player"),
                    Transform::at(at),
                    Velocity::default(),
                    PlayerController {
                        move_axis: actions.movement,
                    },
                    Health::new(100),
                    MoveSpeed(130.0),
                    MeleeAttack::new(30.0, 25),
                    Faction::player(),
                    Sprite::new(assets.player, size)
                        .layer(10)
                        .tint(vec4(0.4, 0.7, 1.0, 1.0)),
                    Collider::box_of(size),
                )
            })?
            .require::<Transform>()
            .require::<Collider>()
            .require::<Sprite>()
            .require::<Health>()
            .require::<Faction>()
            .require::<PlayerController>();
        Ok(())
    })?;

    game.prefab(SLIME, |prefab| {
        let size = vec2(22.0, 22.0);
        prefab
            .spawn(move |at| {
                (
                    Name::new("Enemy"),
                    Transform::at(at),
                    Velocity::default(),
                    EnemyTag,
                    Health::new(40),
                    MoveSpeed(80.0),
                    Faction::enemy(),
                    MeleeAttack::new(26.0, 6).cooldown(0.75),
                    AiController::chase_player(),
                    ChaseTarget::player(180.0, 26.0 * 0.8, 80.0, 0.25),
                    PathFollow::default(),
                    Sprite::new(assets.enemy, size)
                        .layer(10)
                        .tint(vec4(1.0, 0.4, 0.4, 1.0)),
                    Collider::box_of(size),
                )
            })?
            .require::<Transform>()
            .require::<Collider>()
            .require::<Sprite>()
            .require::<Health>()
            .require::<Faction>()
            .require::<EnemyTag>()
            .require::<AiController>();
        Ok(())
    })?;

    Ok(())
}

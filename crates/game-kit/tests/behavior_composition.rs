use game_kit::advanced::prelude::{ChaseTarget, Velocity};
use game_kit::beginner::prelude::*;
use game_kit::testing::GameTestHarness;

struct ComposedBehaviorPlugin;

struct MovementBehaviorPlugin;

impl GamePlugin for MovementBehaviorPlugin {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        let controls = game.input(|input| input.top_down_controls())?;
        game.player_prefab("player")
            .sprite(TextureHandle(1))
            .moves_with(controls.movement, 120.0)
            .build()?;
        game.map("movement")
            .tiles(["#####", "#P..#", "#####"])
            .simple_theme(TextureHandle(0), TextureHandle(0))
            .legend('P', "player")
            .start();
        game.on_start(|game| {
            game.init_resource::<SimpleGameState>();
            game.spawn_start_map()
        });
        game.use_behavior(MovementBehavior)?;
        Ok(())
    }
}

impl GamePlugin for ComposedBehaviorPlugin {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        let controls = game.input(|input| input.top_down_controls())?;
        game.player_prefab("player")
            .sprite(TextureHandle(1))
            .moves_with(controls.movement, 120.0)
            .build()?;
        game.enemy_prefab("slime")
            .sprite(TextureHandle(2))
            .chases_player()
            .speed(80.0)
            .build()?;
        game.map("composed")
            .tile_size(32.0)
            .tiles(["########", "#P....E#", "########"])
            .simple_theme(TextureHandle(0), TextureHandle(0))
            .legend('P', "player")
            .legend('E', "slime")
            .start();

        // The preset supplies setup/state handling; individual behaviors add
        // only the systems this game wants to customize.
        game.use_top_down_game().controls(controls).build();
        game.use_behavior(EnemyChaseBehavior { range: Some(240.0) })?;
        game.use_behavior(CollisionBehavior)?;
        Ok(())
    }
}

#[test]
fn individual_behaviors_compose_with_the_top_down_preset() {
    let mut game = GameTestHarness::from_plugin(ComposedBehaviorPlugin).unwrap();
    let before = game.enemy(0).position();

    game.fixed_step(0.25);

    let chaser = game.world().ids_with::<ChaseTarget>()[0];
    assert_eq!(
        game.world()
            .get::<ChaseTarget>(chaser)
            .unwrap()
            .aggro_radius,
        240.0
    );
    assert!(
        game.enemy(0).position().x < before.x,
        "the independently registered chase and collision behaviors should move the enemy"
    );
}

#[test]
fn movement_behavior_is_a_standalone_unit_plugin() {
    let mut game = GameTestHarness::from_plugin(MovementBehaviorPlugin)
        .unwrap()
        .set_axis("move", glam::Vec2::X);
    let player = game.world().ids_with::<Player>()[0];

    game.fixed_step(0.25);

    assert_eq!(
        game.world().get::<Velocity>(player).unwrap().0,
        glam::vec2(120.0, 0.0)
    );
}

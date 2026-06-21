use game_kit::advanced::prelude::*;
use game_kit::testing::GameTestHarness;

fn move_players(
    mut players: Query<(&mut Transform, &Velocity), With<Player>>,
    input: Res<Input>,
    dt: DeltaTime,
) {
    let _ = input.mouse_position();
    for (_, (transform, velocity)) in &mut players {
        let _ = velocity.0;
        transform.pos.x += 2.0 + dt.0;
    }
}

fn copy_player_positions(
    mut transforms: Query<&mut Transform, With<Player>>,
    mut velocities: Query<&Velocity, With<Player>>,
) {
    let _ = velocities.iter().next().map(|(_, velocity)| velocity.0);
    for (_, transform) in &mut transforms {
        transform.pos.y += 3.0;
    }
}

struct QueryPlugin;

impl GamePlugin for QueryPlugin {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        let controls = game.input(|input| input.top_down_controls())?;
        game.player_prefab("player")
            .sprite(TextureHandle(1))
            .moves_with(controls.movement, 120.0)
            .build()?;
        game.map("query")
            .tiles(["###", "#P#", "###"])
            .simple_theme(TextureHandle(0), TextureHandle(0))
            .legend('P', "player")
            .start();
        game.use_top_down_game().controls(controls).build();
        game.fixed_params(move_players)?;
        game.fixed_params(copy_player_positions)?;
        Ok(())
    }
}

fn conflicting_access(
    _write: Query<&mut Transform, With<Player>>,
    _read: Query<&Transform, With<Player>>,
) {
}

struct ConflictingQueryPlugin;

impl GamePlugin for ConflictingQueryPlugin {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        game.fixed_params(conflicting_access)
    }
}

#[test]
fn disjoint_query_parameters_run_through_the_game_app_schedule() {
    let mut game = GameTestHarness::from_plugin(QueryPlugin).unwrap();
    let before = game.player().position();

    game.fixed_step(0.25);

    let after = game.player().position();
    assert_eq!(after.x, before.x + 2.25);
    assert_eq!(after.y, before.y + 3.0);
}

#[test]
fn conflicting_query_parameters_are_rejected_while_registering() {
    let error = match GameTestHarness::from_plugin(ConflictingQueryPlugin) {
        Ok(_) => panic!("conflicting parameter system should be rejected"),
        Err(error) => error.to_string(),
    };
    assert!(error.contains("conflicting component access"));
    assert!(error.contains("Transform"));
}

use game_kit::beginner::prelude::*;
use game_kit::testing::GameTestHarness;

struct ResetPlugin;

impl GamePlugin for ResetPlugin {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        let controls = game.input(|input| input.top_down_controls())?;
        game.player_prefab("player")
            .sprite(TextureHandle(1))
            .moves_with(controls.movement, 120.0)
            .build()?;
        game.map("reset")
            .tiles(["###", "#P#", "###"])
            .simple_theme(TextureHandle(0), TextureHandle(0))
            .legend('P', "player")
            .start();
        game.use_top_down_game().controls(controls).build();
        Ok(())
    }
}

#[test]
fn reset_control_recreates_the_world_once_per_press() {
    let mut game = GameTestHarness::from_plugin(ResetPlugin).unwrap();
    let before = game.world().ids_with::<Player>()[0];

    game = game.press_action("reset");
    game.fixed_step(1.0 / 120.0);

    let after = game.world().ids_with::<Player>()[0];
    assert_ne!(before, after);
    assert!(
        format!("{after:?}").contains("generation: 1"),
        "one reset must advance the player entity generation exactly once; got {after:?}"
    );
}

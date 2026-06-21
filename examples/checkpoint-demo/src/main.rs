use game_starter::prelude::*;
fn main() -> Result<()> {
    run_game("Checkpoints", |game| {
        game.asset_bag()
            .texture("player", "textures/test.png")?
            .texture("checkpoint", "textures/test.png")?
            .texture("floor", "textures/test.png")?
            .texture("wall", "textures/test.png")?
            .build();
        let controls = game.input(|input| input.top_down_controls())?;
        game.player_prefab("player")
            .sprite("player")
            .moves_with(controls.movement, 130.0)
            .health(10)
            .build()?;
        game.checkpoint_prefab("checkpoint")
            .sprite("checkpoint")
            .size(vec2(32.0, 32.0))
            .build()?;
        game.map("checkpoints")
            .tiles(["########", "#P.C...#", "########"])
            .simple_theme("floor", "wall")
            .legend('P', "player")
            .legend('C', "checkpoint")
            .start();
        game.rules()
            .top_down_controls(controls)
            .player_activates_checkpoints()
            .respawn_at_checkpoint()
            .show_player_health()
            .build();
        Ok(())
    })
}

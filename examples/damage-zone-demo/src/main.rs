use game_starter::prelude::*;
fn main() -> Result<()> {
    run_game("Damage Zone", |game| {
        game.asset_bag()
            .texture("player", "textures/test.png")?
            .texture("zone", "textures/test.png")?
            .texture("floor", "textures/test.png")?
            .texture("wall", "textures/test.png")?
            .build();
        let controls = game.input(|input| input.top_down_controls())?;
        game.player_prefab("player")
            .sprite("player")
            .moves_with(controls.movement, 130.0)
            .health(100)
            .build()?;
        game.trigger_prefab("lava")
            .sprite("zone")
            .size(vec2(48.0, 48.0))
            .build()?;
        game.map("lava")
            .tiles(["########", "#P.L...#", "########"])
            .simple_theme("floor", "wall")
            .legend('P', "player")
            .legend('L', "lava")
            .start();
        game.rules()
            .top_down_controls(controls)
            .show_player_health()
            .build();
        game.on_collision("player", "lava", |event| {
            event.player().damage(1);
        });
        Ok(())
    })
}

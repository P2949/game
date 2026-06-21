use game_starter::prelude::*;
fn main() -> Result<()> {
    run_game("Status Panel", |game| {
        game.asset_bag()
            .texture("player", "textures/test.png")?
            .texture("floor", "textures/test.png")?
            .texture("wall", "textures/test.png")?
            .build();
        let controls = game.input(|input| input.top_down_controls())?;
        game.player_prefab("player")
            .sprite("player")
            .moves_with(controls.movement, 130.0)
            .health(100)
            .build()?;
        game.map("status")
            .tiles(["#####", "#P..#", "#####"])
            .simple_theme("floor", "wall")
            .legend('P', "player")
            .start();
        game.rules().top_down_controls(controls).build();
        game.draw_ui(|game, _dt| {
            game.ui()
                .status_panel()
                .score()
                .player_health()
                .enemy_count()
                .build();
        });
        Ok(())
    })
}

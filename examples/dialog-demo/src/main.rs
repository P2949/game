use game_starter::prelude::*;
fn main() -> Result<()> {
    run_game("Dialog", |game| {
        game.asset_bag()
            .texture("floor", "textures/test.png")?
            .texture("wall", "textures/test.png")?
            .build();
        let controls = game.input(|input| input.top_down_controls())?;
        game.map("dialog")
            .tiles(["..."])
            .simple_theme("floor", "wall")
            .start();
        game.rules().top_down_controls(controls).build();
        game.draw_ui(|game, _dt| {
            game.ui()
                .dialog("Old Man")
                .line("Welcome to the arena.")
                .line("Collect all coins!")
                .build();
        });
        Ok(())
    })
}

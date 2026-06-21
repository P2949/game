use game_starter::prelude::*;
fn main() -> Result<()> {
    run_game("Title Menu", |game| {
        game.asset_bag()
            .texture("floor", "textures/test.png")?
            .texture("wall", "textures/test.png")?
            .build();
        let controls = game.input(|input| input.top_down_controls())?;
        game.map("menu")
            .tiles(["..."])
            .simple_theme("floor", "wall")
            .start();
        game.start_scene("menu").scene("game");
        game.use_top_down_game().controls(controls).build();
        game.draw_ui(|game, _dt| {
            if game.current_scene_name().as_deref() == Some("menu") {
                game.ui()
                    .menu("My Game")
                    .button("Start")
                    .go_to_scene("game")
                    .button("Quit")
                    .quit()
                    .build();
            }
        });
        Ok(())
    })
}

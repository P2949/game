use game_starter::prelude::*;

fn main() -> Result<()> {
    run_game("Audio Demo", |game| {
        game.asset_bag()
            .texture("floor", "textures/test.png")?
            .texture("wall", "textures/test.png")?
            .generated_sound("coin")?
            .generated_sound("theme")?
            .generated_sound("battle")?
            .build();
        let controls = game.input(|input| input.top_down_controls())?;

        game.map("audio")
            .tiles([".....", "....."])
            .simple_theme("floor", "wall")
            .start();

        game.rules().top_down_controls(controls).build();

        game.on_action(controls.attack, |game| {
            game.audio().bus("ambience").volume(0.5);
            game.audio().play_sound("coin").bus("ambience");
            game.audio().play_music("theme").volume(0.4).fade_in(1.0);
        });
        game.on_action(controls.reset, |game| {
            game.audio().fade_music_to(0.0, 1.0);
        });
        game.on_action(controls.pause, |game| {
            game.audio().pause_music();
        });
        game.on_action(controls.debug_overlay, |game| {
            game.audio().crossfade_music("battle", 1.0);
        });

        game.draw_ui(|game, _dt| {
            game.ui()
                .panel("Audio Demo")
                .line("Space: sound + music")
                .line("R: fade out | P: pause | F1: crossfade")
                .center();
        });
        Ok(())
    })
}

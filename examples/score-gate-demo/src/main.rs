use game_starter::prelude::*;

fn main() -> Result<()> {
    run_game("Score Gate Demo", |game| {
        game.asset_bag()
            .texture("player", "textures/test.png")?
            .texture("coin", "textures/test.png")?
            .texture("floor", "textures/test.png")?
            .texture("wall", "textures/test.png")?
            .sound("coin", "sounds/hit.wav")?
            .build();

        let controls = game.input(|input| input.top_down_controls())?;

        game.player_prefab("player")
            .sprite("player")
            .moves_with(controls.movement, 130.0)
            .build()?;
        game.pickup_prefab("coin")
            .sprite("coin")
            .score(1)
            .play_sound("coin")
            .despawn_on_collect()
            .build()?;

        game.map("collect")
            .tiles(["########", "#P.C.C.#", "#..C...#", "########"])
            .simple_theme("floor", "wall")
            .legend('P', "player")
            .legend('C', "coin")
            .start();

        game.start_scene("playing").scene("win");
        game.rules()
            .top_down_controls(controls)
            .player_collects_pickups()
            .camera_follows_player()
            .show_score()
            .build();

        game.on_score_reaches(3, |game| {
            game.change_scene_or_log("win");
        });
        game.draw_ui(|game, _dt| {
            if game.current_scene_name().as_deref() == Some("win") {
                game.ui().panel("Gate Open").line("Score reached").center();
            }
        });

        Ok(())
    })
}

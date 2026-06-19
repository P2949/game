use game_starter::prelude::*;

fn main() -> Result<()> {
    run_game("Coin Collector", |game| {
        let assets = game
            .asset_bag()
            .texture("player", "textures/test.png")?
            .texture("coin", "textures/test.png")?
            .texture("floor", "textures/test.png")?
            .texture("wall", "textures/test.png")?
            .sound("coin", "sounds/hit.wav")?
            .build();
        let controls = game.input(|input| input.top_down_controls())?;

        game.player_prefab("player")
            .sprite(assets.texture("player"))
            .moves_with(controls.movement, 130.0)
            .build()?;

        game.pickup_prefab("coin")
            .sprite(assets.texture("coin"))
            .score(1)
            .play_sound(assets.sound("coin"))
            .despawn_on_collect()
            .build()?;

        game.map("coins")
            .tiles(["########", "#P.C..C#", "#..C...#", "########"])
            .simple_theme(assets.texture("floor"), assets.texture("wall"))
            .legend('P', "player")
            .legend('C', "coin")
            .start();

        game.rules()
            .top_down_controls(controls)
            .camera_follows_player()
            .show_basic_ui()
            .build();

        game.on_player_collect_pickup(|game: &mut Game<'_, '_>| {
            game.camera2d().shake(0.08);
        });

        game.draw_ui(|game: &mut Game<'_, '_>, _dt| {
            let score = game.score().value();
            game.text(
                &format!("Score: {score}"),
                vec2(24.0, 24.0),
                vec4(1.0, 0.95, 0.35, 1.0),
            );
        });

        Ok(())
    })
}

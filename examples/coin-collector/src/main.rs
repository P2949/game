use game_starter::prelude::*;

fn main() -> Result<()> {
    run_game("Coin Collector", |game| {
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

        game.map("coins")
            .tiles(["########", "#P.C..C#", "#..C...#", "########"])
            .simple_theme("floor", "wall")
            .legend('P', "player")
            .legend('C', "coin")
            .start();

        game.rules()
            .top_down_controls(controls)
            .camera_follows_player()
            .show_score()
            .build();

        game.on_player_collect_pickup(|game| {
            game.camera2d().shake(0.08);
        });

        Ok(())
    })
}

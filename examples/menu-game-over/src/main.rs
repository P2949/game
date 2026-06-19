use game_starter::prelude::*;

fn main() -> Result<()> {
    run_game("Menu And Game Over", |game| {
        let assets = game
            .asset_bag()
            .texture("player", "textures/test.png")?
            .texture("slime", "textures/test.png")?
            .texture("floor", "textures/test.png")?
            .texture("wall", "textures/test.png")?
            .build();
        let controls = game.input(|input| input.top_down_controls())?;

        game.player_prefab("player")
            .sprite(assets.texture("player"))
            .moves_with(controls.movement, 130.0)
            .health(1)
            .build()?;

        game.enemy_prefab("slime")
            .sprite(assets.texture("slime"))
            .chases_player()
            .melee(26.0, 1)
            .build()?;

        game.map("menu")
            .tiles(["..."])
            .simple_theme(assets.texture("floor"), assets.texture("wall"))
            .start();

        game.map("game")
            .tiles(["########", "#P.E...#", "#......#", "########"])
            .simple_theme(assets.texture("floor"), assets.texture("wall"))
            .legend('P', "player")
            .legend('E', "slime")
            .finish();

        game.map("game_over")
            .tiles(["..."])
            .simple_theme(assets.texture("floor"), assets.texture("wall"))
            .finish();

        game.use_simple_scene_flow()
            .menu("menu")
            .game("game")
            .game_over("game_over")
            .start_on(controls.attack)
            .restart_on(controls.reset)
            .build();

        game.on_scene_enter("game", |game: &mut Game<'_, '_>| {
            game.score().reset();
        });

        Ok(())
    })
}

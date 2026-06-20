use game_starter::prelude::*;

fn main() -> Result<()> {
    run_game("Menu And Game Over", |game| {
        game.asset_bag()
            .texture("player", "textures/test.png")?
            .texture("slime", "textures/test.png")?
            .texture("floor", "textures/test.png")?
            .texture("wall", "textures/test.png")?
            .build();
        let controls = game.input(|input| input.top_down_controls())?;

        game.player_prefab("player")
            .sprite("player")
            .moves_with(controls.movement, 130.0)
            .health(1)
            .build()?;

        game.enemy_prefab("slime")
            .sprite("slime")
            .chases_player()
            .melee(26.0, 1)
            .build()?;

        game.map("menu")
            .tiles(["..."])
            .simple_theme("floor", "wall")
            .start();

        game.map("game")
            .tiles(["########", "#P.E...#", "#......#", "########"])
            .simple_theme("floor", "wall")
            .legend('P', "player")
            .legend('E', "slime")
            .finish();

        game.map("game_over")
            .tiles(["..."])
            .simple_theme("floor", "wall")
            .finish();

        game.map("win")
            .tiles(["..."])
            .simple_theme("floor", "wall")
            .finish();

        game.use_simple_scene_flow()
            .menu("menu")
            .menu_text("Press Space to Start")
            .game("game")
            .game_over("game_over")
            .game_over_text("Game Over - Press R")
            .win("win")
            .win_text("You cleared the level!")
            .win_when_all_enemies_dead()
            .start_on(controls.attack)
            .restart_on(controls.reset)
            .build();

        game.on_scene_enter("game", |game| {
            game.score().reset();
        });

        Ok(())
    })
}

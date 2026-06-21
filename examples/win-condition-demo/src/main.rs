use game_starter::prelude::*;
fn main() -> Result<()> {
    run_game("Win Condition", |game| {
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
            .melee(30.0, 25)
            .build()?;
        game.enemy_prefab("slime")
            .sprite("slime")
            .health(20)
            .chases_player()
            .build()?;
        game.map("game")
            .tiles(["#######", "#P.E..#", "#######"])
            .simple_theme("floor", "wall")
            .legend('P', "player")
            .legend('E', "slime")
            .start();
        game.start_scene("game").scene("win");
        game.rules()
            .top_down_controls(controls)
            .enemies_damage_player()
            .dead_enemies_despawn()
            .win_when_all_enemies_dead()
            .show_win_panel()
            .build();
        Ok(())
    })
}

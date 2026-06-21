use game_starter::prelude::*;
fn main() -> Result<()> {
    run_game("Enemy Drops", |game| {
        game.asset_bag()
            .texture("player", "textures/test.png")?
            .texture("slime", "textures/test.png")?
            .texture("coin", "textures/test.png")?
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
            .drops("coin")
            .build()?;
        game.pickup_prefab("coin")
            .sprite("coin")
            .score(10)
            .despawn_on_collect()
            .build()?;
        game.map("drops")
            .tiles(["#######", "#P.E..#", "#######"])
            .simple_theme("floor", "wall")
            .legend('P', "player")
            .legend('E', "slime")
            .start();
        game.rules()
            .top_down_controls(controls)
            .enemies_damage_player()
            .enemy_drops()
            .dead_enemies_despawn()
            .player_collects_pickups()
            .show_score()
            .build();
        Ok(())
    })
}

use game_starter::prelude::*;
fn main() -> Result<()> {
    run_game("Health Pickup", |game| {
        game.asset_bag()
            .texture("player", "textures/test.png")?
            .texture("heart", "textures/test.png")?
            .texture("floor", "textures/test.png")?
            .texture("wall", "textures/test.png")?
            .build();
        let controls = game.input(|input| input.top_down_controls())?;
        game.player_prefab("player")
            .sprite("player")
            .moves_with(controls.movement, 130.0)
            .health(100)
            .build()?;
        game.pickup_prefab("heart")
            .sprite("heart")
            .heal_player(25)
            .despawn_on_collect()
            .build()?;
        game.map("hearts")
            .tiles(["#######", "#P.H..#", "#######"])
            .simple_theme("floor", "wall")
            .legend('P', "player")
            .legend('H', "heart")
            .start();
        game.rules()
            .top_down_controls(controls)
            .player_collects_pickups()
            .show_player_health()
            .build();
        Ok(())
    })
}

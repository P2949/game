use game_starter::prelude::*;
fn main() -> Result<()> {
    run_game("Boss", |game| {
        game.asset_bag()
            .texture("player", "textures/test.png")?
            .texture("boss", "textures/test.png")?
            .texture("floor", "textures/test.png")?
            .texture("wall", "textures/test.png")?
            .build();
        let controls = game.input(|input| input.top_down_controls())?;
        game.player_prefab("player")
            .sprite("player")
            .moves_with(controls.movement, 130.0)
            .melee(34.0, 25)
            .build()?;
        game.enemy_prefab("boss")
            .sprite("boss")
            .size(44.0)
            .health(300)
            .speed(55.0)
            .chases_player()
            .melee(42.0, 12)
            .build()?;
        game.map("boss")
            .tiles(["#########", "#P....B.#", "#########"])
            .simple_theme("floor", "wall")
            .legend('P', "player")
            .legend('B', "boss")
            .start();
        game.rules()
            .top_down_controls(controls)
            .enemies_damage_player()
            .camera_follows_player()
            .show_player_health()
            .show_enemy_count()
            .build();
        Ok(())
    })
}

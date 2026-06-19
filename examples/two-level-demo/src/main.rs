use game_starter::prelude::*;

fn main() -> Result<()> {
    run_game("Two Levels", |game| {
        let assets = game
            .asset_bag()
            .texture("player", "textures/test.png")?
            .texture("slime", "textures/test.png")?
            .texture("door", "textures/test.png")?
            .texture("floor", "textures/test.png")?
            .texture("wall", "textures/test.png")?
            .sound("hit", "sounds/hit.wav")?
            .build();
        let controls = game.input(|input| input.top_down_controls())?;

        game.player_prefab("player")
            .sprite(assets.texture("player"))
            .moves_with(controls.movement, 130.0)
            .melee(30.0, 25)
            .build()?;

        game.enemy_prefab("slime")
            .sprite(assets.texture("slime"))
            .health(25)
            .chases_player()
            .build()?;

        game.door_prefab("exit")
            .sprite(assets.texture("door"))
            .change_map("level_2")
            .requires_all_enemies_dead()
            .build()?;

        game.door_prefab("restart")
            .sprite(assets.texture("door"))
            .restart_level()
            .build()?;

        game.map("level_1")
            .tiles(["########", "#P.E..D#", "#......#", "########"])
            .simple_theme(assets.texture("floor"), assets.texture("wall"))
            .legend('P', "player")
            .legend('E', "slime")
            .legend('D', "exit")
            .start();

        game.map("level_2")
            .tiles(["########", "#P....R#", "#......#", "########"])
            .simple_theme(assets.texture("floor"), assets.texture("wall"))
            .legend('P', "player")
            .legend('R', "restart")
            .finish();

        game.rules()
            .top_down_controls(controls)
            .enemies_damage_player()
            .dead_enemies_despawn()
            .doors_change_maps()
            .camera_follows_player()
            .build();

        Ok(())
    })
}

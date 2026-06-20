use game_starter::prelude::*;

fn main() -> Result<()> {
    run_game("Enemy Waves", |game| {
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

        game.spawner_prefab("spawner")
            .spawn("slime")
            .every_seconds(2.0)
            .max_alive(4)
            .near_player(96.0)
            .build()?;

        game.map("waves")
            .tiles(["########", "#P....S#", "#......#", "########"])
            .simple_theme("floor", "wall")
            .legend('P', "player")
            .legend('S', "spawner")
            .start();

        game.rules()
            .top_down_controls(controls)
            .spawners_spawn_prefabs()
            .enemies_damage_player()
            .dead_enemies_despawn()
            .camera_follows_player()
            .show_score()
            .show_enemy_count()
            .build();

        Ok(())
    })
}

use game_starter::prelude::*;

fn main() -> Result<()> {
    run_game("Enemy Waves", |game| {
        let assets = game
            .asset_bag()
            .texture("player", "textures/test.png")?
            .texture("slime", "textures/test.png")?
            .texture("spawner", "textures/test.png")?
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
            .health(20)
            .chases_player()
            .build()?;

        game.spawner_prefab("spawner")
            .spawn("slime")
            .every_seconds(2.0)
            .max_alive(4)
            .build()?;

        game.map("waves")
            .tiles(["########", "#P....S#", "#......#", "########"])
            .simple_theme(assets.texture("floor"), assets.texture("wall"))
            .legend('P', "player")
            .legend('S', "spawner")
            .start();

        game.rules()
            .top_down_controls(controls)
            .enemies_damage_player()
            .dead_enemies_despawn()
            .camera_follows_player()
            .show_basic_ui()
            .build();

        game.every_seconds(2.0, |game: &mut Game<'_, '_>| {
            if game.enemies().alive().count() < 4 {
                game.spawn("slime").near_player(96.0);
            }
        });

        game.draw_ui(|game: &mut Game<'_, '_>, _dt| {
            let alive = game.enemies().alive().count();
            game.text(
                &format!("Enemies: {alive}/4"),
                vec2(24.0, 24.0),
                vec4(1.0, 0.95, 0.75, 1.0),
            );
        });

        Ok(())
    })
}

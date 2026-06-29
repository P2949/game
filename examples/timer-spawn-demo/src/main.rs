use game_starter::prelude::*;

fn main() -> Result<()> {
    run_game("Timer Spawn Demo", |game| {
        game.asset_bag()
            .texture("player", "textures/test.png")?
            .texture("slime", "textures/test.png")?
            .texture("floor", "textures/test.png")?
            .texture("wall", "textures/test.png")?
            .sound("spawn", "sounds/hit.wav")?
            .build();

        let controls = game.input(|input| input.top_down_controls())?;

        game.player_prefab("player")
            .sprite("player")
            .moves_with(controls.movement, 130.0)
            .health(60)
            .melee(32.0, 20)
            .build()?;
        game.enemy_prefab("slime")
            .sprite("slime")
            .health(20)
            .chases_player()
            .build()?;

        game.map("arena")
            .tiles(["########", "#P.....#", "#......#", "########"])
            .simple_theme("floor", "wall")
            .legend('P', "player")
            .start();

        game.rules()
            .top_down_controls(controls)
            .enemies_damage_player()
            .dead_enemies_despawn()
            .camera_follows_player()
            .show_score()
            .show_enemy_count()
            .show_player_health()
            .build();

        game.on_timer("first_wave", 1.0, |game| {
            game.spawn("slime").near_player(96.0);
            game.play_sound_named("spawn");
        });
        game.every_seconds_while_playing(3.0, |game| {
            game.spawn("slime").near_player(128.0);
            game.score().add(1);
        });
        game.on_wave_cleared(|game| {
            game.score().add(10);
        });

        Ok(())
    })
}

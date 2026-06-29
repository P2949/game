use game_starter::prelude::*;

fn main() -> Result<()> {
    run_game("Projectile Hit Demo", |game| {
        game.asset_bag()
            .texture("player", "textures/test.png")?
            .texture("slime", "textures/test.png")?
            .texture("bolt", "textures/test.png")?
            .texture("spark", "textures/test.png")?
            .texture("floor", "textures/test.png")?
            .texture("wall", "textures/test.png")?
            .sound("shoot", "sounds/hit.wav")?
            .sound("hit", "sounds/hit.wav")?
            .build();

        let controls = game.input(|input| input.top_down_controls())?;

        game.player_prefab("player")
            .sprite("player")
            .moves_with(controls.movement, 130.0)
            .build()?;
        game.enemy_prefab("slime")
            .sprite("slime")
            .health(30)
            .build()?;
        game.projectile_prefab("bolt")
            .sprite("bolt")
            .damage(10)
            .speed(280.0)
            .lifetime(0.8)
            .despawn_on_hit()
            .build()?;
        game.pickup_prefab("spark")
            .sprite("spark")
            .score(0)
            .despawn_on_collect()
            .build()?;

        game.map("range")
            .tiles(["########", "#P..E..#", "#......#", "########"])
            .simple_theme("floor", "wall")
            .legend('P', "player")
            .legend('E', "slime")
            .start();

        game.rules()
            .top_down_controls(controls)
            .projectiles()
            .dead_enemies_despawn()
            .camera_follows_player()
            .show_score()
            .show_enemy_count()
            .build();

        game.on_action_cooldown(controls.attack, 0.2, |game| {
            game.player().shoot("bolt").towards_mouse();
            game.play_sound_named("shoot");
        });
        game.on_projectile_hit("bolt", "slime", |event| {
            let position = event.position();
            event.score().add(2);
            event.play_sound("hit");
            event.enemy().set_tag("marked");
            event.spawn("spark").at_world(position);
        });

        Ok(())
    })
}

use game_starter::prelude::*;

fn main() -> Result<()> {
    run_game("Projectile Demo", |game| {
        let assets = game
            .asset_bag()
            .texture("player", "textures/test.png")?
            .texture("slime", "textures/test.png")?
            .texture("bolt", "textures/test.png")?
            .texture("floor", "textures/test.png")?
            .texture("wall", "textures/test.png")?
            .sound("hit", "sounds/hit.wav")?
            .build();
        let controls = game.input(|input| input.top_down_controls())?;

        game.player_prefab("player")
            .sprite(assets.texture("player"))
            .moves_with(controls.movement, 130.0)
            .build()?;

        game.enemy_prefab("slime")
            .sprite(assets.texture("slime"))
            .health(30)
            .build()?;

        game.projectile_prefab("bolt")
            .sprite(assets.texture("bolt"))
            .damage(15)
            .speed(260.0)
            .lifetime(0.8)
            .despawn_on_hit()
            .build()?;

        game.map("projectiles")
            .tiles(["########", "#P..E..#", "#......#", "########"])
            .simple_theme(assets.texture("floor"), assets.texture("wall"))
            .legend('P', "player")
            .legend('E', "slime")
            .start();

        game.rules()
            .top_down_controls(controls)
            .dead_enemies_despawn()
            .camera_follows_player()
            .build();

        game.on_action_cooldown(controls.attack, 0.2, move |game: &mut Game<'_, '_>| {
            game.spawn("bolt").near_player(28.0);
            if game.enemies().alive().near_player(96.0).damage(15) > 0 {
                game.commands().play_sound(assets.sound("hit"));
            }
        });

        Ok(())
    })
}

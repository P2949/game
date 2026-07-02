use game_starter::prelude::*;

fn main() -> Result<()> {
    run_game("Animation Demo", |game| {
        let assets = game
            .asset_bag()
            .spritesheet_from_meta("player", "animations/player.toml")?
            .spritesheet_from_meta("slime", "animations/slime.toml")?
            .spritesheet_from_meta("bolt", "animations/bolt.toml")?
            .texture("floor", "textures/test.png")?
            .texture("wall", "textures/test.png")?
            .build();
        let controls = game.input(|input| input.top_down_controls())?;

        game.player_prefab("player")
            .animation_sheet(assets.animation_sheet("player"))
            .moves_with(controls.movement, 130.0)
            .build()?;

        game.enemy_prefab("slime")
            .animation_sheet(assets.animation_sheet("slime"))
            .health(30)
            .chases_player()
            .build()?;

        game.projectile_prefab("bolt")
            .animation_sheet(assets.animation_sheet("bolt"))
            .speed(260.0)
            .damage(15)
            .lifetime(0.8)
            .despawn_on_hit()
            .build()?;

        game.map("animation")
            .tiles(["########", "#P..E..#", "#......#", "########"])
            .simple_theme("floor", "wall")
            .legend('P', "player")
            .legend('E', "slime")
            .start();

        game.rules()
            .top_down_controls(controls)
            .camera_follows_player()
            .enemies_damage_player()
            .animate_player_directionally()
            .animate_enemies_directionally()
            .animate_attacks_directionally()
            .projectiles()
            .projectile_impact_animation_before_despawn()
            .build();

        game.on_action(controls.attack, |game| {
            game.player().shoot("bolt").towards_mouse();
        });

        Ok(())
    })
}

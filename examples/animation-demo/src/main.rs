use game_starter::prelude::*;

fn main() -> Result<()> {
    run_game("Animation Demo", |game| {
        let assets = game
            .asset_bag()
            .spritesheet("player", "textures/test.png", 4, 1)?
            .spritesheet("slime", "textures/test.png", 4, 1)?
            .spritesheet("bolt", "textures/test.png", 4, 1)?
            .texture("floor", "textures/test.png")?
            .texture("wall", "textures/test.png")?
            .build();
        let controls = game.input(|input| input.top_down_controls())?;

        game.player_prefab("player")
            .spritesheet(assets.spritesheet("player"))
            .idle(0..1)
            .walk_up(0..1)
            .walk_down(1..2)
            .walk_left(2..3)
            .walk_right(3..4)
            .moves_with(controls.movement, 130.0)
            .build()?;

        game.enemy_prefab("slime")
            .spritesheet(assets.spritesheet("slime"))
            .idle(0..1)
            .walk_up(0..1)
            .walk_down(1..2)
            .walk_left(2..3)
            .walk_right(3..4)
            .health(30)
            .chases_player()
            .build()?;

        game.projectile_prefab("bolt")
            .spritesheet(assets.spritesheet("bolt"))
            .flight(0..2)
            .impact(2..4)
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
            .projectiles()
            .projectile_impact_animation_before_despawn()
            .build();

        game.on_action(controls.attack, |game| {
            game.player().shoot("bolt").towards_mouse();
        });

        Ok(())
    })
}

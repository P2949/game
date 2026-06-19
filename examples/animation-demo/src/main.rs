use game_starter::prelude::*;

fn main() -> Result<()> {
    run_game("Animation Demo", |game| {
        let assets = game
            .asset_bag()
            .spritesheet("player", "textures/test.png", 4, 1)?
            .texture("slime", "textures/test.png")?
            .texture("floor", "textures/test.png")?
            .texture("wall", "textures/test.png")?
            .build();
        let controls = game.input(|input| input.top_down_controls())?;

        game.player_prefab("player")
            .spritesheet(assets.spritesheet("player"))
            .idle(0..1)
            .walk(1..3)
            .attack(3..4)
            .moves_with(controls.movement, 130.0)
            .melee(30.0, 25)
            .build()?;

        game.enemy_prefab("slime")
            .sprite(assets.texture("slime"))
            .health(30)
            .build()?;

        game.map("animation")
            .tiles(["########", "#P..E..#", "#......#", "########"])
            .simple_theme(assets.texture("floor"), assets.texture("wall"))
            .legend('P', "player")
            .legend('E', "slime")
            .start();

        game.use_top_down_game()
            .controls(controls)
            .with_melee_combat()
            .with_player_animation_by_movement()
            .with_attack_animation("attack")
            .with_camera_follow()
            .build();

        Ok(())
    })
}

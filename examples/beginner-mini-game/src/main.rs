use game_starter::prelude::*;

fn main() -> Result<()> {
    run_game("Beginner Mini Game", |game| {
        let assets = game
            .asset_bag()
            .spritesheet("player", "textures/test.png", 4, 1)?
            .texture("slime", "textures/test.png")?
            .texture("coin", "textures/test.png")?
            .texture("door", "textures/test.png")?
            .texture("floor", "textures/test.png")?
            .texture("wall", "textures/test.png")?
            .sound("coin", "sounds/hit.wav")?
            .sound("hit", "sounds/hit.wav")?
            .build();

        let controls = game.input(|input| input.top_down_controls())?;

        game.player_prefab("player")
            .spritesheet(assets.spritesheet("player"))
            .idle(0..1)
            .walk(1..3)
            .attack(3..4)
            .moves_with(controls.movement, 130.0)
            .health(60)
            .melee(30.0, 25)
            .build()?;

        game.enemy_prefab("slime")
            .sprite("slime")
            .health(30)
            .chases_player()
            .melee(26.0, 8)
            .build()?;

        game.pickup_prefab("coin")
            .sprite("coin")
            .score(1)
            .play_sound("coin")
            .despawn_on_collect()
            .build()?;

        game.door_prefab("exit")
            .sprite("door")
            .change_map("level_2")
            .requires_all_enemies_dead()
            .build()?;

        game.door_prefab("restart")
            .sprite("door")
            .restart_level()
            .build()?;

        game.map("level_1")
            .tiles(["##########", "#P.C..E.D#", "#..C.....#", "##########"])
            .simple_theme("floor", "wall")
            .legend('P', "player")
            .legend('C', "coin")
            .legend('E', "slime")
            .legend('D', "exit")
            .start();

        game.map("level_2")
            .tiles(["##########", "#P.C.E..R#", "#..C..C..#", "##########"])
            .simple_theme("floor", "wall")
            .legend('P', "player")
            .legend('C', "coin")
            .legend('E', "slime")
            .legend('R', "restart")
            .finish();

        game.use_top_down_game()
            .controls(controls)
            .hit_sound_named("hit")
            .with_melee_combat()
            .with_enemy_chase()
            .with_collision()
            .with_camera_follow()
            .with_pause_death_ui()
            .with_player_animation_by_movement()
            .with_attack_animation("attack")
            .build();

        game.rules()
            .player_collects_pickups()
            .doors_change_maps()
            .show_score()
            .show_player_health()
            .show_pause_menu()
            .show_game_over_panel()
            .build();

        Ok(())
    })
}

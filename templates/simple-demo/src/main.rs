use game_starter::prelude::*;

fn main() -> Result<()> {
    run_game("{{title}}", |game| {
        // 1. Register files by name. You use these names later in
        // `.sprite("player")` and `.simple_theme("floor", "wall")`.
        game.assets_from_folders()
            .required_textures(["player", "slime", "coin", "floor", "wall", "door", "bolt"])?
            .required_sounds(["hit", "coin", "shoot"])?
            .build();

        let controls = game.input(|input| input.top_down_controls())?;

        // 2. Define the game objects that map symbols can spawn.
        game.player_prefab("player")
            .sprite("player")
            .moves_with(controls.movement, 130.0)
            .health(100)
            .melee(30.0, 25)
            .build()?;

        game.enemy_prefab("slime")
            .sprite("slime")
            .chases_player()
            .health(40)
            .melee(26.0, 6)
            .build()?;

        game.pickup_prefab("coin")
            .sprite("coin")
            .score(1)
            .play_sound("coin")
            .despawn_on_collect()
            .build()?;

        // 3. Load the editable text map from assets/maps/level_1.txt.
        game.map_from_text_auto("level_1")
            .simple_theme("floor", "wall")
            .legend('P', "player")
            .legend('E', "slime")
            .legend('C', "coin")
            .start();

        game.use_top_down_game()
            .controls(controls)
            .with_melee_combat()
            .with_enemy_chase()
            .with_collision()
            .with_camera_follow()
            .with_pause_death_ui()
            .build();

        game.rules().player_collects_pickups().show_score().build();

        Ok(())
    })
}

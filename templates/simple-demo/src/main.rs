use game_starter::prelude::*;

fn main() -> Result<()> {
    run_game("{{title}}", |game| {
        game.assets_from_folders()
            .required_textures(["player", "slime", "floor", "wall"])?
            .required_sounds(["hit"])?
            .build();

        let controls = game.input(|input| input.top_down_controls())?;

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

        game.map_from_text_auto("level_1")
            .simple_theme("floor", "wall")
            .legend('P', "player")
            .legend('E', "slime")
            .start();

        game.use_top_down_game()
            .controls(controls)
            .hit_sound_named("hit")
            .with_melee_combat()
            .with_enemy_chase()
            .with_collision()
            .with_camera_follow()
            .with_pause_death_ui()
            .build();

        Ok(())
    })
}

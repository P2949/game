use game_starter::prelude::*;

fn main() -> Result<()> {
    run_game("{{title}}", |game| {
        let assets = game
            .asset_bag()
            .texture_auto("player")?
            .texture_auto("slime")?
            .texture_auto("floor")?
            .texture_auto("wall")?
            .sound_auto("hit")?
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

        game.map("level_1")
            .tiles(["########", "#......#", "#..P.E.#", "#......#", "########"])
            .simple_theme("floor", "wall")
            .legend('P', "player")
            .legend('E', "slime")
            .start();

        game.use_top_down_game()
            .controls(controls)
            .hit_sound(assets.sound("hit"))
            .with_melee_combat()
            .with_enemy_chase()
            .with_collision()
            .with_camera_follow()
            .with_pause_death_ui()
            .build();

        Ok(())
    })
}

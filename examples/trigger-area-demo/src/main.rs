use game_starter::prelude::*;

fn main() -> Result<()> {
    run_game("Trigger Areas", |game| {
        game.asset_bag()
            .texture("player", "textures/test.png")?
            .texture("debug_trigger", "textures/test.png")?
            .texture("floor", "textures/test.png")?
            .texture("wall", "textures/test.png")?
            .generated_sound("hurt")?
            .generated_sound("warning")?
            .build();
        let controls = game.input(|input| input.top_down_controls())?;

        game.player_prefab("player")
            .sprite("player")
            .moves_with(controls.movement, 130.0)
            .health(100)
            .build()?;
        game.trigger_prefab("danger_zone")
            .size(vec2(64.0, 64.0))
            .visible_debug("debug_trigger")
            .tint(vec4(1.0, 0.15, 0.15, 0.35))
            .build()?;

        game.map("areas")
            .tiles(["#########", "#P...D..#", "#.......#", "#########"])
            .simple_theme("floor", "wall")
            .legend('P', "player")
            .legend('D', "danger_zone")
            .start();

        game.rules()
            .top_down_controls(controls)
            .camera_follows_player()
            .show_player_health()
            .build();
        game.on_start(|game| game.spawn_start_map());

        game.on_enter_area("player", "danger_zone", |event| {
            event.actor().damage(10);
            event.play_sound("hurt");
        });
        game.on_exit_area("player", "danger_zone", |event| {
            event.play_sound("warning");
        });

        Ok(())
    })
}

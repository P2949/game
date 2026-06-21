use game_starter::prelude::*;

fn main() -> Result<()> {
    run_game("No Rust Shapes", |game| {
        game.assets_from_folders()
            .textures(["test"])?
            .sounds(["hit"])?
            .build();
        game.asset_bag().music("theme", "sounds/hit.wav")?.build();

        let controls = game.input(|input| input.top_down_controls())?;

        game.player_prefab("player")
            .sprite("test")
            .moves_with(controls.movement, 130.0)
            .health(100)
            .build()?;
        game.enemy_prefab("slime")
            .sprite("test")
            .health(30)
            .chases_player()
            .melee(26.0, 8)
            .drops("coin")
            .build()?;
        game.pickup_prefab("coin")
            .sprite("test")
            .score(1)
            .play_sound("hit")
            .despawn_on_collect()
            .build()?;
        game.pickup_prefab("heart")
            .sprite("test")
            .heal_player(25)
            .despawn_on_collect()
            .build()?;
        game.projectile_prefab("bolt")
            .sprite("test")
            .damage(15)
            .speed(260.0)
            .lifetime(0.8)
            .despawn_on_hit()
            .build()?;
        game.trigger_prefab("danger")
            .size(vec2(48.0, 48.0))
            .visible_debug("test")
            .build()?;
        game.checkpoint_prefab("checkpoint")
            .sprite("test")
            .build()?;
        game.door_prefab("restart")
            .sprite("test")
            .restart_level()
            .build()?;

        game.map("menu")
            .tiles(["..."])
            .simple_theme("test", "test")
            .start();
        game.map_from_text_auto("game")
            .simple_theme("test", "test")
            .legend('P', "player")
            .legend('E', "slime")
            .legend('C', "coin")
            .legend('H', "heart")
            .legend('D', "danger")
            .legend('K', "checkpoint")
            .legend('X', "restart")
            .finish();
        game.map("game_over")
            .tiles(["..."])
            .simple_theme("test", "test")
            .finish();
        game.map("win")
            .tiles(["..."])
            .simple_theme("test", "test")
            .finish();

        game.use_simple_scene_flow()
            .menu("menu")
            .menu_title("No Rust Shapes")
            .menu_button("Start", "game")
            .game("game")
            .game_over("game_over")
            .game_over_button("Try Again")
            .win("win")
            .win_text("You cleared the map!")
            .win_button("Play Again")
            .win_when_all_enemies_dead()
            .start_on(controls.attack)
            .restart_on(controls.reset)
            .build();

        game.rules()
            .top_down_controls(controls)
            .player_collects_pickups()
            .enemies_damage_player()
            .enemy_drops()
            .projectiles()
            .player_activates_checkpoints()
            .respawn_at_checkpoint()
            .camera_follows_player()
            .show_basic_ui()
            .show_player_health()
            .build();

        game.on_scene_enter("game", |game| {
            game.audio().play_music("theme").volume(0.2).fade_in(0.5);
        });
        game.on_action_cooldown(controls.attack, 0.2, |game| {
            game.player().shoot("bolt").towards_mouse();
            game.audio().play_sound("hit");
        });
        game.on_enter_area("player", "danger", |event| {
            event.actor().damage(5);
            event.play_sound("hit");
        });

        Ok(())
    })
}

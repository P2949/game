use game_starter::prelude::*;

fn main() -> Result<()> {
    run_game("Script-like Custom Rules", |game| {
        game.asset_bag()
            .texture("player", "textures/test.png")?
            .texture("slime", "textures/test.png")?
            .texture("coin", "textures/test.png")?
            .texture("bolt", "textures/test.png")?
            .texture("door", "textures/test.png")?
            .texture("floor", "textures/test.png")?
            .texture("wall", "textures/test.png")?
            .sound("coin", "sounds/hit.wav")?
            .sound("shoot", "sounds/hit.wav")?
            .build();

        let controls = game.input(|input| input.top_down_controls())?;

        game.player_prefab("player")
            .sprite("player")
            .moves_with(controls.movement, 130.0)
            .health(100)
            .melee(30.0, 15)
            .build()?;

        game.enemy_prefab("slime")
            .sprite("slime")
            .chases_player()
            .tag("enemy")
            .health(30)
            .melee(26.0, 6)
            .build()?;

        // A custom actor needs no new component type: tags and named data are
        // enough to make this enemy explode after its short fuse burns down.
        game.enemy_prefab("bomber")
            .sprite("slime")
            .chases_player()
            .tag("enemy")
            .tag("explosive")
            .data("fuse", 3.0)
            .health(30)
            .melee(26.0, 6)
            .build()?;

        game.projectile_prefab("bolt")
            .sprite("bolt")
            .damage(15)
            .speed(260.0)
            .lifetime(0.8)
            .despawn_on_hit()
            .build()?;

        game.pickup_prefab("coin")
            .sprite("coin")
            .score(1)
            .play_sound("coin")
            .despawn_on_collect()
            .build()?;

        game.spawner_prefab("spawner")
            .spawn("slime")
            .every_seconds(2.0)
            .max_alive(4)
            .near_player(96.0)
            .build()?;

        game.door_prefab("exit")
            .sprite("door")
            .change_map("level_2")
            .build()?;

        game.map("level_1")
            .tiles(["##########", "#P.C..S.D#", "#........#", "##########"])
            .simple_theme("floor", "wall")
            .legend('P', "player")
            .legend('C', "coin")
            .legend('S', "spawner")
            .legend('D', "exit")
            .start();

        game.map("level_2")
            .tiles(["##########", "#P.BE.C..#", "#........#", "##########"])
            .simple_theme("floor", "wall")
            .legend('P', "player")
            .legend('B', "bomber")
            .legend('E', "slime")
            .legend('C', "coin")
            .finish();

        game.rules()
            .top_down_controls(controls)
            .player_collects_pickups()
            .projectiles()
            .spawners_spawn_prefabs()
            .enemies_damage_player()
            .doors_change_maps()
            .dead_enemies_despawn()
            .camera_follows_player()
            .pause_and_reset()
            .show_score()
            .show_player_health()
            .show_pause_menu()
            .show_game_over_panel()
            .build();

        game.on_action_cooldown(controls.attack, 0.2, |game| {
            game.player().shoot("bolt").towards_mouse();
            game.play_sound_named("shoot");
        });

        game.custom_rule("explosive fuse")
            .each_tag("explosive")
            .countdown("fuse")
            .when_zero()
            .damage_tag("enemy", 20, 48.0)
            .damage_player(20, 48.0)
            .despawn_self()
            .build();

        Ok(())
    })
}

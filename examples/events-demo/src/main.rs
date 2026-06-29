use game_starter::prelude::*;

fn main() -> Result<()> {
    run_game("Events Demo", |game| {
        game.asset_bag()
            .texture("player", "textures/test.png")?
            .texture("slime", "textures/test.png")?
            .texture("coin", "textures/test.png")?
            .texture("door", "textures/test.png")?
            .texture("floor", "textures/test.png")?
            .texture("wall", "textures/test.png")?
            .sound("chime", "sounds/hit.wav")?
            .build();

        let controls = game.input(|input| input.top_down_controls())?;

        game.player_prefab("player")
            .sprite("player")
            .moves_with(controls.movement, 130.0)
            .health(30)
            .melee(30.0, 10)
            .build()?;
        game.enemy_prefab("slime")
            .sprite("slime")
            .health(10)
            .chases_player()
            .build()?;
        game.pickup_prefab("coin")
            .sprite("coin")
            .score(1)
            .despawn_on_collect()
            .build()?;
        game.door_prefab("exit")
            .sprite("door")
            .change_map("bonus")
            .requires_all_enemies_dead()
            .build()?;

        game.map("field")
            .tiles(["########", "#P.E.CD#", "#......#", "########"])
            .simple_theme("floor", "wall")
            .legend('P', "player")
            .legend('E', "slime")
            .legend('C', "coin")
            .legend('D', "exit")
            .start();
        game.map("bonus")
            .tiles(["#####", "#P.C#", "#####"])
            .simple_theme("floor", "wall")
            .legend('P', "player")
            .legend('C', "coin")
            .finish();

        game.rules()
            .top_down_controls(controls)
            .player_collects_pickups()
            .enemies_damage_player()
            .dead_enemies_despawn()
            .doors_change_maps()
            .camera_follows_player()
            .show_score()
            .show_player_health()
            .show_enemy_count()
            .build();

        game.on_collect("player", "coin", |event| {
            event.score().add(4);
            event.play_sound("chime");
        });
        game.on_enemy_death_event(|event| {
            if let Some(position) = event.enemy_position() {
                event.spawn("coin").at_world(position);
            }
            event.score().add(5);
        });
        game.on_door_open("exit", |event| {
            event.play_sound("chime");
            event.score().add(10);
        });
        game.on_map_changed(|event| {
            event.score().add(20);
            event.play_sound("chime");
        });

        Ok(())
    })
}

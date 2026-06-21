# 07 - Add Doors And Levels

## Goal

Connect two maps with a door that opens only after all enemies are gone.

## Files to edit

Edit `src/main.rs`.

## Full code

```rust
use game_starter::prelude::*;

fn main() -> Result<()> {
    run_game("Two Levels", |game| {
        game.asset_bag()
            .texture("player", "textures/test.png")?
            .texture("slime", "textures/test.png")?
            .texture("door", "textures/test.png")?
            .texture("floor", "textures/test.png")?
            .texture("wall", "textures/test.png")?
            .build();

        let controls = game.input(|input| input.top_down_controls())?;

        game.player_prefab("player")
            .sprite("player")
            .moves_with(controls.movement, 130.0)
            .melee(30.0, 25)
            .build()?;

        game.enemy_prefab("slime")
            .sprite("slime")
            .health(25)
            .chases_player()
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
            .tiles(["########", "#P.E..D#", "#......#", "########"])
            .simple_theme("floor", "wall")
            .legend('P', "player")
            .legend('E', "slime")
            .legend('D', "exit")
            .start();

        game.map("level_2")
            .tiles(["########", "#P....R#", "#......#", "########"])
            .simple_theme("floor", "wall")
            .legend('P', "player")
            .legend('R', "restart")
            .finish();

        game.rules()
            .top_down_controls(controls)
            .enemies_damage_player()
            .dead_enemies_despawn()
            .doors_change_maps()
            .camera_follows_player()
            .build();

        Ok(())
    })
}
```

## What changed

`.finish()` registers another map without making it the startup map. The `D`
door changes to `level_2`, but its lock checks whether living enemies remain.
The `R` door demonstrates a restart target. `doors_change_maps()` owns the
touch detection and transition work.

## Common errors

If a door says its target map is unknown, check the exact name passed to
`.change_map`. If the exit never opens, make sure dead enemies are removed with
`.dead_enemies_despawn()`. If a level starts unexpectedly, only the first map
should use `.start()`.

## Next step

Add sound effects and music in
[08 - Add sound and music](08-add-sound-and-music.md).

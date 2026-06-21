# 09 - Add UI And Menu

## Goal

Show score and health, then add a title screen and restart screen with both
keyboard/controller controls and clickable buttons.

## Files to edit

Edit `src/main.rs`.

## Full code

```rust
use game_starter::prelude::*;

fn main() -> Result<()> {
    run_game("Menu And Game Over", |game| {
        game.asset_bag()
            .texture("player", "textures/test.png")?
            .texture("slime", "textures/test.png")?
            .texture("floor", "textures/test.png")?
            .texture("wall", "textures/test.png")?
            .build();

        let controls = game.input(|input| input.top_down_controls())?;

        game.player_prefab("player")
            .sprite("player")
            .moves_with(controls.movement, 130.0)
            .health(1)
            .build()?;

        game.enemy_prefab("slime")
            .sprite("slime")
            .chases_player()
            .melee(26.0, 1)
            .build()?;

        game.map("menu")
            .tiles(["..."])
            .simple_theme("floor", "wall")
            .start();
        game.map("game")
            .tiles(["########", "#P.E...#", "#......#", "########"])
            .simple_theme("floor", "wall")
            .legend('P', "player")
            .legend('E', "slime")
            .finish();
        game.map("game_over")
            .tiles(["..."])
            .simple_theme("floor", "wall")
            .finish();

        game.use_simple_scene_flow()
            .menu("menu")
            .menu_text("Press Space to Start")
            .menu_button("Start", "game")
            .game("game")
            .game_over("game_over")
            .game_over_text("Game Over - Press R")
            .game_over_button("Restart")
            .start_on(controls.attack)
            .restart_on(controls.reset)
            .build();

        game.rules()
            .top_down_controls(controls)
            .enemies_damage_player()
            .camera_follows_player()
            .show_score()
            .show_player_health()
            .show_game_over_panel()
            .build();

        Ok(())
    })
}
```

## What changed

`show_score` and `show_player_health` are HUD helpers. `use_simple_scene_flow`
describes the named scenes and changes maps as the player starts, dies, or
restarts. `menu_button` and `game_over_button` draw their own rectangles and
mouse hitboxes—there is no coordinate math in your game code. Space/Enter and
R remain available for keyboard and gamepad play.

## Common errors

If clicking Start does nothing, make sure its second argument matches the game
map name. If a blank screen appears after a scene change, check that the scene
and map have both been registered. If score or health is missing during play,
include the matching rule helper.

## Next step

Ship the result in [10 - Package your demo](10-package-your-demo.md).

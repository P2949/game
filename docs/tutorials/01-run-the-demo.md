# 01 - Run The Demo

## Goal

Run the game binary, select a content demo, and know where the demo code and
assets live.

## What you will build

Nothing yet. This first step proves the workspace, runtime, renderer, assets, and
selected content crate all start together.

## Files to edit

None.

## Full code

`examples/one-file-demo/src/main.rs` is the complete working version you can
run while following this course:

```rust
use game_starter::prelude::*;

fn main() -> Result<()> {
    run_game("My First Game", |game| {
        game.asset_bag()
            .texture("player", "textures/test.png")?
            .texture("slime", "textures/test.png")?
            .texture("floor", "textures/test.png")?
            .texture("wall", "textures/test.png")?
            .sound("hit", "sounds/hit.wav")?
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
```

Run that complete example now:

Run the default arena demo:

```bash
cargo run -p game
```

Run the beginner-sized demo:

```bash
GAME_DEMO=simple cargo run -p game
```

## What changed

The `bin/game` binary selects one content plugin with the `GAME_DEMO`
environment variable. With no variable, it runs `arena-content`. With
`GAME_DEMO=simple`, it runs `simple-content`. `GAME_DEMO=testbed` runs the
advanced testbed content crate; it is a reference lab, not beginner copy
material.

Assets live under the workspace `assets/` directory. Content registers paths
relative to that root, such as `textures/test.png` or `sounds/hit.wav`.

Close the window or press `Esc` to quit. In the top-down demos, use WASD or arrow
keys to move, Space or Enter to attack, `R` to reset, and `F1` to toggle the
debug overlay.

## Common errors

From a generated project, run the quick project check before chasing individual
errors:

```bash
game-dev check
```

It runs the setup doctor, validates assets, validates legacy `assets/game.ron`
when present, and then runs `cargo check`.

If startup reports a missing asset, check the path passed to
`game.asset_bag().texture(...)` or `.sound(...)`. The path should not include
the leading `assets/` directory.

If the wrong demo opens, check the exact `GAME_DEMO` value. The supported values
are `simple`, `testbed`, and the default arena demo. Stay on `simple` while
following this beginner tutorial.

If a release build cannot find assets, run with `GAME_ASSET_DIR=assets` or place
the `assets/` directory next to the executable.

## Next step

Make the player prefab in [02 - Your first player](02-your-first-player.md).

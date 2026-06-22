# 10 - Package Your Demo

## Goal

Build a release version, validate its files, and create one folder you can send
to someone else.

## Files to edit

None. Keep the complete `src/main.rs` from
[09 - Add UI and menu](09-add-ui-and-menu.md) unchanged.

## Full code

Your game source is complete at this point. Its entry point still has the same
small shape from chapter 00:

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

Create the package from a local checkout:

```bash
cargo xtask package-demo --release --out dist/my-game
```

The result is:

```text
dist/my-game/
├── game            # game.exe on Windows
├── assets/
├── run.sh
├── run.bat
└── README-RUN.txt
```

## What changed

Before copying, the command checks PNG decoding, the bundled font, supported
sound decoding, rectangular text maps, TMX maps, and LDtk JSON. Its release
build also confirms the shaders compile. It then copies the executable and the
entire `assets/` folder together.

Send the whole `dist/my-game` directory, not just the executable. On Linux run
`./run.sh`; on Windows double-click `run.bat`; on macOS run `./run.sh` in a
Terminal. `README-RUN.txt` inside the package repeats these instructions.

When your single file grows uncomfortable, graduate to a content crate: copy
the structure of `simple-content` or `arena-content`, put your setup inside
`content_plugin!(MyContent, plugin, |game| { ... });`, and continue using the
same beginner prefabs, maps, and rules. Do not begin with `testbed-content`; it
is the deliberately advanced reference lab.

## Common errors

If validation reports a PNG, sound, map, or font path, fix that source asset
and rerun the command. If the destination exists already, choose a new
`--out` directory rather than mixing an old package with a new one. If it
selects the wrong bundled demo, set `GAME_DEMO=simple` before launching it.
For a fast startup check without drawing frames, use `GAME_SMOKE_FRAMES=0`.

## Next step

Use [common errors](common-errors.md) while changing your game, then browse the
[cookbook](../cookbook/README.md) for focused features such as animation,
triggers, waves, and gamepad controls.

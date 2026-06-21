# 10 - Package Your Demo

## Goal

Build a release version and put its executable beside the `assets/` folder it
needs on another machine.

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

Build and copy the release binary with its assets:

```bash
cargo build -p game --release --locked
mkdir -p /tmp/game-package
cp target/release/game /tmp/game-package/
cp -r assets /tmp/game-package/
cd /tmp/game-package
GAME_DEMO=simple ./game
```

## What changed

Debug runs can find the workspace assets automatically. A release package
should keep `game` and `assets/` side by side. Set `GAME_ASSET_DIR=assets` if
you choose a different layout.

When your single file grows uncomfortable, graduate to a content crate: copy
the structure of `arena-content`, move your setup into its `GamePlugin`, and
continue using the same beginner prefabs, maps, and rules. Do not begin with
`testbed-content`; it is the deliberately advanced reference lab.

## Common errors

If the release build reports missing textures, check that the package contains
both `game` and `assets/`. If it selects the wrong demo, set `GAME_DEMO=simple`.
For a fast startup check without drawing frames, use `GAME_SMOKE_FRAMES=0`.

## Next step

Use [common errors](common-errors.md) while changing your game, then browse the
[cookbook](../cookbook/README.md) for focused features such as animation,
triggers, waves, and gamepad controls.

# 01 - Run The Demo

## Goal

Run the game binary, select a content demo, and know where the demo code and
assets live.

## What you will build

Nothing yet. This first step proves the workspace, runtime, renderer, assets, and
selected content crate all start together.

## Files you will edit

None.

## Final code

Run the default arena demo:

```bash
cargo run -p game
```

Run the beginner-sized demo:

```bash
GAME_DEMO=simple cargo run -p game
```

## Explanation

The `bin/game` binary selects one content plugin with the `GAME_DEMO`
environment variable. With no variable, it runs `arena-content`. With
`GAME_DEMO=simple`, it runs `simple-content`. `GAME_DEMO=testbed` runs the
testbed content crate.

Assets live under the workspace `assets/` directory. Content registers paths
relative to that root, such as `textures/test.png` or `sounds/hit.wav`.

Close the window or press `Esc` to quit. In the top-down demos, use WASD or arrow
keys to move, Space or Enter to attack, `R` to reset, and `F1` to toggle the
debug overlay.

## Common errors

If startup reports a missing asset, check the path passed to
`assets.texture(...)` or `assets.sound(...)`. The path should not include the
leading `assets/` directory.

If the wrong demo opens, check the exact `GAME_DEMO` value. The supported values
are `simple`, `testbed`, and the default arena demo.

If a release build cannot find assets, run with `GAME_ASSET_DIR=assets` or place
the `assets/` directory next to the executable.

## Next step

Make the player prefab in [02 - Your first player](02-your-first-player.md).

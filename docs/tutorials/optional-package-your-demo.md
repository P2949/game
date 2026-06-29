# Optional - Package Your Demo

## Goal

Build a release binary and package it with the assets it needs.

## What you will build

A small folder containing the `game` executable and an adjacent `assets/`
directory.

## Files you will edit

None.

## Final code

Use the project package command from your generated game:

```bash
game-dev package --release --out dist/my-game
```

## Explanation

The task validates assets and copies the release executable, `assets/`,
`run.sh`, `run.ps1`, `run.bat`, and `README.txt` into `dist/my-game`. Send that
whole folder. See the newer [Package your demo](10-package-your-demo.md)
tutorial for zip packaging, platform launch details, and common verification
failures.

The same binary can run multiple content crates. Use `GAME_DEMO=simple` for the
beginner demo, `GAME_DEMO=testbed` only for the advanced testbed reference, or
omit `GAME_DEMO` for the arena demo.

Engine contributors can still package the bundled workspace demo from a local
checkout with:

```bash
cargo xtask package-demo --release --out dist/my-game
```

## Common errors

If a release package starts but reports missing textures, confirm the folder
contains both `game` and `assets/` side by side.

If the package uses the wrong demo, set `GAME_DEMO=simple` before launching it.

If you want a quick startup check without rendering frames, use
`GAME_SMOKE_FRAMES=0`.

## Next step

Use [common errors](common-errors.md) while changing your own content crate.

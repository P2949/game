# 08 - Package Your Demo

## Goal

Build a release binary and package it with the assets it needs.

## What you will build

A small folder containing the `game` executable and an adjacent `assets/`
directory.

## Files you will edit

None.

## Final code

Build the binary:

```bash
cargo build -p game --release --locked
```

Create a package folder:

```bash
mkdir -p /tmp/game-package
cp target/release/game /tmp/game-package/
cp -r assets /tmp/game-package/
```

Run it from the package folder:

```bash
(cd /tmp/game-package && GAME_DEMO=simple ./game)
```

## Explanation

Debug runs can find the workspace `assets/` directory automatically. Release
packages should carry `assets/` next to the executable or set `GAME_ASSET_DIR`.

The same binary can run multiple content crates. Use `GAME_DEMO=simple` for the
beginner demo, `GAME_DEMO=testbed` for testbed content, or omit `GAME_DEMO` for
the arena demo.

## Common errors

If a release package starts but reports missing textures, confirm the folder
contains both `game` and `assets/` side by side.

If the package uses the wrong demo, set `GAME_DEMO=simple` before launching it.

If you want a quick startup check without rendering frames, use
`GAME_SMOKE_FRAMES=0`.

## Next step

Use [common errors](common-errors.md) while changing your own content crate.

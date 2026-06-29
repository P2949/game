# Quickstart - Zero to a Running Demo

## Goal

Create a game you can run, then make your first visible changes without
learning engine internals first.

## Files to edit

- `assets/maps/level_1.txt`
- `assets/textures/player.png`
- `src/main.rs`

## Full code

You do not need to write code for this first step. Create and run the generated
demo from anywhere:

```bash
cargo install cargo-generate
cargo generate --git https://github.com/P2949/game templates/simple-demo --name my-first-game
cd my-first-game
cargo install --git https://github.com/P2949/game game-cli
game-dev run
```

From a local checkout, `cargo xtask new-demo ../my-first-game` creates the same
shape with a local path dependency for framework development.

The CI "first 15 minutes" acceptance test follows this same beginner path from
a local checkout:

```bash
cargo install cargo-generate
cargo generate --path templates/simple-demo --name first-demo --destination /tmp
cd /tmp/first-demo
GAME_SMOKE_FRAMES=60 cargo run
printf '%s\n' '##########' '#P..C..E.#' '#..C.....#' '##########' > assets/maps/level_1.txt
game-dev asset-check
game-dev package --release --out dist/first-demo --zip
```

## What changed

The generated project gives you one Rust file and an `assets/` folder. Start by
editing files instead of the engine:

1. Edit `assets/maps/level_1.txt` to change the walls, player, and enemies.
2. Replace `assets/textures/player.png` with your own PNG.
3. Change the player speed in `src/main.rs` (look for `130.0`).
4. Press <kbd>F5</kbd> in a debug build after changing the text map to reload it.

The map uses simple symbols: `#` is a wall, `.` is floor, `P` is the player,
and `E` is an enemy. The generated README explains the rest of the first
changes.

The first build creates small starter files such as
`assets/textures/player.png`, `assets/textures/slime.png`,
`assets/textures/coin.png`, `assets/sounds/hit.wav`, and
`assets/sounds/coin.wav` when they are missing. Your own files with those names
are left untouched.

## Common errors

- **`cargo: command not found`:** install Rust with
  [rustup](https://rustup.rs/), then open a new terminal.
- **The game cannot find assets:** run `cargo run` from inside `my-first-game`.
- **F5 does nothing:** make sure you ran the debug command above, not a release
  build.

## Next step

Read the generated project's README, then continue with
[Run the demo](01-run-the-demo.md) when you want a guided tour of the project.

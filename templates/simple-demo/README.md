# {{title}}

This is the secondary beginner Rust template. Use it when you want to write a
small `src/main.rs` with the beginner Rust API. The primary no-Rust template is
`templates/no-rust-demo` and uses `game.toml` without Cargo project files.

## Start here

1. Run `cargo run` or `game-dev run`.
2. Edit `assets/maps/level_1.txt`.
3. Replace files in `assets/textures/` and `assets/sounds/`.
4. Add a coin by putting `C` on an empty floor tile in the map.
5. Add an enemy by putting `E` on an empty floor tile in the map.
6. Change numbers in `src/main.rs`, such as the player's `130.0` movement
   speed or a coin's score.

The map symbols are simple:

- `#` wall
- `.` floor
- `P` player start (use one)
- `E` enemy
- `C` coin

Press <kbd>F5</kbd> in a debug build after changing the text map to reload it
without restarting the game.

The first build makes small starter assets if they do not already exist:

```text
assets/textures/player.png -> .sprite("player")
assets/textures/slime.png  -> .sprite("slime")
assets/textures/coin.png   -> .sprite("coin")
assets/textures/floor.png  -> .simple_theme("floor", "wall")
assets/textures/wall.png   -> .simple_theme("floor", "wall")
assets/textures/door.png   -> .sprite("door")
assets/textures/bolt.png   -> .sprite("bolt")
assets/sounds/hit.wav      -> .play_sound("hit")
assets/sounds/coin.wav     -> .play_sound("coin")
assets/sounds/shoot.wav    -> .play_sound("shoot")
```

Replace any of them with your own PNG or WAV files whenever you are ready.

## Controls

- Move: WASD, arrow keys, left stick, or D-pad
- Attack / confirm: Space, Enter, left mouse, or the south face button
- Pause: P, Esc, or Start
- Reset: R or Select
- Debug overlay: F1 or the north face button

## Project tools

Install the beginner helper once:

```bash
cargo install --git https://github.com/P2949/game game-cli
```

Useful commands:

```bash
game-dev doctor
game-dev check
game-dev run
game-dev asset-check
game-dev package --release --out dist/my-game --zip
```

## Need a fresh copy?

From a local checkout of the game kit, run:

```bash
cargo xtask new-demo my-game
```

From anywhere, generate the template with:

```bash
cargo install cargo-generate
cargo generate --git https://github.com/P2949/game templates/simple-demo --name my-game
```

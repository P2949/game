# {{title}}

## Start here

1. Run `cargo run`.
2. Edit `assets/maps/level_1.txt`.
3. Replace files in `assets/textures/`.
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

The first build makes placeholder `player.png`, `slime.png`, `coin.png`,
`floor.png`, and `wall.png` files if they do not already exist. Replace any of
them with your own PNG art whenever you are ready.

## Controls

- Move: WASD, arrow keys, left stick, or D-pad
- Attack / confirm: Space, Enter, left mouse, or the south face button
- Pause: P, Esc, or Start
- Reset: R or Select
- Debug overlay: F1 or the north face button

## Need a fresh copy?

From a local checkout of the game kit, run:

```bash
cargo xtask new-demo my-game
```

From anywhere, generate the template with:

```bash
cargo generate gh:P2949/game templates/simple-demo
```

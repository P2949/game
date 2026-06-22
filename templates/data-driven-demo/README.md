# {{title}}

## Start here

1. Run `cargo run`.
2. Edit `assets/game.ron` to change player/enemy/pickup numbers and rules.
3. Edit `assets/maps/level_1.txt` to change the level.
4. Replace PNG files in `assets/textures/` with your own art.

The editable RON file is intentionally small:

- `assets.textures`, `sounds`, and `music` can register conventional asset names.
- `prefabs` define players, enemies, and pickups.
- `maps` connects a text map and its `P`/`E`/`C` legend to prefabs.
- `rules` selects the common first-game behaviors.

The map symbols are:

- `#` wall
- `.` floor
- `P` player start (use one)
- `E` enemy
- `C` coin

Press <kbd>F5</kbd> in a debug build after changing the map. The RON setup is
read on startup; use the small `src/main.rs` to add custom Rust behavior when
you outgrow the standard rules.

## Need a fresh copy?

From a local checkout, run:

```bash
cargo xtask new-demo my-game --data-driven
```

# 03 - Add A Map

## Goal

Add a tiny tile map and spawn the player into it.

## What you will build

A map named `level_1` with walls, floors, a simple tile theme, and one player
spawn point.

## Files you will edit

`crates/simple-content/src/game.rs`

## Final code

```rust
game.map("level_1")
    .tiles([
        "########",
        "#......#",
        "#..P...#",
        "#......#",
        "########",
    ])
    .simple_theme(assets.texture("floor"), assets.texture("wall"))
    .spawn("player_start", "player", cell(3, 2))
    .start();
```

## Explanation

Tiles use `#` for walls and `.` for floor. The `P` in the drawing is a visual
note for you; the actual spawn is the `.spawn(...)` call. `cell(3, 2)` means
column 3, row 2.

`simple_theme` gives the map floor and wall sprites without making you build a
full `TileTheme`. `.start()` marks this as the map loaded when the demo starts.

## Common errors

If the map reports no tile theme, add
`.simple_theme(assets.texture("floor"), assets.texture("wall"))`.

If the map references an unknown prefab, check that `"player"` matches the name
passed to `game.player_prefab("player")`.

If startup says there is no start map, make sure one map ends with `.start()`.

## Next step

Add something to chase the player in [04 - Add an enemy](04-add-an-enemy.md).

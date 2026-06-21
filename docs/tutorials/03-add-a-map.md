# 03 - Add A Map

## Goal

Add a tiny text-map file and spawn the player into it.

## What you will build

A map named `level_1` stored in `assets/maps/level_1.txt`, with walls, floors,
a simple tile theme, and one player spawn point.

## Files to edit

`crates/simple-content/src/game.rs`

## Full code

```rust
use game_starter::prelude::*;

fn main() -> Result<()> {
    run_game("Text Map", |game| {
        game.asset_bag()
            .texture("player", "textures/test.png")?
            .texture("floor", "textures/test.png")?
            .texture("wall", "textures/test.png")?
            .build();

        let controls = game.input(|input| input.top_down_controls())?;

        game.player_prefab("player")
            .sprite("player")
            .moves_with(controls.movement, 130.0)
            .build()?;

        game.map_from_text("level_1", "maps/level_1.txt")
            .simple_theme("floor", "wall")
            .legend('P', "player")
            .start();

        game.rules()
            .top_down_controls(controls)
            .camera_follows_player()
            .build();

        Ok(())
    })
}
```

Create `assets/maps/level_1.txt` with:

```text
########
#......#
#..P...#
#......#
########
```

## What changed

Tiles use `#` for walls and `.` for floor. `P` is a player spawn because the
builder connects it with `.legend('P', "player")`. Keeping the layout in a text
file means you can edit it without touching Rust code.

`simple_theme` gives the map floor and wall sprites without making you build a
full `TileTheme`. `.start()` marks this as the map loaded when the demo starts.

For a tiny temporary prototype, inline rows are still available:

```rust
game.map("level_1")
    .tiles(["########", "#..P...#", "########"])
    .simple_theme("floor", "wall")
    .legend('P', "player")
    .start();
```

## Common errors

If the map reports no tile theme, add
`.simple_theme("floor", "wall")`.

If the map references an unknown prefab, check that `"player"` matches the name
passed to `game.player_prefab("player")`.

If the map reports an unknown symbol, either add a matching `.legend(...)` or
replace that character with `.` or `#`. Every text-map row must have the same
width.

If startup says there is no start map, make sure one map ends with `.start()`.

## Next step

Add something to chase the player in [04 - Add an enemy](04-add-an-enemy.md).

# Rust you need for this game

## Goal

Recognize the small pieces of Rust in the generated game so you can edit it
with confidence.

## Files to edit

- `src/main.rs`

## Full code

This is the shape of the generated program:

```rust
use game_starter::prelude::*;

fn main() -> Result<()> {
    run_game("My Game", |game| {
        game.player_prefab("player")
            .sprite("player")
            .moves_with(controls.movement, 130.0)
            .build()?;

        Ok(())
    })
}
```

## What changed

- **Cargo** runs and builds the project. `cargo run` is the command you will
  use most often.
- `fn main()` is where the program starts.
- `run_game("My Game", |game| { ... })` gives the indented block a short name,
  `game`, so the block can describe your game. That `|game| { ... }` block is a
  **closure**: a small piece of code passed to `run_game` to run during setup.
- A **method chain** is the vertical sequence after `game.player_prefab(...)`.
  Read it from top to bottom: create a player, choose its picture, set movement,
  then finish building it.
- Names in quotes, such as `"player"`, connect things. The same name is used
  by a prefab, a map symbol, and an asset file when those things belong
  together.
- `?` means “if this setup step has a problem, stop and show the useful error.”
  Leave it at the end of setup lines.
- `Ok(())` says setup finished successfully. Leave it as the final line inside
  the `run_game` block.

For first edits, change strings, numbers, map files, and PNGs. You do not need
to learn more Rust before making a small game with this template.

## Common errors

- **A name does not match:** use the same spelling everywhere. For example,
  `.sprite("coin")` needs a registered `coin` texture and the map legend needs
  `.legend('C', "coin")`.
- **A line ends without `?`:** copy the ending from a nearby builder line.
- **`cargo run` reports an error:** read the last few lines first; they usually
  name the map symbol, file, or setup call that needs changing.

## Next step

Return to [Zero to a running demo](00-zero-to-demo.md), then edit a map or a
number in `src/main.rs` and run the game again.

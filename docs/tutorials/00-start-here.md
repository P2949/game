# 00 - Start Here

## Goal

Create the smallest possible one-file game project. You only need a Rust
editor, Cargo, and this workspace; you do not need to learn ECS, lifetimes, or
engine setup first.

## Files to edit

Create `src/main.rs` in a project that depends on `game-starter`.

## Full code

```rust
use game_starter::prelude::*;

fn main() -> Result<()> {
    run_game("My Game", |game| {
        Ok(())
    })
}
```

This is the shape every later chapter keeps: `run_game` starts the game and
hands `game` to your setup code. The next chapter adds actual assets and a map,
which makes the program playable.

## What changed

- `use ...::*` brings in the small beginner vocabulary.
- `main` is where a Rust program starts.
- `run_game` owns window, input, audio, rendering, and the game loop.
- The closure is the list of things your game contains.

## Common errors

If `game_starter` cannot be found, add a `game-starter` dependency to your
project's `Cargo.toml`. If Cargo cannot run at all, install Rust from
<https://rustup.rs> and restart your terminal.

## Next step

Run a working example in [01 - Run the demo](01-run-the-demo.md).

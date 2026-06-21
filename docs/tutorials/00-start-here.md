# 00 - Start Here

## Goal

Create the smallest possible one-file game project. You only need a Rust
editor and Cargo; you do not need to clone this workspace or learn ECS,
lifetimes, or engine setup first.

## Files to edit

Create the project from anywhere:

```bash
cargo install cargo-generate
cargo generate gh:P2949/game templates/simple-demo
cd my-game
cargo run
```

The generator asks for a project name and game title. It gives the project a
git dependency on `game-starter`; the first build creates tiny placeholder
textures you can replace later. If you already have a local checkout, the
equivalent command is `cargo xtask new-demo my-game`.

Then edit `src/main.rs`.

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

Before installing Rust packages or changing game code, install the graphics
prerequisites for your operating system:

- [Windows setup](../setup/windows.md)
- [macOS setup](../setup/macos.md)
- [Linux setup](../setup/linux.md)

Then run `cargo xtask doctor` from a local repository checkout when you need a
prerequisite check. If `game_starter` cannot be fetched, first check your
network connection and the generated `Cargo.toml` git dependency. If Cargo
cannot run at all, install Rust from <https://rustup.rs> and restart your
terminal.

## Next step

Run a working example in [01 - Run the demo](01-run-the-demo.md).

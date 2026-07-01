# Tiled Demo

This is the beginner Rust Tiled example. It uses high-level builder calls and does not require ECS traversal.

From this folder:

```bash
cargo run --locked
```

From the repository root:

```bash
GAME_ASSET_DIR=examples/tiled-demo/assets cargo run -p tiled-demo --locked
```

Edit `assets/maps/tiled_demo.tmx` in Tiled. Objects with class/type/name `Player` and `Slime` are mapped in Rust with:

```rust
.object("Player", "player")
.object("Slime", "slime")
```

Use `examples/data-driven-tiled-demo` for the equivalent no-Rust `assets/game.ron` workflow.

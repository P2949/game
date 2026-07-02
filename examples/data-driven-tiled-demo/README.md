# Data Driven Tiled Demo

Legacy Rust-wrapper example. The primary no-Rust package equivalent is
`examples/no-rust-tiled`.

This example keeps the transitional `src/main.rs` wrapper that loads
`assets/game.ron`. The data file maps Tiled object identifiers to beginner
prefabs and uses the normal top-down beginner rules.

Run it from this folder with:

```bash
cargo run --locked
```

Run it from the workspace root with:

```bash
GAME_ASSET_DIR=examples/data-driven-tiled-demo/assets cargo run -p data-driven-tiled-demo --locked
```

# One-File Demo

This example shows the smallest standalone beginner game shape. It uses:

```rust
use game_starter::prelude::*;
```

The demo registers assets with `game.asset_bag()`, creates player and enemy
prefabs, defines one in-code map, and enables the beginner top-down preset.

Run it from the workspace root:

```bash
cargo run -p one-file-demo
```

Controls:

- Move: WASD or arrow keys
- Attack: Space or Enter
- Reset: R
- Debug overlay: F1

# {{title}}

Generate this template from anywhere with:

```bash
cargo generate gh:P2949/game templates/simple-demo
```

`cargo xtask new-demo <name>` remains available when you are already working
inside a local checkout of this repository.

The generated `src/main.rs` is a one-file beginner game. It imports:

```rust
use game_starter::prelude::*;
```

It registers textures with `game.assets_from_folders()`, builds player and
enemy prefabs, draws a small map, and enables the beginner top-down preset. The
first `cargo run` writes tiny placeholder images into `assets/textures/`,
so the demo opens before you have made any art.

Run it with:

```bash
cargo run
```

This template is deliberately the standalone path. If the game later becomes a
workspace content crate, import `game_kit::beginner::prelude::*` and wrap the
same setup in `content_plugin!(MyContent, plugin, |game| { ... });` instead of
writing plugin glue by hand.

Controls:

- Move: WASD or arrow keys
- Attack: Space or Enter
- Reset: R
- Debug overlay: F1

# Assets and text maps

The initial build creates `player.png`, `slime.png`, `floor.png`, and
`wall.png` in `assets/textures/`. Replace them with your own PNG files
whenever you are ready. The template loads these conventional names with:

```rust
game.assets_from_folders()
    .required_textures(["player", "slime", "floor", "wall"])?
    .build();
```

To add sound later, put a WAV in `assets/sounds/` and register it with
`.sound("hit", "sounds/hit.wav")?` in an `asset_bag()`.

The generated `assets/maps/level_1.txt` is ready to edit immediately. Its map
builder is:

```rust
game.map_from_text_auto("level_1")
    .simple_theme("floor", "wall")
    .legend('P', "player")
    .legend('E', "slime")
    .start();
```

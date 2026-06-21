# Beginner authoring

Start here when you want to make a small game rather than study engine internals.
Use one of these two beginner paths:

1. **Standalone game (start here):** `use game_starter::prelude::*;` and
   `run_game("My Game", |game| { ... })` in one `main.rs` file.
2. **Workspace content crate:** `use game_kit::beginner::prelude::*;` and
   `content_plugin!(MyContent, plugin, |game| { ... });`. The macro gives the
   workspace a plugin without making you write the crate glue yourself.

Keep the standalone path until your game actually benefits from separate
content-crate files.

The usual order is:

1. Register named assets with `asset_bag()`.
2. Define player, enemy, pickup, door, projectile, and spawner prefabs.
3. Describe maps with `map(...)` or `map_from_text(...)`.
4. Compose behavior with `game.rules()`.
5. Add small custom reactions with `on_action`, `on_collect`, or scene hooks.

The [tutorials](tutorials/README.md) teach that path in order. The
[cookbook](cookbook/README.md) is for copying a focused feature. For the most
complete one-file example, see
[`script-like-custom-rules`](../examples/script-like-custom-rules/src/main.rs).

Use named assets in gameplay:

```rust
game.asset_bag()
    .texture_auto("player")?
    .sound_auto("shoot")?
    .build();

game.on_action_cooldown(controls.attack, 0.2, |game| {
    game.player().shoot("bolt").towards_mouse();
    game.audio().play_sound("shoot");
});
```

For a conventional folder layout, validate all of a starter game's files in one
place:

```rust
game.assets_from_folders()
    .required_textures(["player", "slime", "floor", "wall"])?
    .required_sounds(["hit"])?
    .build();
```

Put those files in `assets/textures/` and `assets/sounds/`. If a required file
is missing, setup tells you the exact path to add and how to use a custom path
instead.

The beginner helpers keep engine details out of your game code.

## Fast map iteration

Use `map_from_text_auto("level_1")` for `assets/maps/level_1.txt`. In a debug
build, press F5 after editing that file to reload the current map without
recompiling Rust. Release builds keep this development action disabled unless
you deliberately set `GAME_DEV_RELOAD=1`.

If the project also uses `game.tuning_from_file(...)`, the same F5 action
reloads its tuning RON file before respawning the text map.

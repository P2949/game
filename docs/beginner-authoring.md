# Beginner authoring

Start here when you want to make a small game rather than study engine internals.
Use one of these beginner paths:

1. **No-Rust data file:** edit `assets/game.ron` and `assets/maps/*.txt`.
   Start with `templates/data-driven-demo`,
   `examples/data-driven-events-demo`, `examples/data-driven-waves-demo`,
   `examples/data-driven-projectiles-demo`, or `examples/data-driven-full-demo`.
2. **Standalone Rust game:** `use game_starter::prelude::*;` and
   `run_game("My Game", |game| { ... })` in one `main.rs` file.
3. **Workspace content crate:** `use game_kit::beginner::prelude::*;` and
   `content_plugin!(MyContent, plugin, |game| { ... });`. The macro gives the
   workspace a plugin without making you write the crate glue yourself.

Keep the standalone path until your game actually benefits from separate
content-crate files.

## Choose your starting point

- **No-Rust / edit data first:** `templates/data-driven-demo`,
  `examples/data-driven-events-demo`, `examples/data-driven-waves-demo`,
  `examples/data-driven-projectiles-demo`, and `examples/data-driven-full-demo`.
- **Beginner Rust / copy this first:** `examples/one-file-demo`,
  `examples/no-rust-shapes-demo`, `examples/script-like-custom-rules`,
  `simple-content`, and `templates/simple-demo`.
- **Structured beginner Rust:** `arena-content`, which uses the same
  beginner-facing API but splits assets, maps, and plugin setup into small
  files.
- **Advanced / do not copy first:** `testbed-content`. It is a lab for manual
  systems, RON maps, tuple prefabs, direct component composition, custom
  state, and lower-level facade APIs.

| Feature | No-Rust data-driven | Beginner Rust | Advanced |
| --- | --- | --- | --- |
| Player/enemy/pickups | yes | yes | yes |
| Doors/maps/scenes | yes | yes | yes |
| Projectiles/spawners | yes | yes | yes |
| Custom countdown/explosion | yes/basic | yes | yes/manual |
| Custom ECS systems | no | no | yes |
| No Rust required | yes | no | no |

## When to use typed assets

Use `asset_bag()` first. When a content crate grows and you want compile-time
field names, use a small typed asset struct like `arena-content`. This is an
organization choice, not a reason to switch to the advanced API.

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
    .required_textures(["player", "slime", "coin", "floor", "wall", "door", "bolt"])?
    .required_sounds(["hit", "coin", "shoot"])?
    .build();
```

Put those files in `assets/textures/` and `assets/sounds/`. If a required file
is missing, setup tells you the exact path to add and how to use a custom path
instead.

The folder convention is literal:

```text
assets/textures/player.png -> .sprite("player")
assets/sounds/hit.wav      -> .play_sound("hit")
assets/music/theme.ogg     -> .play_music("theme")
assets/maps/level_1.txt    -> map_from_text_auto("level_1")
```

For quick prototypes, you can register every supported file directly in the
conventional folders instead:

```rust
game.assets_from_folders()
    .discover_textures()?
    .discover_sounds()?
    .discover_music()?
    .build();
```

Filename stems become names: `assets/textures/player.png` becomes `"player"`,
`assets/sounds/hit.wav` becomes `"hit"`, and `assets/music/theme.ogg` becomes
`"theme"`. Keep `required_*` in tutorials and finished games when missing
files should stop setup with a direct diagnostic.

Animations use the same asset-root convention. Put the image and metadata here:

```text
assets/textures/player_sheet.png
assets/animations/player.ron
```

Then load the metadata by name:

```rust
let assets = game.assets_from_folders()
    .required_textures(["player_sheet"])?
    .animation_sheet_auto("player")?
    .build();

game.player_prefab("player")
    .animation_sheet(assets.animation_sheet("player"))
    .play("idle")
    .build()?;
```

`game-dev asset-check` validates PNGs, WAV/OGG/MP3 sound files, rectangular
text maps, `assets/game.ron`, and animation metadata texture references.

The beginner helpers keep engine details out of your game code.

## Data-driven files

New data files use a version and structured names:

```ron
version: 1,
controls: TopDown,
rules: [TopDownControls, PlayerCollectsPickups, ShowBasicUi],
```

Legacy string controls and rules still load, but current templates use the
structured form. The data path compiles through the same public beginner
builders as Rust content, covering players, enemies, pickups, doors,
projectiles, spawners, triggers, checkpoints, scene flow, music, player
shooting, enemy drops, UI, win conditions, timed/event rules, and a countdown
custom-rule helper.

## Fast map iteration

Use `map_from_text_auto("level_1")` for `assets/maps/level_1.txt`. In a debug
build, press F5 after editing that file to reload the current map without
recompiling Rust. Release builds keep this development action disabled unless
you deliberately set `GAME_DEV_RELOAD=1`.

If the project also uses `game.tuning_from_file(...)`, the same F5 action
reloads its tuning RON file before respawning the text map.

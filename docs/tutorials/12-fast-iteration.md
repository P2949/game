# Fast iteration

You can change a text map without restarting the game or rebuilding Rust code.
This is the beginner editor loop: edit a small file, save, and press F5.

## Put your map in the standard folder

Create `assets/maps/level_1.txt`:

```text
#########
#...E...#
#...P...#
#########
```

Load it with the short map helper:

```rust
game.map_from_text_auto("level_1")
    .simple_theme("floor", "wall")
    .legend('P', "player")
    .legend('E', "slime")
    .start();
```

Use your normal top-down controls. They already include the reload action:

```rust
let controls = game.input(|input| input.top_down_controls())?;

game.rules()
    .top_down_controls(controls)
    .build();
```

## Edit, save, reload

Run a debug build, edit `assets/maps/level_1.txt`, save it, then press F5. The
game reparses the current text map and respawns its objects with the same
prefabs, legend, and tile theme.

If your prefab values come from `game.tuning_from_file(...)`, F5 reloads that
RON file first. The respawned actors then use the new health, speed, and damage
values. See [Live tuning](../cookbook/live-tuning.md) for the complete pattern.

Debug builds enable this automatically. For a release build, explicitly opt in:

```bash
GAME_DEV_RELOAD=1 cargo run --release
```

Press F1 to open the debug overlay. It shows the current map, asset count, and
whether the most recent reload worked. If a map has a typo, fix the reported
row or symbol and press F5 again.

Only text maps reload today. Changing Rust code, textures, or sounds still
requires a restart. This keeps the first iteration loop predictable while those
asset replacement paths stay deliberately small and safe.

# Fast iteration

You can change a text map without restarting the game or rebuilding Rust code.
This is the beginner editor loop: edit a small file, save, and press F5.

For a primary no-Rust package, keep the prebuilt player running through the
watch command while you edit:

```bash
game-dev preview --watch
```

That command watches `game.toml` and the project assets, then restarts the
prebuilt player after a save. Adding or removing prefabs, maps, rules, actions,
or asset keys uses this preview restart path. It does not run Cargo and does not
compile user Rust code. A packaged player reads the current `game.toml` and
assets each time it launches, so normal packaged edits are picked up by
relaunching the package.

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
tuning TOML file first. The respawned actors then use the new health, speed,
and damage values. See [Live tuning](../cookbook/live-tuning.md) for the
complete pattern.

If your setup comes from the legacy `game.load_beginner_file("game.ron")` path,
F5 also reparses and validates `assets/game.ron`. Existing data can change and
the current map respawns from the updated file. Future spawns from beginner
rules use the updated prefab values too, and existing custom countdown rule
details reload. Scene text/buttons, audio scene settings, and existing action
settings also reload when their scene names and input bindings stay the same.
Adding, removing, or reordering asset, prefab, map, action, scene, or custom
rule names still requires a restart so the runtime registries stay stable.

| Change | F5 reload? | Notes |
| --- | --- | --- |
| Edit existing text map file | Yes | Current map can respawn. |
| Change map path for existing map | Yes | Uses the existing map identity. |
| Change existing prefab values | Partial | Runtime config updates for respawns and future beginner-rule spawns. |
| Change existing custom countdown rule values | Partial | Tag/key identity must stay the same. |
| Change existing scene text/menu/audio settings | Partial | Scene identity and input bindings must stay the same. |
| Replace PNG/WAV for existing key | Yes | Registered asset handles are reloaded in place. |
| Add a new prefab | No | Use `game-dev preview --watch` for a prebuilt restart. |
| Add a new map | No | Use `game-dev preview --watch` for a prebuilt restart. |
| Add a new texture key | No | Use `game-dev preview --watch` for a prebuilt restart. |
| Add a new action | No | Use `game-dev preview --watch`; action IDs are created at startup. |
| Add/remove/reorder scenes or rules | No | Use `game-dev preview --watch`; runtime systems are created at startup. |

Debug builds enable this automatically. For a release build, explicitly opt in:

```bash
GAME_DEV_RELOAD=1 cargo run --release
```

Press F1 to open the debug overlay. It shows the current map, asset count, and
whether the most recent reload worked. The reload label uses the loaded file
name, such as `game.toml reload: partial` for a primary package or
`game.ron reload: partial` for the legacy RON path. If a map has a typo, fix
the reported row or symbol and press F5 again.

Text maps, configured tuning, partial legacy `game.ron` prefab/map data, and
existing custom countdown rule details reload today. Registered textures/sounds
reload too. Changing Rust code, scene names, scene input bindings, action input
bindings, or the enabled rule list still requires a restart; in the primary
no-Rust workflow, `game-dev preview --watch` performs that restart without a
build.
Texture reload keeps the same content handle even when the replacement image
dimensions change; sound reload stops voices using old samples so later
playback uses the new file.

# Live tuning

Keep numbers such as health, movement speed, and melee damage in a small RON
file so you can tune them without recompiling Rust.

Create `assets/tuning/arena.ron`:

```ron
(
    "player.health": 100.0,
    "player.speed": 130.0,
    "slime.health": 40.0,
    "slime.speed": 80.0,
    "slime.melee_damage": 6.0,
)
```

Load it while defining the prefabs:

```rust
let tuning = game.tuning_from_file("tuning/arena.ron")?;

game.player_prefab("player")
    .sprite("player")
    .moves_with(controls.movement, tuning.float("player.speed"))
    .health(tuning.int("player.health"))
    .build()?;

game.enemy_prefab("slime")
    .sprite("slime")
    .speed(tuning.float("slime.speed"))
    .melee(26.0, tuning.int("slime.melee_damage"))
    .chases_player()
    .build()?;
```

## Reload while you work

Use a text map such as `assets/maps/level_1.txt`, run a debug build, save the
RON file, then press F5. The standard development reload action reloads a
configured tuning file before it reparses and respawns the current text map.
Freshly spawned actors therefore use the new numbers.

For a release build, opt into that same manual development action:

```bash
GAME_DEV_RELOAD=1 cargo run --release
```

Tuning values are copied into components when an actor spawns. Reloading does
not mutate actors already alive in the world; the map reload is what creates
new actors using the new values.

## Automatic file watching

`GAME_HOT_RELOAD=1` is reserved for the upcoming automatic watcher. It is not
available in this build yet: until the runtime watcher is installed, save and
press F5. Textures and sounds also still require a restart; their renderer
replacement path is deliberately tracked separately as Phase 3b.

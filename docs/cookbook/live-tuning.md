# Live tuning

Keep numbers such as health, movement speed, and melee damage in a small TOML
tuning file so you can tune them without recompiling Rust.

Create `assets/tuning/arena.toml`:

```toml
[tuning]
player_health = 100
player_speed = 130
slime_health = 40
slime_speed = 80
slime_melee_damage = 6
```

Load it while defining the prefabs:

```rust
let tuning = game.tuning_from_file("tuning/arena.toml")?;

game.player_prefab("player")
    .sprite("player")
    .moves_with(controls.movement, tuning.float("player_speed"))
    .health(tuning.int("player_health"))
    .build()?;

game.enemy_prefab("slime")
    .sprite("slime")
    .speed(tuning.float("slime_speed"))
    .melee(26.0, tuning.int("slime_melee_damage"))
    .chases_player()
    .build()?;
```

## Reload while you work

Use a text map such as `assets/maps/level_1.txt`, run a debug build, save the
tuning TOML file, then press F5. The standard development reload action reloads
a configured tuning file before it reparses and respawns the current text map.
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
press F5. Registered textures and sounds reload in the same development loop;
texture replacement preserves its content handle, while sound replacement stops
any old static-sound voice and uses the new file for later plays. A streamed
music track restarts from its updated file. This manual F5 loop is deliberately
separate from the future automatic watcher.

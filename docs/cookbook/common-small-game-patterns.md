# Common Small-Game Patterns

These short combinations cover the first questions that come up in a small
top-down game.

## Locked exit

```rust
game.door_prefab("exit")
    .sprite("door")
    .change_map("level_2")
    .requires_all_enemies_dead()
    .build()?;

game.rules().doors_change_maps().dead_enemies_despawn().build();
```

## Timed enemy waves

```rust
game.spawner_prefab("spawner")
    .spawn("slime")
    .every_seconds(2.0)
    .max_alive(4)
    .near_player(96.0)
    .build()?;

game.rules().spawners_spawn_prefabs().build();
```

## Win zone

```rust
game.trigger_prefab("finish")
    .size(vec2(64.0, 64.0))
    .build()?;

game.on_enter_area("player", "finish", |event| {
    event.play_sound("win");
    event.actor().heal(999);
});
```

For a score pickup, a projectile, or a multi-level door, use the focused recipe
linked from the [cookbook index](README.md).

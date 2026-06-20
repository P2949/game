# Enemy Waves

Copy [examples/waves-demo/src/main.rs](../../examples/waves-demo/src/main.rs)
when you want timed enemy waves with a maximum alive count.

The recipe registers a spawner prefab for the map:

```rust
game.spawner_prefab("spawner")
    .spawn("slime")
    .every_seconds(2.0)
    .max_alive(4)
    .near_player(96.0)
    .build()?;
```

Enable the rule that runs the registered spawners:

```rust
game.rules()
    .spawners_spawn_prefabs()
    .build();
```

# Enemy Waves

Copy [examples/waves-demo/src/main.rs](../../examples/waves-demo/src/main.rs)
when you want timed enemy waves with a maximum alive count.

The recipe registers a spawner prefab for the map:

```rust
game.spawner_prefab("spawner")
    .spawn("slime")
    .every_seconds(2.0)
    .max_alive(4)
    .build()?;
```

Then it uses a beginner timer to spawn enemies near the player:

```rust
game.every_seconds(2.0, |game: &mut Game<'_, '_>| {
    if game.enemies().alive().count() < 4 {
        game.spawn("slime").near_player(96.0);
    }
});
```

# Two Levels

Copy [examples/two-level-demo/src/main.rs](../../examples/two-level-demo/src/main.rs)
when you want a door that opens after enemies are defeated.

The recipe uses:

```rust
game.door_prefab("exit")
    .sprite(assets.texture("door"))
    .change_map("level_2")
    .requires_all_enemies_dead()
    .build()?;

game.rules()
    .top_down_controls(controls)
    .doors_change_maps()
    .dead_enemies_despawn()
    .build();
```

Declare the first map with `.start()` and later maps with `.finish()`.

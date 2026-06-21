# Two Levels

Copy [examples/two-level-demo/src/main.rs](../../examples/two-level-demo/src/main.rs)
when you want a door that opens after enemies are defeated.

The recipe uses:

```rust
game.door_prefab("exit")
    .sprite("door")
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

For file-authored levels, use the same setup with one map file per level:

```rust
game.map_from_text("level_2", "maps/level_2.txt")
    .simple_theme("floor", "wall")
    .legend('P', "player")
    .legend('E', "slime")
    .finish();
```

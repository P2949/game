# Enemy drops

Copy [the enemy-drops demo](../../examples/enemy-drops-demo/src/main.rs).

```rust
game.enemy_prefab("slime")
    .sprite("slime")
    .chases_player()
    .drops("coin")
    .build()?;

game.rules().enemy_drops().build();
```

The drop spawns at the defeated enemy's position. Use `.drop_chance(0.5)` after
`.drops(...)` when the drop should be less common.

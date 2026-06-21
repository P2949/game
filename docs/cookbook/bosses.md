# Bosses

Copy [the boss demo](../../examples/boss-demo/src/main.rs). A boss is an enemy
with intentionally larger health, size, and attack values:

```rust
game.enemy_prefab("boss")
    .sprite("boss")
    .size(44.0)
    .health(300)
    .chases_player()
    .melee(42.0, 12)
    .build()?;
```

Add projectiles or spawners only after the simple close-range version feels
right; that keeps the first boss easy to tune.

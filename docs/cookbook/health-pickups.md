# Health pickups

Copy [the health-pickup demo](../../examples/health-pickup-demo/src/main.rs).

```rust
game.pickup_prefab("heart")
    .sprite("heart")
    .heal_player(25)
    .despawn_on_collect()
    .build()?;
```

Add `player_collects_pickups()` to your rules. Healing never raises health past
the player's maximum.

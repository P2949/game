# Coins And Score

Copy [examples/coin-collector/src/main.rs](../../examples/coin-collector/src/main.rs)
when you want coins, score, collect sounds, and score UI.

The recipe uses:

```rust
game.pickup_prefab("coin")
    .score(1)
    .play_sound(assets.sound("coin"))
    .despawn_on_collect()
    .build()?;

game.rules()
    .top_down_controls(controls)
    .show_score()
    .build();

game.on_player_collect_pickup(|game| {
    game.camera2d().shake(0.08);
});
```

For score UI, use the high-level rule:

```rust
game.rules().show_score().build();
```

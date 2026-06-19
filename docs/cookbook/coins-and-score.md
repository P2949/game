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
    .build();

game.on_player_collect_pickup(|game: &mut Game<'_, '_>| {
    game.camera2d().shake(0.08);
});
```

For score UI, draw text from the score helper:

```rust
game.draw_ui(|game: &mut Game<'_, '_>, _dt| {
    let score = game.score().value();
    game.text(&format!("Score: {score}"), vec2(24.0, 24.0), vec4(1.0, 0.95, 0.35, 1.0));
});
```

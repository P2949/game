# Inventory and status

Copy [the status-panel demo](../../examples/inventory-demo/src/main.rs).

```rust
game.draw_ui(|game, _dt| {
    game.ui()
        .status_panel()
        .score()
        .player_health()
        .enemy_count()
        .build();
});
```

Use a dialog or panel for a small hand-written inventory list; the status panel
is the right default for score and combat information.

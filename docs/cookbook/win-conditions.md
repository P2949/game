# Win conditions

Copy [the win-condition demo](../../examples/win-condition-demo/src/main.rs).
Use a `win` scene and let rules change to it when the final objective is gone:

```rust
game.rules()
    .win_when_all_pickups_collected()
    .win_when_all_enemies_dead()
    .show_win_panel()
    .build();
```

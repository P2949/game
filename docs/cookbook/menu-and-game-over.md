# Menu And Game Over

Copy [examples/menu-game-over/src/main.rs](../../examples/menu-game-over/src/main.rs)
when you want a menu scene, a gameplay scene, and a game-over scene.

The recipe uses:

```rust
game.use_simple_scene_flow()
    .menu("menu")
    .game("game")
    .game_over("game_over")
    .start_on(controls.attack)
    .restart_on(controls.reset)
    .build();
```

The helper draws basic menu/game-over text, starts the game from the menu, sends
the player to `game_over` on death, and restarts the game scene from game over.

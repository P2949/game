# Menu And Game Over

Copy [examples/menu-game-over/src/main.rs](../../examples/menu-game-over/src/main.rs)
when you want a menu scene, a gameplay scene, and a game-over scene.

The recipe uses:

```rust
game.use_simple_scene_flow()
    .menu("menu")
    .menu_text("Press Space to Start")
    .game("game")
    .game_over("game_over")
    .game_over_text("Game Over - Press R")
    .win("win")
    .win_when_all_enemies_dead()
    .start_on(controls.attack)
    .restart_on(controls.reset)
    .build();
```

The helper draws the configured menu/game-over/win text, starts the game from
the menu, sends the player to `game_over` on death, restarts the game scene from
game over, and can transition to `win` when every enemy is dead (or every pickup
has been collected with `.win_when_all_pickups_collected()`).

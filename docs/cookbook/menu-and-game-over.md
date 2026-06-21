# Menu And Game Over

Copy [examples/menu-game-over/src/main.rs](../../examples/menu-game-over/src/main.rs)
when you want a menu scene, a gameplay scene, and a game-over scene.

The recipe uses:

```rust
game.use_simple_scene_flow()
    .menu("menu")
    .menu_title("My Game")
    .game("game")
    .menu_button("Start", "game")
    .game_over("game_over")
    .game_over_text("Game Over - Press R")
    .game_over_button("Restart")
    .win("win")
    .win_button("Play Again")
    .win_when_all_enemies_dead()
    .start_on(controls.attack)
    .restart_on(controls.reset)
    .build();
```

The helper draws the configured menu/game-over/win panels, starts the game from
the menu, sends the player to `game_over` on death, restarts the game scene from
game over, and can transition to `win` when every enemy is dead (or every pickup
has been collected with `.win_when_all_pickups_collected()`). `menu_button` and
`game_over_button` add clickable controls; the action bindings remain useful for
keyboard and gamepad play.

Add the standard score, health, and state panels without coordinate math:

```rust
game.rules()
    .show_score()
    .show_player_health()
    .show_game_over_panel()
    .show_win_panel()
    .build();
```

For a custom title panel, use
`game.ui().panel("My Game").line("Press Space to Start").center();` from a
`game.draw_ui(...)` callback. The built-in scene flow already draws its own
configured menu, game-over, and win panels.

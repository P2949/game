# UI And Menus

Use rule helpers for score, health, pause, and game-over UI without screen
coordinates:

```rust
game.rules()
    .show_score()
    .show_player_health()
    .show_pause_menu()
    .show_game_over_panel()
    .build();
```

For a custom panel, draw it in a UI callback. Panels and buttons have visible
screen-space rectangles, while their API stays deliberately small:

```rust
game.draw_ui(|game, _dt| {
    game.ui()
        .panel("Inventory")
        .line("Coins: 12")
        .line("Potion: 1")
        .center();

    game.ui().button("Restart").center().on_click(|game| {
        game.restart_level();
    });
});
```

Use the immediate form when the button should choose between several actions:

```rust
game.draw_ui(|game, _dt| {
    if game.ui().button("Restart").center().clicked() {
        game.restart_level();
    }
});
```

For a few labels on one row or a column, use the lightweight layout helpers:

```rust
game.draw_ui(|game, _dt| {
    game.ui()
        .top_left()
        .horizontal()
        .top_left_text("Gold: 12")
        .top_left_text("Keys: 1")
        .build();
});
```

For a title/game-over/win flow, start with the
[menu and game-over recipe](menu-and-game-over.md). It can also create
clickable scene-flow controls without coordinate math:

```rust
game.use_simple_scene_flow()
    .menu("menu")
    .game("level_1")
    .menu_button("Start", "level_1")
    .game_over("game_over")
    .game_over_button("Restart")
    .build();
```

The rules above provide the gameplay HUD.

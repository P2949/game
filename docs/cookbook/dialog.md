# Dialog

Copy [the dialog demo](../../examples/dialog-demo/src/main.rs).

```rust
game.draw_ui(|game, _dt| {
    game.ui()
        .dialog("Old Man")
        .line("Welcome to the arena.")
        .line("Collect all coins!")
        .build();
});
```

This first dialog helper is deliberately linear: it displays a clear message
without introducing branches or a conversation system.

## Choices

Use a panel for a short speaker-and-line dialog, then use the focus-aware menu
for choices. This keeps dialog copy and scene actions simple for keyboard,
mouse, and gamepad users:

```rust
game.draw_ui(|game, _dt| {
    game.ui()
        .dialog("Guide")
        .line("Collect all coins!")
        .build();

    game.ui()
        .menu("What next?")
        .button("Continue")
        .go_to_scene("game")
        .button("Quit")
        .quit()
        .build();
});
```

`UiMenu` keeps a `UiFocus` selection and uses the standard menu-up,
menu-down, and accept controls. It is the current beginner choice UI; richer
branching dialog data can be layered on later without exposing low-level UI
state.

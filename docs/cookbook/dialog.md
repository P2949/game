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

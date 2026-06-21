# Title menu

Copy [the title-menu demo](../../examples/title-menu-demo/src/main.rs).

```rust
game.ui()
    .menu("My Game")
    .button("Start").go_to_scene("game")
    .button("Quit").quit()
    .build();
```

Up/Down and the controller D-pad select a button; Space, Enter, or controller
South activates it. Mouse hover and click work too.

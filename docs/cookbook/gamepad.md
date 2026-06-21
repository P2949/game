# Gamepad Controls

The standard controls work with keyboard, mouse, and the first connected
gamepad—no platform-specific code required:

```rust
let controls = game.input(|input| input.top_down_controls())?;

game.rules().top_down_controls(controls).build();
```

The left stick moves, the south face button attacks, Start pauses, and Select
resets. Keyboard alternatives remain WASD/arrows, Space/Enter, P/Escape, and R.

For a controller-only action, bind the neutral gamepad name rather than SDL
details:

```rust
let dash = game.input(|input| input.action("dash")?.gamepad_east())?;
game.on_action(dash, |game| {
    game.camera2d().shake(0.1);
});
```

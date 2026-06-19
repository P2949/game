# Animations

Copy [examples/animation-demo/src/main.rs](../../examples/animation-demo/src/main.rs)
when you want idle, walk, and attack clips from a sprite sheet.

The recipe uses:

```rust
game.player_prefab("player")
    .spritesheet(assets.spritesheet("player"))
    .idle(0..1)
    .walk(1..3)
    .attack(3..4)
    .moves_with(controls.movement, 130.0)
    .build()?;
```

Then enable movement and attack animation in the preset:

```rust
game.use_top_down_game()
    .controls(controls)
    .with_player_animation_by_movement()
    .with_attack_animation("attack")
    .build();
```

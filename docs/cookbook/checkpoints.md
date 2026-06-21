# Checkpoints

Copy [the checkpoint demo](../../examples/checkpoint-demo/src/main.rs).

```rust
game.checkpoint_prefab("checkpoint")
    .sprite("checkpoint")
    .build()?;

game.rules()
    .player_activates_checkpoints()
    .respawn_at_checkpoint()
    .build();
```

The latest marker the player enters becomes the respawn position after death.

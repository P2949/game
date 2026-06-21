# Damage zones

Copy [the damage-zone demo](../../examples/damage-zone-demo/src/main.rs).

```rust
game.trigger_prefab("lava")
    .sprite("lava")
    .size(vec2(48.0, 48.0))
    .build()?;

game.on_collision("player", "lava", |event| {
    event.player().damage(1);
});
```

Use `on_enter_area` instead when damage should happen once on entry rather than
every update while the player remains in the zone.

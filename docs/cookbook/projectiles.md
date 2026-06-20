# Projectiles

Copy [examples/projectile-demo/src/main.rs](../../examples/projectile-demo/src/main.rs)
when you want an attack button to fire a bolt that moves, damages enemies, and
expires automatically.

The recipe uses:

```rust
game.projectile_prefab("bolt")
    .sprite("bolt")
    .damage(15)
    .speed(260.0)
    .lifetime(0.8)
    .despawn_on_hit()
    .build()?;
```

Enable the behavior rules, then wire the attack action:

```rust
game.rules()
    .projectiles()
    .build();

game.on_action_cooldown(controls.attack, 0.2, |game| {
    game.player().shoot("bolt").towards_mouse();
    game.play_sound_named("shoot");
});
```

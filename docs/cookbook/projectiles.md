# Projectiles

Copy [examples/projectile-demo/src/main.rs](../../examples/projectile-demo/src/main.rs)
when you want an attack button to spawn a visible bolt and damage nearby enemies.

The recipe uses:

```rust
game.projectile_prefab("bolt")
    .sprite(assets.texture("bolt"))
    .damage(15)
    .speed(260.0)
    .lifetime(0.8)
    .despawn_on_hit()
    .build()?;
```

Then wire the attack action:

```rust
game.on_action_cooldown(controls.attack, 0.2, move |game: &mut Game<'_, '_>| {
    game.spawn("bolt").near_player(28.0);
    game.enemies().alive().near_player(96.0).damage(15);
});
```

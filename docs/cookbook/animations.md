# Animations

Copy [examples/animation-demo/src/main.rs](../../examples/animation-demo/src/main.rs)
when you want idle, directional walk, flight, impact, attack, or death clips
from a sprite sheet.

The recipe uses:

```rust
game.player_prefab("player")
    .spritesheet(assets.spritesheet("player"))
    .idle(0..1)
    .walk_up(0..1)
    .walk_down(1..2)
    .walk_left(2..3)
    .walk_right(3..4)
    .moves_with(controls.movement, 130.0)
    .build()?;
```

Then let the rule choose the walk clip from velocity. It falls back to `walk`
when a prefab intentionally omits a direction, and uses `idle` when stopped:

```rust
game.rules()
    .top_down_controls(controls)
    .animate_player_directionally()
    .animate_enemies_directionally()
    .build();
```

For a player-fired projectile, `flight` loops until it hits and `impact` plays
once before it is removed:

```rust
game.projectile_prefab("bolt")
    .spritesheet(assets.spritesheet("bolt"))
    .flight(0..2)
    .impact(2..4)
    .despawn_on_hit()
    .build()?;

game.rules()
    .projectiles()
    .projectile_impact_animation_before_despawn()
    .build();
```

Use `.attack(...)` for a one-shot player attack, `.die(...)` with
`.dead_enemies_play_death_animation()` and
`.dead_enemies_despawn_after_animation()` for enemy death, and
`game.on_animation_finished("impact", |event| { ... })` for a custom action
when a one-shot clip ends.

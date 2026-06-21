# Animations

Copy [examples/animation-demo/src/main.rs](../../examples/animation-demo/src/main.rs)
when you want idle, directional walk, flight, impact, attack, or death clips
from a sprite sheet.

Put clip names in `assets/animations/player.ron` so the Rust game setup stays
about game behavior instead of frame numbers:

```ron
(
    texture: "textures/player.png",
    columns: 4,
    rows: 1,
    clips: {
        "idle": (frames: [0], fps: 6.0),
        "walk_right": (frames: [3], fps: 10.0),
        "attack_right": (frames: [0, 1], fps: 12.0, looping: false),
    },
)
```

Load and use that sheet with no frame ranges in Rust:

```rust
let assets = game.asset_bag()
    .spritesheet_from_meta("player", "animations/player.ron")?
    .build();

game.player_prefab("player")
    .animation_sheet(assets.animation_sheet("player"))
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
    .animate_attacks_directionally()
    .build();
```

`.animate_attacks_directionally()` plays `attack_up`, `attack_down`,
`attack_left`, or `attack_right` when the player attacks. It remembers the last
movement direction for a stationary attack and falls back to a normal
`.attack(...)` clip if a directional clip is absent. One-shot attack clips take
priority over walk/idle until they finish.

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

If you prefer to keep clips in Rust, use `.attack(...)`, `.attack_up(...)`,
`.attack_down(...)`, `.attack_left(...)`, and `.attack_right(...)` for one-shot
player attacks. Use `.die(...)` with
`.dead_enemies_play_death_animation()` and
`.dead_enemies_despawn_after_animation()` for enemy death, and
`game.on_animation_finished("impact", |event| { ... })` for a custom action
when a one-shot clip ends.

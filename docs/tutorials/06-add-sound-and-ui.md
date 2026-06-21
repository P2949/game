# 06 - Add Sound And UI

## Goal

Add a hit sound, pause/death UI, reset, and the beginner debug overlay.

## What you will build

The small quality-of-life layer that makes the demo easier to test: sound on
hits, `R` to reset, `P` to pause, and `F1` for debug text.

## Files you will edit

`crates/simple-content/src/game.rs`

## Final code

Register the sound:

```rust
let assets = game
    .asset_bag()
    .texture("floor", "textures/test.png")?
    .texture("wall", "textures/test.png")?
    .texture("player", "textures/test.png")?
    .texture("slime", "textures/test.png")?
    .sound("hit", "sounds/hit.wav")?
    .build();
```

Use the registered name in the beginner top-down preset:

```rust
game.use_top_down_game()
    .controls(controls)
    .hit_sound_named("hit")
    .with_melee_combat()
    .with_enemy_chase()
    .with_collision()
    .with_camera_follow()
    .with_pause_death_ui()
    .build();
```

## Explanation

The preset resolves the name registered by `asset_bag`. The runtime loads the
actual sound file from `assets/sounds/hit.wav`.

`with_pause_death_ui` draws simple text when the game is paused or the player is
dead. `debug_toggle` lets the preset toggle `DebugOverlay`; by default the
top-down controls bind that to `F1`.

## Common errors

If `F1` does not show the overlay, check that the preset has
`.controls(controls)` or an explicit `.debug_toggle(controls.debug_overlay)`.

If `R` resets but also restarts the current map, that is expected in this simple
preset because both reset paths can share the same action while you are learning.

If the sound fails validation, the path should be `sounds/hit.wav`, not
`assets/sounds/hit.wav`.

## Next step

Animate the player in [07 - Add animation](07-add-animation.md).

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
#[derive(Clone, Copy, Debug)]
struct SimpleAssets {
    floor: TextureHandle,
    wall: TextureHandle,
    player: TextureHandle,
    slime: TextureHandle,
    hit: SoundHandle,
}

fn register_assets(assets: &mut AssetAuthor<'_>) -> Result<SimpleAssets> {
    Ok(SimpleAssets {
        floor: assets.texture("simple/floor", "textures/test.png")?,
        wall: assets.texture("simple/wall", "textures/test.png")?,
        player: assets.texture("simple/player", "textures/test.png")?,
        slime: assets.texture("simple/slime", "textures/test.png")?,
        hit: assets.sound("simple/hit", "sounds/hit.wav")?,
    })
}
```

Use it in the preset:

```rust
game.use_top_down_game()
    .movement(controls.movement)
    .attack(controls.attack)
    .pause(controls.pause)
    .reset(controls.reset)
    .debug_toggle(controls.debug_overlay)
    .debug_restart(controls.reset)
    .hit_sound(assets.hit)
    .with_melee_combat()
    .with_enemy_chase()
    .with_collision()
    .with_camera_follow()
    .with_pause_death_ui()
    .build();
```

## Explanation

The asset handle is copied into the preset. The runtime loads the actual sound
file from `assets/sounds/hit.wav`.

`with_pause_death_ui` draws simple text when the game is paused or the player is
dead. `debug_toggle` lets the preset toggle `DebugOverlay`; by default the
top-down controls bind that to `F1`.

## Common errors

If `F1` does not show the overlay, check that the preset has
`.debug_toggle(controls.debug_overlay)`.

If `R` resets but also restarts the current map, that is expected in this simple
preset because both reset paths can share the same action while you are learning.

If the sound fails validation, the path should be `sounds/hit.wav`, not
`assets/sounds/hit.wav`.

## Next step

Animate the player in [07 - Add animation](07-add-animation.md).

# 05 - Add Combat

## Goal

Enable player attacks, enemy melee, and hit sounds through the top-down preset.

## What you will build

A gameplay preset that reads the attack action and runs the beginner melee combat
system every active tick.

## Files you will edit

`crates/simple-content/src/game.rs`

## Final code

```rust
game.use_top_down_game()
    .attack(controls.attack)
    .hit_sound(assets.hit)
    .with_melee_combat()
    .build();
```

## Explanation

`top_down_controls()` binds Space and Enter to `controls.attack`. The top-down
preset uses that action for the player's melee swing. Enemies with `.melee(...)`
can hit the player while combat is running.

`hit_sound` is optional. Combat still works without it, but passing a sound handle
makes successful hits easier to feel.

Most demos also enable movement, chase, collision, camera follow, pause UI, and
reset in the same chain.

## Common errors

If pressing Space does nothing, check that the preset includes
`.attack(controls.attack)`.

If no sound plays, check that the sound was registered with
`assets.sound("simple/hit", "sounds/hit.wav")?` and that the file exists under
`assets/sounds/`.

If enemies never reach the player, include `.with_enemy_chase()` and
`.with_collision()` in the same preset chain.

## Next step

Add the sound, reset, pause UI, and debug overlay in
[06 - Add sound and UI](06-add-sound-and-ui.md).

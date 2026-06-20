# 05 - Add Combat

## Goal

Enable player attacks and enemy melee through beginner rules.

## What you will build

Rules that read the attack action and run the beginner melee combat system every
active tick.

## Files you will edit

`crates/simple-content/src/game.rs`

## Final code

```rust
game.rules()
    .top_down_controls(controls)
    .enemies_damage_player()
    .dead_enemies_despawn()
    .camera_follows_player()
    .build();
```

## Explanation

`top_down_controls()` binds Space, Enter, mouse-left, and the first controller's
south face button to `controls.attack`. The rules builder uses that action for the player's melee swing. Enemies with
`.melee(...)` can hit the player while combat is running.

Combat works without a sound handle. The next tutorial swaps to the beginner
top-down preset when you want optional hit sounds and UI toggles.

Most demos also enable movement, chase, collision, camera follow, pause UI, and
reset in one beginner setup chain.

## Common errors

If pressing Space does nothing, check that the rules include
`.top_down_controls(controls)`.

If enemies take damage but never disappear, include `.dead_enemies_despawn()`.

If enemies never reach the player, include `.enemies_damage_player()` or switch
to the top-down preset with `.with_enemy_chase()` and `.with_collision()`.

## Next step

Add the sound, reset, pause UI, and debug overlay in
[06 - Add sound and UI](06-add-sound-and-ui.md).

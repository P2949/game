# 07 - Add Animation

## Goal

Use a sprite sheet and play idle/walk animations on an actor prefab.

## What you will build

A player sprite sheet with idle and walk clips that the top-down preset advances
and switches by movement.

## Files you will edit

`crates/simple-content/src/game.rs`

## Final code

Register a sprite sheet:

```rust
let assets = game
    .asset_bag()
    .spritesheet("player", "textures/test.png", 4, 1)?
    .texture("slime", "textures/test.png")?
    .texture("floor", "textures/test.png")?
    .texture("wall", "textures/test.png")?
    .sound("hit", "sounds/hit.wav")?
    .build();
```

Use the sheet in the player prefab:

```rust
game.player_prefab("player")
    .spritesheet(assets.spritesheet("player"))
    .idle(0..1)
    .walk(1..4)
    .moves_with(controls.movement, 130.0)
    .build()?;
```

Enable movement-driven switching in the beginner top-down preset:

```rust
game.use_top_down_game()
    .controls(controls)
    .with_player_animation_by_movement()
    .build();
```

## Explanation

`spritesheet` registers one texture and describes how many columns and rows of
frames it contains. `.idle(0..1)` plays frame 0 while still, and `.walk(1..4)`
plays frames 1, 2, and 3 while moving.

The top-down preset already calls the beginner animation updater, so actors with
animation clips advance automatically. `.with_player_animation_by_movement()`
switches the player between `idle` and `walk`.

## Common errors

If startup says the prefab has animations but uses a static texture, replace
`.sprite(...)` with `.spritesheet(...)`.

If the player never switches to `walk`, make sure the top-down preset includes
`.with_player_animation_by_movement()`.

If only one frame appears, check the sprite sheet column and row counts.

## Next step

Prepare the demo for another machine in
[08 - Package your demo](08-package-your-demo.md).

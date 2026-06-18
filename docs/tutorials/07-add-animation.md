# 07 - Add Animation

## Goal

Use a sprite sheet and play a looping animation on an actor prefab.

## What you will build

A player sprite sheet with an idle or walk clip that the top-down preset advances
each frame.

## Files you will edit

`crates/simple-content/src/game.rs`

## Final code

Register a sprite sheet:

```rust
#[derive(Clone, Copy, Debug)]
struct SimpleAssets {
    player: SpriteSheet,
    slime: TextureHandle,
    floor: TextureHandle,
    wall: TextureHandle,
    hit: SoundHandle,
}

fn register_assets(assets: &mut AssetAuthor<'_>) -> Result<SimpleAssets> {
    Ok(SimpleAssets {
        player: assets.spritesheet("simple/player", "textures/test.png", 4, 1)?,
        slime: assets.texture("simple/slime", "textures/test.png")?,
        floor: assets.texture("simple/floor", "textures/test.png")?,
        wall: assets.texture("simple/wall", "textures/test.png")?,
        hit: assets.sound("simple/hit", "sounds/hit.wav")?,
    })
}
```

Use the sheet in the player prefab:

```rust
game.player_prefab("player")
    .spritesheet(assets.player)
    .animation("walk", AnimationClip::frames(0..4).fps(8.0))
    .play("walk")
    .moves_with(controls.movement, 130.0)
    .build()?;
```

## Explanation

`spritesheet` registers one texture and describes how many columns and rows of
frames it contains. `AnimationClip::frames(0..4)` plays frames 0, 1, 2, and 3.

The top-down preset already calls the beginner animation updater, so actors with
`Animation` and `AnimationSet` advance automatically.

## Common errors

If startup says the prefab has animations but uses a static texture, replace
`.sprite(...)` with `.spritesheet(...)`.

If the animation name is missing, make sure `.play("walk")` matches the name in
`.animation("walk", ...)`.

If only one frame appears, check the sprite sheet column and row counts.

## Next step

Prepare the demo for another machine in
[08 - Package your demo](08-package-your-demo.md).

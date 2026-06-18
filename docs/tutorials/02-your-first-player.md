# 02 - Your First Player

## Goal

Create a controllable player with the beginner prefab builder.

## What you will build

A player prefab named `player` that has a sprite and moves with the standard
top-down controls.

## Files you will edit

`crates/simple-content/src/game.rs`

## Final code

```rust
game.player_prefab("player")
    .sprite(assets.player)
    .moves_with(controls.movement, 130.0)
    .build()?;
```

## Explanation

`game.player_prefab("player")` creates a reusable spawn recipe. The sprite comes
from the asset handles returned by `game.assets(...)`. The movement axis comes
from `game.input(|input| input.top_down_controls())?`.

The builder adds the beginner player pieces for you. Keep this first version
small: one sprite, one movement binding, one speed.

## Common errors

If the player prefab says it has no sprite, add `.sprite(assets.player)` before
`.build()?`.

If the player prefab says it has no movement axis, add
`.moves_with(controls.movement, 130.0)` before `.build()?`.

If the map cannot spawn the player later, make sure the prefab name in the map
matches the name here.

## Next step

Place the player in [03 - Add a map](03-add-a-map.md).

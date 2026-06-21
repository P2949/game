# 02 - Your First Player

## Goal

Create a controllable player with the beginner prefab builder.

## What you will build

A player prefab named `player` that has a sprite and moves with the standard
top-down controls.

## Files to edit

`crates/simple-content/src/game.rs`

## Full code

```rust
use game_starter::prelude::*;

fn main() -> Result<()> {
    run_game("My First Player", |game| {
        game.asset_bag()
            .texture("player", "textures/test.png")?
            .texture("floor", "textures/test.png")?
            .texture("wall", "textures/test.png")?
            .build();

        let controls = game.input(|input| input.top_down_controls())?;

        game.player_prefab("player")
            .sprite("player")
            .moves_with(controls.movement, 130.0)
            .build()?;

        game.map("first_room")
            .tiles(["#####", "#...#", "#.P.#", "#####"])
            .simple_theme("floor", "wall")
            .legend('P', "player")
            .start();

        game.rules()
            .top_down_controls(controls)
            .camera_follows_player()
            .build();

        Ok(())
    })
}
```

## What changed

`game.player_prefab("player")` creates a reusable spawn recipe. The sprite comes
from the name registered by `game.asset_bag()`. The movement axis comes from
`game.input(|input| input.top_down_controls())?`.

The builder adds the beginner player pieces for you. Keep this first version
small: one sprite, one movement binding, one speed.

## Common errors

If the player prefab says it has no sprite, add
`.sprite("player")` before `.build()?`.

If the player prefab says it has no movement axis, add
`.moves_with(controls.movement, 130.0)` before `.build()?`.

If the map cannot spawn the player later, make sure the prefab name in the map
matches the name here.

## Next step

Place the player in [03 - Add a map](03-add-a-map.md).

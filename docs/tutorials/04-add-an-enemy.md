# 04 - Add An Enemy

## Goal

Create a simple enemy prefab.

## What you will build

A `slime` enemy that has a sprite, chases the player, and can make melee hits.

## Files to edit

`crates/simple-content/src/game.rs`

## Full code

```rust
use game_starter::prelude::*;

fn main() -> Result<()> {
    run_game("First Enemy", |game| {
        game.asset_bag()
            .texture("player", "textures/test.png")?
            .texture("slime", "textures/test.png")?
            .texture("floor", "textures/test.png")?
            .texture("wall", "textures/test.png")?
            .build();

        let controls = game.input(|input| input.top_down_controls())?;

        game.player_prefab("player")
            .sprite("player")
            .moves_with(controls.movement, 130.0)
            .health(100)
            .build()?;

        game.enemy_prefab("slime")
            .sprite("slime")
            .chases_player()
            .melee(26.0, 6)
            .build()?;

        game.map("level_1")
            .tiles(["########", "#P...E.#", "#......#", "########"])
            .simple_theme("floor", "wall")
            .legend('P', "player")
            .legend('E', "slime")
            .start();

        game.rules()
            .top_down_controls(controls)
            .enemies_damage_player()
            .camera_follows_player()
            .build();

        Ok(())
    })
}
```

## What changed

`enemy_prefab` creates a reusable enemy spawn recipe. `.chases_player()` attaches
the beginner chase behavior. `.melee(26.0, 6)` means the enemy can hit in a
26-unit range for 6 damage.

The map still decides where enemies appear. Prefabs describe what an actor is;
`E` creates an instance because the map legend points at the slime prefab.

## Common errors

If the enemy does not appear, check the map spawn name and position.

If the enemy does not move, make sure the rules later include
`.enemies_damage_player()` or that the top-down preset includes
`.with_enemy_chase()`.

If the enemy cannot damage the player, make sure the combat rules are enabled in
the next tutorial.

## Next step

Add collectable coins in [05 - Add pickups and score](05-add-pickups-and-score.md).

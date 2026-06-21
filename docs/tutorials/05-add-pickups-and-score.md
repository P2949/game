# 05 - Add Pickups And Score

## Goal

Put coins on the map, collect them by touching them, and show a score without
writing a collision loop.

## Files to edit

Edit `src/main.rs`.

## Full code

```rust
use game_starter::prelude::*;

fn main() -> Result<()> {
    run_game("Coin Collector", |game| {
        game.asset_bag()
            .texture("player", "textures/test.png")?
            .texture("coin", "textures/test.png")?
            .texture("floor", "textures/test.png")?
            .texture("wall", "textures/test.png")?
            .sound("coin", "sounds/hit.wav")?
            .build();

        let controls = game.input(|input| input.top_down_controls())?;

        game.player_prefab("player")
            .sprite("player")
            .moves_with(controls.movement, 130.0)
            .build()?;

        game.pickup_prefab("coin")
            .sprite("coin")
            .score(1)
            .play_sound("coin")
            .despawn_on_collect()
            .build()?;

        game.map("coins")
            .tiles(["########", "#P.C..C#", "#..C...#", "########"])
            .simple_theme("floor", "wall")
            .legend('P', "player")
            .legend('C', "coin")
            .start();

        game.rules()
            .top_down_controls(controls)
            .player_collects_pickups()
            .camera_follows_player()
            .show_score()
            .build();

        game.on_player_collect_pickup(|game| {
            game.camera2d().shake(0.08);
        });

        Ok(())
    })
}
```

## What changed

`pickup_prefab` describes every coin once. `C` places a coin because its legend
points at that prefab. The pickup rule notices player/coin overlaps, adds the
configured score, plays the configured sound, and removes the coin. The last
callback is optional polish: it shakes the camera after each collection.

## Common errors

If a `C` reports an unknown prefab, check that both names are exactly `"coin"`.
If coins remain on the map after contact, include
`.player_collects_pickups()`. If no score appears, include `.show_score()`.

## Next step

Shoot enemies in [06 - Add projectiles](06-add-projectiles.md).

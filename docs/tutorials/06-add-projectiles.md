# 06 - Add Projectiles

## Goal

Fire a projectile toward the mouse or aiming direction with one cooldown
callback and a rule that handles hits.

## Files to edit

Edit `src/main.rs`.

## Full code

```rust
use game_starter::prelude::*;

fn main() -> Result<()> {
    run_game("Projectile Demo", |game| {
        game.asset_bag()
            .texture("player", "textures/test.png")?
            .texture("slime", "textures/test.png")?
            .texture("bolt", "textures/test.png")?
            .texture("floor", "textures/test.png")?
            .texture("wall", "textures/test.png")?
            .sound("shoot", "sounds/hit.wav")?
            .build();

        let controls = game.input(|input| input.top_down_controls())?;

        game.player_prefab("player")
            .sprite("player")
            .moves_with(controls.movement, 130.0)
            .build()?;

        game.enemy_prefab("slime")
            .sprite("slime")
            .health(30)
            .build()?;

        game.projectile_prefab("bolt")
            .sprite("bolt")
            .damage(15)
            .speed(260.0)
            .lifetime(0.8)
            .despawn_on_hit()
            .build()?;

        game.map("projectiles")
            .tiles(["########", "#P..E..#", "#......#", "########"])
            .simple_theme("floor", "wall")
            .legend('P', "player")
            .legend('E', "slime")
            .start();

        game.rules()
            .top_down_controls(controls)
            .projectiles()
            .dead_enemies_despawn()
            .camera_follows_player()
            .build();

        game.on_action_cooldown(controls.attack, 0.2, |game| {
            game.player().shoot("bolt").towards_mouse();
            game.audio().play_sound("shoot");
        });

        Ok(())
    })
}
```

## What changed

The projectile builder gives `bolt` damage, speed, a lifetime, and its hit
policy. `.projectiles()` moves bolts and applies damage. The callback says only
when to fire; `.on_action_cooldown` prevents a held attack button from creating
hundreds of bolts per second.

## Common errors

If the player never fires, confirm the callback uses `controls.attack`. If a
bolt appears but does not damage enemies, include `.projectiles()`. If a bolt
never disappears, set a `.lifetime(...)` or use `.despawn_on_hit()`.

## Next step

Add a locked exit and another map in
[07 - Add doors and levels](07-add-doors-and-levels.md).

# 11 - Custom behavior

## Goal

Make a bomber enemy that counts down from three seconds, then damages nearby
enemies and the player. You do not need to invent a Rust component or touch the
engine: give the prefab string tags and named data, then write a small tick rule.

## Files to edit

Edit `src/main.rs`. This chapter starts with the one-file game from the earlier
chapters; replace it with this complete version so every required asset and
rule is visible in one place.

## Full code

```rust
use game_starter::prelude::*;

fn main() -> Result<()> {
    run_game("Custom Bomber", |game| {
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
            .tag("enemy")
            .health(40)
            .build()?;

        game.enemy_prefab("bomber")
            .sprite("slime")
            .chases_player()
            .tag("enemy")
            .tag("explosive")
            .data("fuse", 3.0)
            .health(40)
            .build()?;

        game.map("level_1")
            .tiles(["########", "#PBE...#", "#......#", "########"])
            .simple_theme("floor", "wall")
            .legend('P', "player")
            .legend('B', "bomber")
            .legend('E', "slime")
            .start();

        game.rules()
            .top_down_controls(controls)
            .enemies_damage_player()
            .camera_follows_player()
            .show_player_health()
            .build();

        game.every_active_tick::<SimpleGameState>(|game, dt| {
            let mut explosions = Vec::new();
            game.actors_tagged("explosive").for_each(|actor| {
                let fuse = actor.data("fuse").unwrap_or(0.0) - dt;
                actor.set_data("fuse", fuse);
                if fuse <= 0.0 {
                    explosions.push(actor.position());
                }
            });

            for position in explosions.into_iter().flatten() {
                game.actors_tagged("enemy").near(position, 48.0).damage(20);
                game.player().damage_if_near(position, 48.0, 20);
            }
        });

        Ok(())
    })
}
```

## What changed

`.tag("explosive")` labels the bomber and `.data("fuse", 3.0)` gives it a
number to count down. `actors_tagged("explosive")` finds only bombers, and its
`for_each` callback gives each one a small actor-shaped handle. That handle can
read its position and data, update the fuse, damage it, or play an animation.

Tags are names you choose. A tag query only selects actors that you explicitly
tagged, so the ordinary slime also has `.tag("enemy")` before the explosion
rule asks for nearby enemies.

## Common errors

If a tag query finds zero actors, check spelling and make sure every prefab you
expect to match has the same `.tag("...")` call. If `data("fuse")` is empty,
make sure the prefab includes `.data("fuse", 3.0)` before `.build()?`.

An exploding bomber keeps triggering on later ticks in this small teaching
example. In a real game, add a second tag or value to mark it spent, or use a
separate event to remove it after the explosion.

## Next step

Try the [custom tags and timers cookbook recipe](../cookbook/custom-tags-and-timers.md)
for more patterns. This custom-behavior chapter is the bridge before you choose
to graduate to a content crate or the deliberately advanced ECS authoring path.

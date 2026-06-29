# Advanced When Needed

## Goal

Know when to stay with beginner tools and when a project is ready for the
advanced authoring surface.

## Files to edit

Usually none. This is a decision guide.

## Full code

For normal small games, keep this shape:

```rust
use game_starter::prelude::*;

run_game("My Game", |game| {
    let controls = game.input(|input| input.top_down_controls())?;

    game.assets_from_folders()
        .required_textures(["player", "slime", "floor", "wall"])?
        .build();

    game.player_prefab("player")
        .sprite("player")
        .moves_with(controls.movement, 130.0)
        .build()?;

    game.rules().top_down_controls(controls).build();

    Ok(())
})
```

## What changed

Nothing changes when the beginner API already describes what you want. Stay
with beginner Rust or `assets/game.ron` for:

- player, enemy, pickup, projectile, door, checkpoint, trigger, and spawner
  behavior
- score, health, scenes, maps, UI, sound, music, and animation
- custom behavior built from `on_*` hooks, actor handles, collections, and
  custom rules
- packaging, asset checking, data validation, and fast iteration

Use the advanced path only when the game idea truly needs lower-level engine
experiments or custom systems that the beginner vocabulary cannot express.

## Common errors

Do not switch to the advanced path just to add a new enemy, a projectile, a
door, score text, a pickup sound, or a menu. Those are beginner features.

Do not copy `testbed-content` for a first game. It is useful as a reference lab,
not as the starter shape.

## Next step

Use the [cookbook](../cookbook/README.md) for focused beginner recipes before
graduating. If you still need lower-level control, read
[When to use the advanced API](../when-to-use-advanced-api.md), then
[Advanced content authoring](../advanced-content-authoring.md).

## Advanced Path

Use the advanced API when you intentionally need custom ECS systems, manual
component composition, direct query-style logic, engine behavior experiments,
or tests and labs that inspect lower-level state.

Advanced content imports:

```rust
use game_kit::advanced::prelude::*;
```

Advanced callbacks may work with lower-level context and query concepts. Keep
those out of beginner templates, examples, and tutorials unless this boundary
has already been crossed.

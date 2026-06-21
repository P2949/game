# Beginner authoring

Start here when you want to make a small game rather than study engine internals.
Import `game_starter::prelude::*` for a standalone demo, or
`game_kit::beginner::prelude::*` in a content crate.

The usual order is:

1. Register named assets with `asset_bag()`.
2. Define player, enemy, pickup, door, projectile, and spawner prefabs.
3. Describe maps with `map(...)` or `map_from_text(...)`.
4. Compose behavior with `game.rules()`.
5. Add small custom reactions with `on_action`, `on_collect`, or scene hooks.

The [tutorials](tutorials/README.md) teach that path in order. The
[cookbook](cookbook/README.md) is for copying a focused feature. For the most
complete one-file example, see
[`script-like-custom-rules`](../examples/script-like-custom-rules/src/main.rs).

Use named assets in gameplay:

```rust
game.asset_bag()
    .texture_auto("player")?
    .sound_auto("shoot")?
    .build();

game.on_action_cooldown(controls.attack, 0.2, |game| {
    game.player().shoot("bolt").towards_mouse();
    game.audio().play_sound("shoot");
});
```

There is no need to name scheduling, command queues, entity ids, components, or
backend types on this path.

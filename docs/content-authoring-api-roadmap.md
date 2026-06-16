# Content Authoring API roadmap

The engine/content split is mechanically complete. The next target is to reduce
content crates to a friendly game-authoring facade so they do not operate
builder, schedule, registry, validation, or raw world plumbing directly.

## Target

Content crates (`arena-content`, `testbed-content`) should read like game
authoring code:

```rust
use game_kit::prelude::*;

impl GamePlugin for ArenaPlugin {
    fn build(&self, game: &mut GameApp) -> Result<()> {
        let assets = game.assets(assets::register)?;
        let controls = game.input(input::register)?;
        prefabs::register(game, assets, controls)?;
        game.map("arena").tile_size(32.0).tiles([..]).start();
        systems::register(game, assets, controls);
        Ok(())
    }
}
```

Content should express **assets, controls, prefabs, maps, and systems**. It must
not manually operate `GameBuilder`, `Schedule`, `PrefabRegistry`, `MapRegistry`,
`StartCtx`, raw `Ctx`, validators, or runtime internals.

## Final polish target

The remaining goal is to remove raw `World`, `Input`, `NavGrid`, and `TileMap`
access from production content systems. Content may use `GameCtx`, authoring
builders, and high-level `game-kit` helpers only.

Use these measurements while polishing the facade:

```bash
rg "World|Entity::new|ids_with|get::<|get_mut::<|world_and_|world_mut\(|world\(" \
  crates/arena-content/src crates/testbed-content/src

rg "game_kit::prelude::(movement_system|chase_system|patrol_system|apply_damage)" \
  crates/arena-content/src crates/testbed-content/src
```

The first command is complete when it reports no production usage outside
`#[cfg(test)]` test code.

## Layering

```text
bin/game                 selects plugin + runtime config
content crates           use game-kit only
game-kit                 friendly authoring facade over core/map/ai/combat/physics
engine-neutral crates    game-core, game-map, game-ai, game-combat, game-physics
runtime/backend crates   game-runtime, game-renderer-vulkan, game-platform-sdl, game-audio
```

Content never sees SDL, Vulkan, audio devices, swapchains, descriptor sets,
renderer texture ids, event pumps, or the fixed-timestep loop.

## Status

This roadmap is implemented by the `game-kit` facade. See
[content-authoring.md](content-authoring.md) for the author-facing guide and
`docs/ARCHITECTURE.md` for the layer diagram. The architecture-boundary tests in
`crates/game-core/tests/architecture_boundaries.rs` enforce that content crates
depend only on `game-kit` (plus `anyhow`/`glam`), never reach a backend, and do
not use raw ECS/world escape hatches in production code.

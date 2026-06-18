# Content Authoring API status and stabilization roadmap

## Status

The Content Authoring API 1.0 foundation is implemented.

Content crates depend only on `game-kit`; production content imports
`game_kit::prelude::*`; raw ECS/world access is restricted to tests via
`game_kit::testing::prelude::*`; systems use `GameCtx` helpers; prefabs use
tuple bundles; maps use `MapAuthor`; architecture tests enforce the boundary.

Remaining items are API polish and future feature decisions, not blockers for
the foundation milestone. Beginner-first authoring is tracked separately in
[beginner-authoring-roadmap.md](beginner-authoring-roadmap.md); that roadmap
builds on this facade instead of replacing it.

## What not to do next

Do not split more crates just for neatness.
Do not rewrite the renderer/runtime for this milestone.
Do not add scripting/editor support yet.
Do not replace the ECS/query model until real content pressure justifies it.

The next work should stabilize the current authoring API and add small examples.

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

## Stabilization checks

Production content should stay free of raw `World`, `Input`, `NavGrid`, and
`TileMap` access. Content may use `GameCtx`, authoring builders, and high-level
`game-kit` helpers only.

Use these measurements while stabilizing the facade:

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

## Cross-references

This roadmap is implemented by the `game-kit` facade. See
[content-authoring.md](content-authoring.md) for the current author-facing guide,
[beginner-authoring-roadmap.md](beginner-authoring-roadmap.md) for the next
beginner layer, and `docs/ARCHITECTURE.md` for the layer diagram. The
architecture-boundary tests in
`crates/game-core/tests/architecture_boundaries.rs` enforce that content crates
depend only on `game-kit` (plus `anyhow`/`glam`), never reach a backend, and do
not use raw ECS/world escape hatches in production code.

# Content Authoring API status and stabilization roadmap

> Status: Historical. The work described here has been implemented.
> Current release polish is tracked in `docs/beginner-productization-roadmap.md`.

## Status

The Content Authoring API 1.0 foundation is implemented.

Content crates depend only on `game-kit`. Beginner production content imports
`game_kit::beginner::prelude::*`; advanced content imports
`game_kit::advanced::prelude::*`; beginner tests use
`game_kit::beginner::testing::prelude::*`; raw ECS/world access is restricted to
advanced tests via `game_kit::advanced::testing::prelude::*`. Beginner prefabs
use game-object builders, maps use `MapAuthor`, rules use `game.rules()`, and
architecture tests enforce the boundary.

Remaining items are API polish and future feature decisions, not blockers for
the foundation milestone. Beginner-first authoring is summarized in
[beginner-authoring-roadmap.md](beginner-authoring-roadmap.md); that layer builds
on this facade instead of replacing it.

## What not to do next

Do not split more crates just for neatness.
Do not rewrite the renderer/runtime for this milestone.
Do not add scripting/editor support yet.
Do not replace the ECS/query model until real content pressure justifies it.

Future work should preserve the current split: beginner docs first, advanced ECS
facade available for content that needs it.

## Target

Beginner content should read like game authoring code:

```rust
use game_kit::beginner::prelude::*;

impl GamePlugin for ArenaPlugin {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        let assets = game
            .asset_bag()
            .texture("player", "textures/test.png")?
            .texture("floor", "textures/test.png")?
            .texture("wall", "textures/test.png")?
            .build();
        let controls = game.input(|input| input.top_down_controls())?;

        game.player_prefab("player")
            .sprite(assets.texture("player"))
            .moves_with(controls.movement, 130.0)
            .build()?;

        game.map("arena")
            .tiles(["###", "#P#", "###"])
            .simple_theme(assets.texture("floor"), assets.texture("wall"))
            .legend('P', "player")
            .start();

        game.rules().top_down_controls(controls).build();
        Ok(())
    }
}
```

Content should express **assets, controls, prefabs, maps, rules, and systems**.
It must not manually operate `GameBuilder`, `Schedule`, `PrefabRegistry`,
`MapRegistry`, `StartCtx`, raw `Ctx`, validators, or runtime internals.

## Stabilization checks

Beginner production content should stay free of raw `World`, `Input`, `NavGrid`,
and `TileMap` access. Content may use `GameCtx`, authoring builders, and
high-level `game-kit` helpers only.

Use these measurements while stabilizing the facade:

```bash
rg "World|Entity::new|ids_with|get::<|get_mut::<|world_and_|world_mut\(|world\(" \
  crates/simple-content/src crates/arena-content/src

rg "movement_system|chase_system|patrol_system|apply_damage" \
  crates/simple-content/src crates/arena-content/src
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

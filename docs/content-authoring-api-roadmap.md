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
        let assets = assets::register(game);
        let controls = input::register(game);
        prefabs::register(game, assets, controls);
        game.map("arena").tile_size(32.0).tiles([..]).start();
        systems::register(game, assets, controls);
        Ok(())
    }
}
```

Content should express **assets, controls, prefabs, maps, and systems**. It must
not manually operate `GameBuilder`, `Schedule`, `PrefabRegistry`, `MapRegistry`,
`StartCtx`, raw `Ctx`, validators, or runtime internals.

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
depend only on `game-kit` (plus `anyhow`/`glam`) and never reach a backend.

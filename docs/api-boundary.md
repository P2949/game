# API Boundary

This project keeps game authoring code separate from engine, runtime, and
backend code. Pick the narrowest path that fits the job.

## No-Rust data path

Use `assets/game.ron` plus asset files under `assets/`.

- Runtime, renderer, platform, audio, ECS, schedules, registries, and backend
  setup remain hidden.
- The data schema is versioned through `version: 1` today.
- Use `game-dev validate-data assets/game.ron` and `game-dev asset-check` to
  catch mistakes before running the game.

## Beginner Rust path

Standalone demos use:

```rust
use game_starter::prelude::*;
```

Workspace content crates use:

```rust
use game_kit::beginner::prelude::*;
```

Beginner docs, templates, and examples should use game-shaped vocabulary:
player, enemy, pickup, projectile, map, scene, sound, music, animation, score,
UI, rule, event, tag, and timer.

Beginner docs, templates, and examples should not teach or require these
engine-shaped concepts: `GameCtx`, `StartupGameCtx`, `EntityId`, `Component`,
`World`, `Transform`, `Velocity`, `Sprite::new`, `Collider::box_of`, raw
`Commands`, raw registries, runtime crates, or backend crates.

## Advanced content path

Advanced content uses:

```rust
use game_kit::advanced::prelude::*;
```

This path is for deliberate lower-level content: custom ECS-style systems,
queries, manual prefab composition, custom resources, and lower-level tests.

Advanced content still depends on `game-kit`. Content crates must not import
`game-core`, `game-map`, `game-runtime`, `game-renderer-vulkan`,
`game-platform-sdl`, or `game-audio` directly.

## Facade/internal path

`game-kit` is the authoring facade. It may use engine-neutral crates such as
`game-core`, `game-map`, `game-ai`, `game-combat`, and `game-physics`.

`game-kit` must not depend on runtime or backend crates.

## Runtime/backend path

`game-runtime` owns loop orchestration.

`game-renderer-vulkan`, `game-platform-sdl`, and `game-audio` own backend
complexity. Runtime and backend crates must not depend on content crates.

## Compatibility policy

`game_kit::prelude::*` is a deprecated compatibility surface for one migration
window. New code should choose `game_starter::prelude::*`,
`game_kit::beginner::prelude::*`, or `game_kit::advanced::prelude::*`.

Root reexports should be reduced after migration docs exist. When feasible,
beginner API renames keep a deprecated alias for one release.

# API Boundary

This project keeps game authoring code separate from engine, runtime, and
backend code. Pick the narrowest tier that fits the job.

## Authoring Tiers

### 1. Primary No-Rust Authoring

<!-- primary-no-rust:start -->
The Rust builder API is not the primary authoring surface. It is a secondary
tier. The primary surface is a plain data/config package that runs through the
prebuilt executable (`game-player`). A primary package has `game.toml` and
`assets/`; use `game-dev check`, `game-dev preview`, and `game-dev package` to
check, run, and share it without a generated wrapper project.
<!-- primary-no-rust:end -->

The target primary package is:

- `game.toml` or equivalent canonical config at the project root.
- Text maps, image/audio/font files, and optional Tiled/LDtk files under
  `assets/`.
- No `Cargo.toml`, no `src/main.rs`, and no `build.rs`.
- A prebuilt runner and CLI for check, preview, and package commands.
- Runtime, renderer, platform, audio, ECS, schedules, registries, and backend
  setup remain hidden.
- Users should not need Cargo, rustc, cargo-generate, Rust syntax, or a Rust
  wrapper to edit and preview the primary package.

Status: the foundation pieces are implemented and the roadmap is now hardening
enforcement, migration, release packaging, and SDK verification.

### 2. Secondary Beginner Rust Authoring

Use this tier only when you want to write Rust.

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

The beginner Rust API stays supported, but it is not proof that the primary
no-Rust objective is complete.

### 3. Advanced Rust Authoring

Use this tier only for deliberate lower-level Rust authoring.

Advanced Rust authoring is not the primary no-Rust surface.

Advanced content uses:

```rust
use game_kit::advanced::prelude::*;
```

This path is for deliberate lower-level content: custom ECS-style systems,
queries, manual prefab composition, custom resources, and lower-level tests.

Advanced content still depends on `game-kit`. Content crates must not import
`game-core`, `game-map`, `game-runtime`, `game-renderer-vulkan`,
`game-platform-sdl`, or `game-audio` directly.

## Legacy Data Path

`assets/game.ron` exists today as a legacy/transitional data format. It may be
validated with `game-dev validate-data assets/game.ron` and checked with
`game-dev asset-check`, and old projects can migrate with
`game-dev migrate-ron assets/game.ron --out game.toml`. Public start-here
material should describe RON as legacy, migration, advanced, internal fixture,
or historical roadmap material, not as the official primary no-Rust target.
`game.load_beginner_file("game.ron")` is a legacy compatibility helper; when a
Rust wrapper is temporarily needed to exercise a primary package, use
`game.load_authoring_file("game.toml")`.

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

Compatibility removal plan:

- v0.2.x: compatibility prelude exists but deprecated.
- v0.3.x: docs/examples/templates must not use it.
- v0.4.x or pre-1.0: remove or feature-gate compatibility prelude.

Root reexports should be reduced after migration docs exist. When feasible,
beginner API renames keep a deprecated alias for one release.

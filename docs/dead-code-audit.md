# Public API / Future Surface Audit

This workspace now contains several library crates, so the old binary-crate
`#[allow(dead_code)]` audit is no longer the right framing. This document tracks
public or compatibility surface that is intentionally present but not yet a
fully stable engine API.

Categories:

- **A** — intentional API or accessor kept for diagnostics, tests, or symmetry.
- **B** — compatibility shim kept only during the engine/content split.
- **C** — planned feature surface that must either be implemented or removed
  before the API is treated as stable.

No item currently falls in a "remove immediately" bucket.

## Core API

| Item | Cat | Reason | Revisit when |
| ---- | --- | ------ | ------------ |
| `game_core::backend::{RenderBackend, AudioBackend, PlatformBackend}` | C | Explicitly future-facing backend traits; runtime still wires directly to SDL/audio/Vulkan crates and `game-kit` does not expose them to content. | A headless test backend, second renderer, or trait-driven runtime is needed. |
| `TileMap::from_rows` | A | Lenient constructor for trusted/internal rows and empty defaults. Authoring paths should use `try_from_rows` or `MapBuilder::try_tile_layer`. | A future misuse suggests renaming it to `from_rows_lenient`. |

## Compatibility Shims

No split-era compatibility modules remain. The old `game_core::engine`,
`arena_content::{engine, game}`, `game_renderer_vulkan::renderer`, and
`game_platform_sdl::platform` re-export shims have been removed.

`game_core::prelude` is now intentionally small. The former broad set of raw
builder/schedule/context/registry exports lives under `game_core::internal_prelude`
for runtime/facade/tests. Content crates use `game_kit::prelude::*`.

`game-kit` keeps beginner harness assertions in
`game_kit::beginner::testing::prelude::*` and raw world inspection in
`game_kit::advanced::testing::prelude::*`; the normal prelude exposes authoring
builders, component types, and `GameCtx` helpers only. `MapAuthor` currently
exposes only `.start()`;
additional registered maps and runtime map switching are future work.

## Runtime Reality Checks

- The runtime validates content asset registrations and renderer built-in assets
  before backend startup; the font atlas image is still built during renderer
  creation after that preflight.
- File-backed sound requests exist in `game-core` but are not exposed through
  `game-kit` until runtime playback exists.
- Query order is deterministic for `World::ids_with` / `query` / `query2`; code
  should not depend on `HashMap` iteration order.

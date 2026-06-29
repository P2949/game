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
| `game_core::backend::{RenderBackend, AudioBackend, PlatformBackend}` | A | The trait-driven `Runner` uses these contracts for the production SDL/Vulkan/audio path and the in-memory `game-backend-headless` test path. They remain runtime-internal, not content API. | A second production backend or a public embedding API changes their stability requirements. |
| `TileMap::from_rows` | A | Lenient constructor for trusted/internal rows and empty defaults. Authoring paths should use `try_from_rows` or `MapBuilder::try_tile_layer`. | A future misuse suggests renaming it to `from_rows_lenient`. |

## Compatibility Shims

No split-era compatibility modules remain. The old `game_core::engine`,
`arena_content::{engine, game}`, `game_renderer_vulkan::renderer`, and
`game_platform_sdl::platform` re-export shims have been removed.

`game_core::prelude` is now intentionally small. The former broad set of raw
builder/schedule/context/registry exports lives under `game_core::internal_prelude`
for runtime/facade/tests. Beginner content uses
`game_kit::beginner::prelude::*`; `game_kit::prelude::*` is compatibility-only
and should not appear in new beginner code.

`game-kit` keeps beginner harness assertions in
`game_kit::beginner::testing::prelude::*` and raw world inspection in
`game_kit::advanced::testing::prelude::*`; the normal prelude exposes authoring
builders, component types, and `GameCtx` helpers only. `MapAuthor` exposes
`.start()` for the initial map and `.finish()` for additional registered maps.
Runtime map switching is available through beginner door/map helpers and
lower-level commands.

Beginner CLI, data-file DSL, LDtk/Tiled import paths, runtime map switching,
file-backed sounds, generated-project packaging, and release packaging are
implemented surfaces now. Treat polish around those areas as release-readiness
work, not as unimplemented architecture.

## Runtime Reality Checks

- The runtime validates content asset registrations and renderer built-in assets
  before backend startup; the font atlas image is still built during renderer
  creation after that preflight.
- `game-runtime::Runner` is generic over platform, renderer, and audio
  backends. `game-backend-headless` records input, frames, reloads, and audio
  commands so the full content loop is tested without SDL, Vulkan, or an audio
  device; the production binary still uses the concrete SDL/Vulkan/audio
  implementations.
- File-backed WAV/OGG/MP3 sound loading and runtime playback are available
  through `game-kit` where the relevant decoder features are enabled. Use
  `AssetBagAuthor::sound`/`::music` (or the folder authoring equivalents) and
  `game.audio()`.
- Query order is deterministic for `World::ids_with` / `query` / `query2`; code
  should not depend on `HashMap` iteration order.

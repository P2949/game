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
| `game_core::backend::{RenderBackend, AudioBackend, PlatformBackend}` | C | Documents the intended backend boundary, but runtime still wires directly to SDL/audio/Vulkan crates. | Runtime actually depends on these traits, or the traits move to an explicit future-facing module. |
| `Game` / `GamePlugin::Game` direct update path | C | Kept as a fallback and for tests while content is moving to schedules. | Runtime is schedule-only; then remove or minimize `Game::update`. |
| `AudioCommand::PlayMusic` / `StopMusic` | C | Represents intended audio commands, but runtime currently maps all playback to generated blips. | Real sound/music loading lands, or the commands are removed until then. |
| `TileMap::from_rows` | A | Lenient constructor for trusted/internal rows and empty defaults. Authoring paths should use `try_from_rows` or `MapBuilder::try_tile_layer`. | A future misuse suggests renaming it to `from_rows_lenient`. |

## Compatibility Shims

| Item | Cat | Reason | Remove when |
| ---- | --- | ------ | ----------- |
| `game_core::engine` | B | Re-exports old pre-workspace `engine::` paths. | No crate imports `game_core::engine::*`. |
| `arena_content::{engine, game}` | B | Lets arena content keep old local import paths temporarily. | Arena imports match `testbed-content`'s direct `game_core::*` style. |
| `game_renderer_vulkan::renderer` | B | Keeps old renderer module paths during the physical crate split. | Internal renderer paths use crate-root modules directly. |
| `game_platform_sdl::platform` | B | Keeps old platform module paths during the physical crate split. | Internal/user imports use crate-root modules directly. |

## Runtime Reality Checks

- The renderer validates content asset registrations before backend startup, but
  the built-in UI font is still loaded during renderer creation.
- Sound registrations produce handles, but runtime playback is currently
  generated-only.
- Query order is deterministic for `World::ids_with` / `query` / `query2`; code
  should not depend on `HashMap` iteration order.

# Architectural Improvement Roadmap

> Status: Historical. The work described here has been implemented.
> Current release polish is tracked in `docs/beginner-productization-roadmap.md`.

This document tracks the implementation status of the architectural improvement
roadmap. Items are marked implemented, already satisfied by prior hardening, or
deliberately deferred with a rationale.

## Status summary

- **Phase 1 — release/runtime correctness:** README release instructions
  corrected; CI runs a release-mode headless smoke test and an MSRV gate. The
  declared `rust-version` was corrected to 1.87 (the real floor, set by `sdl3`),
  and let-chains were removed so the code meets it.
- **Phase 2 — small correctness hardening:** fixed-timestep consumption is
  fallible; swapchain image/view lookups and pipeline creation return errors
  instead of indexing; the SDL surface raw-handle safety contract is documented.
- **Phase 3 — renderer resource lifetime:** the dynamic sprite buffer is
  reallocated before the old one is released; texture extent is validated before
  decode. The shared descriptor pool and generation-based `TextureId` remain
  deferred per guiding principle 4 (upgrade architecture only when a feature
  needs it).
- **Phase 4 — render scalability:** device-selection diagnostics completed; the
  rest (indexed quads, layout tracking, non-coherent memory) is deferred as "not
  urgent" per the roadmap.
- **Phase 5 — gameplay correctness:** pause freezes simulation and effects;
  collision remains discrete axis-separated AABB movement with wall sliding.
  Swept collision/depenetration are still planned, and fast movement can tunnel
  through thin solids. Map authoring has moved toward strict validation through
  `game-map`.
- **Phase 6 — audio robustness:** a voice-drop counter and validated generated
  tone playback are present. Loading registered sound files, seamless looping
  music, and resampling stay deferred by design.
- **Phase 7 — text/UI:** ASCII-only limitation documented; dynamic glyph/shaping
  work deferred.
- **Phase 8 — cleanup/docs:** `#[allow(dead_code)]` audit
  ([`dead-code-audit.md`](dead-code-audit.md)), renderer ownership docs
  ([`ARCHITECTURE.md`](ARCHITECTURE.md), [`renderer-lifetime.md`](renderer-lifetime.md)),
  and a [`release-checklist.md`](release-checklist.md).
- **Phase 9 — README polish:** project-status/scope section added. A
  screenshot/GIF still needs a manual capture from a real display.

## Follow-ups after the engine/content workspace split

The split landed with deliberately temporary compatibility shims so the
mechanical crate move did not also have to rewrite every `use` path in one go.
Those shims have now been removed:

- `game_core::engine`
- `arena_content::{engine, game}`
- `game_renderer_vulkan::renderer`
- `game_platform_sdl::platform`

The `architecture_boundaries` integration test records the cross-crate import
gates that must continue to hold afterward.

## Current stabilization targets

- Current roadmap: [Content Authoring API 0.1](content-authoring-api-roadmap.md).
  The workspace/content split is mechanically complete; content crates now use
  `game-kit` as the authoring facade and do not operate engine wiring directly.
- The main architecture split is complete. Remaining beginner-facing work is
  data-driven parity, authoring polish, validation, templates, docs, and runtime
  proof around the no-Rust path.
- Keep CI commands workspace/package-qualified so the virtual workspace does not
  rely on whichever package Cargo happens to infer.
- Keep runtime/content tests on the schedule path now that the direct
  `Game::update` fallback has been removed.
- Keep command APIs honest: only expose commands the runtime actually consumes,
  or carry the necessary registries into runtime before adding map/prefab/event
  commands.
- Tighten duplicate-name validation in registries before more content is added.

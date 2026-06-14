# Architectural Improvement Roadmap

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
  `Entity::try_set_position` added; depenetration and swept-AABB movement added,
  with gameplay switched to swept collision. The data-driven level format is
  deferred (would add a serialization dependency; collision is now strong enough
  to unblock it later).
- **Phase 6 — audio robustness:** seamless looping music, a voice-drop counter,
  and validated tone generation. Resampling stays deferred by design.
- **Phase 7 — text/UI:** ASCII-only limitation documented; dynamic glyph/shaping
  work deferred.
- **Phase 8 — cleanup/docs:** `#[allow(dead_code)]` audit
  ([`dead-code-audit.md`](dead-code-audit.md)), renderer ownership docs
  ([`ARCHITECTURE.md`](ARCHITECTURE.md), [`renderer-lifetime.md`](renderer-lifetime.md)),
  and a [`release-checklist.md`](release-checklist.md).
- **Phase 9 — README polish:** project-status/scope section added. A
  screenshot/GIF still needs a manual capture from a real display.

# Beginner Productization 1.0 Roadmap

This checklist tracks the remaining work after the core engine/content
architecture split. The architecture is complete enough for beginner authoring;
the productization work is about making the beginner layer easier to start,
harder to misuse, and friendlier when something goes wrong.

## Current State

- Architecture status: complete.
- Beginner Productization 1.0 status: complete for `v0.2.0`. The local
  release gate passed, the `v0.2.0` tag was pushed, GitHub Actions release run
  `28523446249` attached Linux/Windows demo zips to the GitHub Release, and
  `scripts/verify-github-release-artifacts.sh 28523446249` verified them.
- Beginner entry points:
  - `game_starter::prelude::*` for standalone beginner projects.
  - `game_kit::beginner::prelude::*` for beginner content crates.
  - `game_kit::advanced::prelude::*` for advanced systems and testbed content.
- Advanced content remains intentionally separate; `testbed-content` is an
  advanced lab and should not be the first thing beginners copy.
- Verified release artifacts:
  - `game-demo-linux-x86_64.zip`
  - `game-demo-windows-x86_64.zip`

## Beginner Productization 1.0 Acceptance Criteria

- A new project can be generated outside the engine workspace.
- The generated project runs with one documented command.
- The generated project includes starter assets and a text map.
- Small demos can be authored through beginner APIs only.
- No-Rust demos can be authored through `assets/game.ron`.
- Common custom behavior uses `on_*` hooks and custom-rule builders.
- `game.ron` validation is friendly, complete, and names valid options.
- Fast iteration covers text maps, tuning, assets, and partial data-file reload.
- Packaging produces a shareable folder or zip.
- Beginner docs follow one clear learning path.
- Architecture tests keep beginner examples/templates/docs off advanced/ECS APIs.
- Advanced APIs remain available, but only in advanced docs/examples.

## API Boundary Rules

Beginner-facing code should use game vocabulary:

```text
player, enemy, map, pickup, projectile, door, scene, score, sound, UI, rule, event
```

Beginner-facing code should not expose engine vocabulary:

```text
GameCtx, StartupGameCtx, EntityId, Component, raw ECS traversal,
renderer/runtime/backend types, commands/resources internals
```

## Phase Status

| Phase | Status | Current note |
| --- | --- | --- |
| Phase 0: Baseline verification | Done | Core cargo checks, CLI checks, generated-project checks, packaging, and graphical smoke pass when run through the documented Xvfb/lavapipe path. |
| Phase 1: Beginner-only context wrappers | Done | `beginner::Game` and `StartupGame` exist. |
| Phase 2: Import surfaces | Done | Beginner/advanced preludes exist; compatibility prelude is deprecated. |
| Phase 3: Generated-project CI | Done | CI checks templates outside the workspace. |
| Phase 4: Standalone CLI | Done | `game-dev` exists. |
| Phase 5: Doctor diagnostics | Done / polish | `doctor --explain` and `game-dev check` exist. |
| Phase 6: Packaging | Done | `game-dev package --zip` creates shareable packages. |
| Phase 7: Data reload | Partial by design | Existing values can reload; structural list changes require restart. |
| Phase 8: Script-like events/rules | Done / expandable | Hooks, events, structured rules, and custom-rule builders exist. |
| Phase 9: Diagnostics | Mostly done | Known-name checks and countdown key validation exist; keep expanding messages as users hit new mistakes. |
| Phase 10: Starter assets | Done / polish | Templates generate starter assets and maps. |
| Phase 11: Tutorial path | Done / polish | Keep no-Rust, beginner Rust, Tiled, and advanced paths clearly separated. |
| Phase 12: Data DSL parity | Mostly done | Structured conditions/effects exist; expand only as examples need it. |
| Phase 13: Packaging docs | Done / verify | Package flow exists; continue verifying on release targets. |
| Phase 14: Prebuilt artifacts | Done | GitHub Actions release run `28523446249` attached verified Linux/Windows demo zips to the `v0.2.0` GitHub Release. |
| Phase 15: Stability/migrations | Initial done | CHANGELOG and migration docs exist; update per release. |
| Phase 16: Advanced separation | Done | `testbed-content` remains advanced. |
| Phase 17: First-15-minutes test | Done | Script exists and CI calls it. |
| Phase 18: Final gate | Done | Local release gates passed; `v0.2.0` release artifacts were attached and verified. |

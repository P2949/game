# Beginner Productization 1.0 Roadmap

This checklist tracks the remaining work after the core engine/content
architecture split. The architecture is complete enough for beginner authoring;
the productization work is about making the beginner layer easier to start,
harder to misuse, and friendlier when something goes wrong.

## Current State

- Architecture status: complete.
- Productization status: in progress.
- Beginner entry points:
  - `game_starter::prelude::*` for standalone beginner projects.
  - `game_kit::beginner::prelude::*` for beginner content crates.
  - `game_kit::advanced::prelude::*` for advanced systems and testbed content.
- Advanced content remains intentionally separate; `testbed-content` is an
  advanced lab and should not be the first thing beginners copy.

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

## Phase Checklist

- Phase 0: Freeze current baseline and add this productization checklist.
- Phase 1: Replace beginner context aliases with beginner-only wrapper types.
- Phase 2: Stabilize public import surfaces.
- Phase 3: Validate generated projects outside the workspace in CI.
- Phase 4: Add a standalone `game-dev` beginner CLI.
- Phase 5: Improve first-run setup and doctor diagnostics.
- Phase 6: Package generated projects.
- Phase 7: Improve no-Rust data validation and reload status.
- Phase 8: Expand script-like events and rules.
- Phase 9: Make beginner diagnostics systematic and teaching-oriented.
- Phase 10: Polish starter assets and asset workflow.
- Phase 11: Curate the beginner learning path.
- Phase 12: Grow the data-file DSL toward beginner Rust parity.
- Phase 13: Make packaging and distribution beginner-obvious.
- Phase 14: Add prebuilt demo artifacts.
- Phase 15: Document beginner API stability and migrations.
- Phase 16: Keep advanced content useful and clearly separate.
- Phase 17: Add a first-15-minutes acceptance test.
- Phase 18: Run the final beginner-friendliness gate.

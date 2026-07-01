# Post-1.0 API Surface Cleanup Roadmap

> Status: Superseded by [Content/engine boundary consolidation](content-engine-boundary-consolidation.md).

Root-level `game-kit` exports are compatibility surface for the current release
window. Do not remove them during 1.0 polish.

## Goals

- Audit root-level `game-kit` exports after the first tagged beginner release.
- Deprecate advanced root exports that should live only under
  `game_kit::advanced::prelude::*`.
- Require beginner and advanced import surfaces in docs:
  `game_kit::beginner::prelude::*`,
  `game_kit::advanced::prelude::*`, and `game_starter::prelude::*`.
- Update migration docs before removing or hiding compatibility exports.
- Keep a compatibility window for one release if possible.

## Non-Goals For 1.0

- Removing `game_kit` root exports.
- Removing the deprecated compatibility prelude.
- Changing beginner examples, templates, or tutorials away from their current
  beginner-first import surfaces.

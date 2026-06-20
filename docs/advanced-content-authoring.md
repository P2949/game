# Advanced content authoring

This document is intentionally second, not the starting point. Use the
[beginner authoring guide](beginner-authoring.md) unless you specifically need
custom ECS-shaped prefabs, systems, queries, RON maps, or engine pressure tests.

Advanced content imports `game_kit::advanced::prelude::*`. The repository's
`testbed-content` crate uses that surface on purpose: it is an engine testbed,
not a template for first projects.

When an advanced game needs a recurring beginner-friendly pattern, prefer adding
one focused `game-kit` builder or rule instead of copying the low-level plumbing
into every new game.

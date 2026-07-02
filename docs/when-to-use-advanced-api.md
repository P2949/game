# When to use the advanced API

Most demos should stay with the primary `game.toml` package or the secondary
beginner Rust API. Use the advanced API only when the game idea needs
lower-level control that the beginner vocabulary cannot express.
Advanced Rust authoring is not the primary no-Rust surface.
For the exact public API boundary, see [api-boundary.md](api-boundary.md).

## Stay beginner for normal demos

Use a primary `game.toml` package, `game_starter::prelude::*`, or
`game_kit::beginner::prelude::*` for:

- players, enemies, pickups, doors, checkpoints, triggers, and projectiles
- maps, scenes, score, health, UI, sound, music, and animation
- simple custom behavior through hooks, rules, actor handles, and collections
- packaging, asset checks, data validation, and fast iteration

Changing from one enemy type to three, adding a pickup sound, wiring a title
menu, or spawning waves is still beginner work.

## Cross the boundary deliberately

Use `game_kit::advanced::prelude::*` when you intentionally need:

- custom ECS systems
- manual component composition
- direct query-style logic
- custom resources or engine-facing tests
- low-level experiments that validate runtime behavior

Advanced systems may use `GameCtx`, typed queries, components, and ECS-style
state because those are the point of the advanced path. Keep those concepts out of beginner templates, examples, and tutorials.

## What to copy

Copy `templates/no-rust-demo` or one of the `examples/no-rust-*` packages first
when you do not want Rust. Copy `templates/simple-demo`,
`examples/one-file-demo`, `examples/no-rust-shapes-demo`,
`examples/script-like-custom-rules`, or `simple-content` when you want the
secondary beginner Rust path.

Use `templates/data-driven-demo` only for legacy RON compatibility or migration
work.

Do not copy `testbed-content` for a first game. It is an advanced lab for
manual systems, advanced RON maps, tuple prefabs, and lower-level content experiments.
It should remain useful, but visibly separate from the beginner path.

When the beginner API almost works but one common feature is missing, prefer
adding a small beginner builder or rule over moving the whole project to
advanced code.

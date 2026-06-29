# Tutorials

These tutorials are the guided beginner path. They use
`game_starter::prelude::*` and avoid engine architecture, ECS, lifetimes, and
workspace-crate theory until you choose to grow into a content crate.

```rust
use game_starter::prelude::*;
```

`testbed-content` is intentionally an advanced lab for manual systems and RON
maps. Keep following the examples below or copy `simple-content` rather than
using it as a beginner starting point.

## Tracks

- **Track A: No-Rust data file.** Start with
  [Data-driven first game](13-data-driven-demo.md) when you want to edit
  `assets/game.ron`, maps, textures, and sounds before writing Rust.
- **Track B: Beginner Rust.** Follow the numbered course below when you want a
  one-file Rust game with beginner builder chains.
- **Track C: Cookbook recipes.** Use the [cookbook](../cookbook/README.md)
  after the course when you need one focused feature.
- **Track D: Advanced API.** Read
  [Advanced when needed](advanced-when-needed.md) only after beginner hooks,
  rules, and data files no longer describe the thing you need.

## Which Example Should I Copy?

- **No Rust:** `templates/data-driven-demo`,
  `examples/data-driven-events-demo`, `examples/data-driven-waves-demo`,
  `examples/data-driven-projectiles-demo`
- **First Rust game:** `templates/simple-demo`
- **One-file example:** `examples/one-file-demo`
- **Full beginner feature sample:** `examples/no-rust-shapes-demo`
- **Custom behavior:** `examples/script-like-custom-rules`
- **Do not copy first:** `testbed-content`

## Beginner Rust Course

0. [Start here](00-start-here.md)
1. [Rust you need](rust-you-need.md)
2. [Run the demo](01-run-the-demo.md)
3. [Your first player](02-your-first-player.md)
4. [Add a map](03-add-a-map.md)
5. [Add an enemy](04-add-an-enemy.md)
6. [Add pickups and score](05-add-pickups-and-score.md)
7. [Add projectiles](06-add-projectiles.md)
8. [Add doors and levels](07-add-doors-and-levels.md)
9. [Add sound and music](08-add-sound-and-music.md)
10. [Add UI and menu](09-add-ui-and-menu.md)
11. [Package your demo](10-package-your-demo.md)
12. [Custom behavior](11-custom-behavior.md)
13. [Fast iteration](12-fast-iteration.md)

Need the shortest possible first run? Use the
[quickstart](quickstart-zero-to-demo.md), then come back to the course.

## Optional Follow-Ups

These older pages are kept as focused follow-up notes, not as part of the main
numbered course:

- [Optional - Add combat](optional-add-combat.md)
- [Optional - Add sound and UI](optional-add-sound-and-ui.md)
- [Optional - Add animation](optional-add-animation.md)
- [Optional - Package your demo](optional-package-your-demo.md)

Keep [common errors](common-errors.md) nearby while editing. The messages are
written to point back to the builder call that fixes the problem.

Every main course chapter includes a goal, files to edit, full code, an
explanation, common errors, and a next step. The custom-behavior chapter is the
bridge before you choose to graduate to a content crate or the deliberately
advanced path.

# Tutorials

These tutorials build a one-file beginner game in order. They use
`game_starter::prelude::*` and avoid engine architecture, ECS, lifetimes, and
workspace-crate theory until you choose to grow into a content crate.

```rust
use game_starter::prelude::*;
```

`testbed-content` is intentionally an advanced lab for manual systems and RON
maps. Keep following the examples below or copy `simple-content` rather than
using it as a beginner starting point.

0. [Zero to a running demo](00-zero-to-demo.md)
1. [Rust you need](rust-you-need.md)
2. [Start here](00-start-here.md)
3. [Run the demo](01-run-the-demo.md)
4. [Your first player](02-your-first-player.md)
5. [Add a map](03-add-a-map.md)
6. [Add an enemy](04-add-an-enemy.md)
7. [Add pickups and score](05-add-pickups-and-score.md)
8. [Add projectiles](06-add-projectiles.md)
9. [Add doors and levels](07-add-doors-and-levels.md)
10. [Add sound and music](08-add-sound-and-music.md)
11. [Add UI and menu](09-add-ui-and-menu.md)
12. [Package your demo](10-package-your-demo.md)
13. [Custom behavior](11-custom-behavior.md)
14. [Fast iteration](12-fast-iteration.md)
15. [Data-driven first game](13-data-driven-demo.md)

Choose the no-Rust data-driven path when you want to edit `assets/game.ron` and
text maps. Choose beginner Rust builder chains when you want custom behavior in
`src/main.rs`. Advanced `testbed-content` remains available for lower-level
engine-shaped experiments, but it is not the first path.

Keep [common errors](common-errors.md) nearby while editing. The messages are
written to point back to the builder call that fixes the problem.

Every chapter includes a goal, files to edit, full code, an explanation, common
errors, and a next step. The custom-behavior chapter is the bridge before you
choose to graduate to a content crate or the deliberately advanced ECS path.
For focused "How do I add X?" recipes, use the [cookbook](../cookbook/README.md).
The older combat and animation pages remain as optional follow-up recipes once
you have finished this course.

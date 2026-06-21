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

0. [Start here](00-start-here.md)
1. [Run the demo](01-run-the-demo.md)
2. [Your first player](02-your-first-player.md)
3. [Add a map](03-add-a-map.md)
4. [Add an enemy](04-add-an-enemy.md)
5. [Add pickups and score](05-add-pickups-and-score.md)
6. [Add projectiles](06-add-projectiles.md)
7. [Add doors and levels](07-add-doors-and-levels.md)
8. [Add sound and music](08-add-sound-and-music.md)
9. [Add UI and menu](09-add-ui-and-menu.md)
10. [Package your demo](10-package-your-demo.md)
11. [Fast iteration](11-fast-iteration.md)

Keep [common errors](common-errors.md) nearby while editing. The messages are
written to point back to the builder call that fixes the problem.

Every chapter includes a goal, files to edit, full code, an explanation, common
errors, and a next step. For focused "How do I add X?" recipes, use the
[cookbook](../cookbook/README.md). The older combat and animation pages remain
as optional follow-up recipes once you have finished this course.

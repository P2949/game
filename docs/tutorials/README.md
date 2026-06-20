# Tutorials

These tutorials build the beginner path in order. They start from the tiny
`simple-content` demo and stay on `game_kit::beginner::prelude::*` APIs so you
can make a playable top-down game without reading the engine architecture first.
Snippet examples assume this import unless the page says it is a standalone
generated project:

```rust
use game_kit::beginner::prelude::*;
```

`testbed-content` is intentionally an advanced lab for manual systems and RON
maps. Keep following the examples below or copy `simple-content` rather than
using it as a beginner starting point.

1. [Run the demo](01-run-the-demo.md)
2. [Your first player](02-your-first-player.md)
3. [Add a map](03-add-a-map.md)
4. [Add an enemy](04-add-an-enemy.md)
5. [Add combat](05-add-combat.md)
6. [Add sound and UI](06-add-sound-and-ui.md)
7. [Add animation](07-add-animation.md)
8. [Package your demo](08-package-your-demo.md)

Keep [common errors](common-errors.md) nearby while editing. The messages are
written to point back to the builder call that fixes the problem.

For focused "How do I add X?" recipes, use the [cookbook](../cookbook/README.md).

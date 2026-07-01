# Root API Cleanup

`game-kit` now keeps broad authoring exports out of the crate root. Choose the
explicit surface that matches your content.

Old compatibility import:

```rust
use game_kit::prelude::*;
```

New beginner content crate import:

```rust
use game_kit::beginner::prelude::*;
```

New standalone demo import:

```rust
use game_starter::prelude::*;
```

New advanced content import:

```rust
use game_kit::advanced::prelude::*;
```

Old root plugin helper:

```rust
pub fn plugin() -> game_kit::Plugin<MyPlugin> {
    game_kit::plugin(MyPlugin)
}
```

New advanced prelude helper:

```rust
use game_kit::advanced::prelude::{Plugin, plugin as kit_plugin};

pub fn plugin() -> Plugin<MyPlugin> {
    kit_plugin(MyPlugin)
}
```

Or use explicit module paths directly:

```rust
pub fn plugin() -> game_kit::app::Plugin<MyPlugin> {
    game_kit::app::plugin(MyPlugin)
}
```

For one compatibility window, the old broad root surface is available under:

```rust
use game_kit::compat::*;
```

Prefer this only as a temporary bridge while migrating to beginner, advanced,
or explicit module imports.

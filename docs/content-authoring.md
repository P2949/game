# Content Authoring

Most games should begin with the [beginner authoring guide](beginner-authoring.md).
It uses names such as `"player"`, `"slime"`, and `"hit"` to describe a game
without exposing engine plumbing.
For the exact layer and import contract, see [api-boundary.md](api-boundary.md).

<!-- primary-no-rust:start -->
## Primary No-Rust Authoring Path

Edit `game.toml` and `assets/`, then run `game-dev check` and
`game-dev preview` through the prebuilt executable (`game-player`). Start with
`templates/no-rust-demo`, `examples/no-rust-minimal`,
`examples/no-rust-full`, or `examples/no-rust-tiled`.
<!-- primary-no-rust:end -->

## Secondary Rust Authoring Path

Use this only if you want to write Rust.

For a standalone game, start with `use game_starter::prelude::*;`. When the
game becomes a workspace content crate, use
`use game_kit::beginner::prelude::*;` instead. Both paths use the same
beginner-facing assets, prefabs, maps, rules, UI, and audio vocabulary.

Choose the level that matches the job:

- **Primary no-Rust target:** edit `game.toml` and `assets/` through a
  prebuilt player and CLI. Start with `templates/no-rust-demo`,
  `examples/no-rust-minimal`, `examples/no-rust-full`, or
  `examples/no-rust-tiled`.
- **Legacy RON data-driven compatibility:** make a small game by editing
  `assets/game.ron` and text maps inside the current Rust-wrapper demos. Start
  with `templates/data-driven-demo` or `examples/data-driven-full-demo` only
  when you are using the transitional RON path.
- **Secondary beginner Rust:** use builder chains for assets, prefabs, maps,
  rules, scenes, UI, audio, and animation.
- **Advanced game-kit/testbed content:** use raw prefabs, custom systems,
  queries, or advanced RON maps when those lower-level tools are truly needed.

| Feature | Primary no-Rust target | Secondary beginner Rust | Advanced |
| --- | --- | --- | --- |
| Player/enemy/pickups | yes | yes | yes |
| Doors/maps/scenes | yes | yes | yes |
| Projectiles/spawners | yes | yes | yes |
| Custom countdown/explosion | yes/basic | yes | yes/manual |
| Custom ECS systems | no | no | yes |
| No Rust required | roadmap target | no | no |

Helpful links:

- [Beginner authoring](beginner-authoring.md): the primary no-Rust package,
  secondary beginner Rust API, and legacy migration notes.
- [Tutorials](tutorials/README.md): build a one-file game in a guided order.
- [Cookbook](cookbook/README.md): copy a focused recipe for a common feature.
- [When to use the advanced API](when-to-use-advanced-api.md): decide whether
  lower-level control is actually needed.
- [Advanced content authoring](advanced-content-authoring.md): use raw prefabs,
  custom systems, queries, or advanced RON maps when those lower-level tools
  are truly needed.

The repository examples follow the same split:

- **Primary no-Rust:** `templates/no-rust-demo`,
  `examples/no-rust-minimal`, `examples/no-rust-events`,
  `examples/no-rust-waves`, `examples/no-rust-projectiles`,
  `examples/no-rust-full`, and `examples/no-rust-tiled`.
- **Legacy RON / data first:** `templates/data-driven-demo` and
  `examples/data-driven-full-demo`.
- **Beginner Rust / copy this first:** `examples/one-file-demo`,
  `examples/no-rust-shapes-demo`, `examples/script-like-custom-rules`,
  `simple-content`, and `templates/simple-demo`.
- **Structured beginner Rust:** `arena-content`. It is the next organization
  step when a beginner content crate needs typed assets and separate files.
- **Advanced / do not copy first:** `testbed-content`. It is an engine testbed
  for manual systems, advanced RON maps, tuple prefabs, direct component
  composition, custom state, and lower-level facade APIs—not a template for a
  first game.

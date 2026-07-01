# Content Authoring

Most games should begin with the [beginner authoring guide](beginner-authoring.md).
It uses names such as `"player"`, `"slime"`, and `"hit"` to describe a game
without exposing engine plumbing.
For the exact layer and import contract, see [api-boundary.md](api-boundary.md).

For a standalone game, start with `use game_starter::prelude::*;`. When the
game becomes a workspace content crate, use
`use game_kit::beginner::prelude::*;` instead. Both paths use the same
beginner-facing assets, prefabs, maps, rules, UI, and audio vocabulary.

Choose the level that matches the job:

- **No-Rust data-driven:** make a small game by editing `assets/game.ron` and
  text maps. Start with `templates/data-driven-demo` or
  `examples/data-driven-full-demo`.
- **Beginner Rust:** use builder chains for assets, prefabs, maps, rules,
  scenes, UI, audio, and animation.
- **Advanced game-kit/testbed content:** use raw prefabs, custom systems,
  queries, or RON maps when those lower-level tools are truly needed.

| Feature | No-Rust data-driven | Beginner Rust | Advanced |
| --- | --- | --- | --- |
| Player/enemy/pickups | yes | yes | yes |
| Doors/maps/scenes | yes | yes | yes |
| Projectiles/spawners | yes | yes | yes |
| Custom countdown/explosion | yes/basic | yes | yes/manual |
| Custom ECS systems | no | no | yes |
| No Rust required | yes | no | no |

Helpful links:

- [Beginner authoring](beginner-authoring.md): the beginner API and data path.
- [Tutorials](tutorials/README.md): build a one-file game in a guided order.
- [Cookbook](cookbook/README.md): copy a focused recipe for a common feature.
- [When to use the advanced API](when-to-use-advanced-api.md): decide whether
  lower-level control is actually needed.
- [Advanced content authoring](advanced-content-authoring.md): use raw prefabs,
  custom systems, queries, or RON maps when those lower-level tools are truly
  needed.

The repository examples follow the same split:

- **No-Rust / data first:** `templates/data-driven-demo` and
  `examples/data-driven-full-demo`.
- **Beginner Rust / copy this first:** `examples/one-file-demo`,
  `examples/no-rust-shapes-demo`, `examples/script-like-custom-rules`,
  `simple-content`, and `templates/simple-demo`.
- **Structured beginner Rust:** `arena-content`. It is the next organization
  step when a beginner content crate needs typed assets and separate files.
- **Advanced / do not copy first:** `testbed-content`. It is an engine testbed
  for manual systems, RON maps, tuple prefabs, direct component composition,
  custom state, and lower-level facade APIs—not a template for a first game.

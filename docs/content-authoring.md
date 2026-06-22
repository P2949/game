# Content Authoring

Most games should begin with the [beginner authoring guide](beginner-authoring.md).
It uses names such as `"player"`, `"slime"`, and `"hit"` to describe a game
without exposing engine plumbing.

For a standalone game, start with `use game_starter::prelude::*;`. When the
game becomes a workspace content crate, use
`use game_kit::beginner::prelude::*;` instead. Both paths use the same
beginner-facing assets, prefabs, maps, rules, UI, and audio vocabulary.

Choose the guide that matches the job:

- [Beginner authoring](beginner-authoring.md): make a small game using assets,
  prefabs, maps, rules, scenes, UI, audio, and animation.
- [Tutorials](tutorials/README.md): build a one-file game in a guided order.
- [Cookbook](cookbook/README.md): copy a focused recipe for a common feature.
- [Advanced content authoring](advanced-content-authoring.md): use raw prefabs,
  custom systems, queries, or RON maps when those lower-level tools are truly
  needed.

The repository examples follow the same split:

- **Beginner / copy this first:** `examples/one-file-demo`,
  `examples/no-rust-shapes-demo`, `examples/script-like-custom-rules`,
  `simple-content`, and `templates/simple-demo`.
- **Structured beginner Rust:** `arena-content`. It is the next organization
  step when a beginner content crate needs typed assets and separate files.
- **Advanced / do not copy first:** `testbed-content`. It is an engine testbed
  for manual systems, RON maps, tuple prefabs, direct component composition,
  custom state, and lower-level facade APIs—not a template for a first game.

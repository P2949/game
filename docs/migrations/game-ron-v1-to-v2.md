# game.ron v1 to v2

Status: draft policy for the first data-file schema change.

Current generated data projects use:

```ron
(
    version: 1,
    assets: (),
    prefabs: [],
    maps: [],
    rules: [],
)
```

`version: 1` remains the supported schema for `v0.1.0`. When a future release
introduces `version: 2`, keep the old file in source control, update one small
section at a time, and run:

```bash
game-dev validate-data assets/game.ron
cargo check
```

Expected migration rules:

- The changelog must name every data-file field or enum that changed.
- The migration guide must show the old v1 snippet and the new v2 snippet.
- Generated templates should move to the newest schema only on a tagged
  release.
- Whenever practical, the validator should reject old or mixed schemas with a
  direct message that points back to this guide.

No v2-only fields exist yet. Treat this file as the place to write the concrete
steps before a `version: 2` template ships.

For maintainers: `BeginnerGameFile.version` is the Rust-side field behind the
top-level `version:` entry in `assets/game.ron`.

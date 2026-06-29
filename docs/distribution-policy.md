# Distribution Policy

This project is still in beginner-productization release-candidate polish. The
distribution model favors reproducible generated projects and a simple release
process over publishing every crate before the beginner API has settled.

## Current Distribution Model

Generated projects use Git dependencies for `game-starter`.

Release-candidate templates pin that dependency to a specific git revision so a
new project does not track a moving branch by accident. After a release tag is
published, generated templates should move from the revision pin to the release
tag:

```toml
game-starter = { git = "https://github.com/P2949/game", tag = "v0.2.0", package = "game-starter" }
```

For development against this checkout, use:

```bash
cargo xtask new-demo my-game
```

That creates the same starter shape with a local path dependency.

Prebuilt demo zips are attached to GitHub Releases for players who want to try
the bundled demo before installing Rust. They are demo packages, not a full SDK
or installer, and they still require a Vulkan-capable GPU/driver.

## Why

The beginner API, data-file schema, and template layout are still young. A
pinned Git dependency gives generated projects reproducible builds without
adding crates.io release overhead before the public API has survived a release
cycle.

## Release Checklist Items

Before tagging a release:

- update generated-template dependency pins from the release-candidate revision
  to the release tag,
- update `CHANGELOG.md`,
- update migration docs in `docs/migrations/`,
- run generated-template CI,
- run the first-15-minutes checks,
- check prebuilt release artifacts.

## Future Work

Track these as future distribution issues, not as missing architecture:

- publish crates.io packages after the beginner API stabilizes,
- split templates into a dedicated `game-template` repository if template
  lifecycle starts moving independently,
- version docs per release,
- add a platform installer for `game-dev` if installing from Git becomes a
  real user obstacle.

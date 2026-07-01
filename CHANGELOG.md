# Changelog

All notable beginner-facing changes should be recorded here before release.

## Unreleased

### Architecture

- Narrowed `game-kit` and `game-core` root APIs toward explicit
  beginner/advanced/internal surfaces.
- Made map transitions content-aware by preventing raw active-map commands from
  normal content paths.
- Added structured runtime command diagnostics.
- Split large data, audio, CLI, and architecture-test modules.

### Migration

- Use `game_kit::beginner::prelude::*`,
  `game_kit::advanced::prelude::*`, or `game_starter::prelude::*` instead of
  `game_kit::prelude::*` or root `game_kit::*` imports.

## v0.2.0

### Added

- Added generated-template CI coverage for both starter templates outside the
  engine workspace.
- Added prebuilt demo release artifact workflow coverage for Linux and Windows
  packages.
- Added `examples/tiled-demo`, a beginner-facing runnable Tiled TMX import
  example, plus CI smoke coverage.
- Added `examples/data-driven-tiled-demo`, a no-Rust `assets/game.ron` Tiled
  TMX import example, plus CI smoke coverage.
- Added `game-dev check` for generated projects, combining doctor output,
  asset validation, optional `assets/game.ron` validation, and `cargo check`.
- Added `cargo xtask release-check` for contributor-side release-candidate
  verification, with generated-project checks and optional smoke skipping.
- Added `docs/distribution-policy.md` to explain tagged generated-project
  dependencies, release-tag pins, and deferred crates.io/template repository
  work.
- Added release-artifact archive verification for prebuilt demo packages before
  workflow upload/release attachment.
- Added a GitHub artifact verifier that downloads Linux and Windows release
  workflow artifacts and checks their package layouts after a workflow run.
- Released verified GitHub Release demo artifacts for `v0.2.0`:
  `game-demo-linux-x86_64.zip` and `game-demo-windows-x86_64.zip`.

### Changed

- Moved beginner callbacks to beginner-facing wrapper contexts.
- Generated templates now pin `game-starter` to the published `v0.2.0` release
  tag so external generated projects resolve a checked release by default.
- The Rust Tiled demo now uses example-local generated assets and a checked-in
  local TMX map instead of the workspace test texture.
- README and tutorial index now present the beginner entry path as three
  tracks: no-Rust data files, beginner Rust, and advanced.
- `game-dev validate-data` now accepts both `game.ron` and `assets/game.ron`
  from a generated project or workspace root.
- Beginner rule dependency diagnostics now use beginner builder method names
  and CLI failures point to doctor, asset/data checks, and common-errors docs.

### Fixed

- Countdown custom rules now reject typo'd data keys during `game.ron`
  validation and no longer treat missing runtime keys as zero.
- Runtime teardown now drops the renderer before the platform/window owner,
  preventing swapchain destruction from touching a torn-down native display.
- `cargo xtask package-demo --features ci-build-sdl3` now forwards the SDL3
  source-build feature through `xtask` to `game-cli`, avoiding system SDL3
  linker requirements in the release artifact workflow.
- The release artifact workflow now builds `xtask` with the source-built SDL3
  feature enabled before running `package-demo`.

### Deprecated

- Marked the broad `game_kit::prelude::*` as compatibility-only and deprecated.
- Renamed beginner actor iteration helpers to `each` / `each_tag`; the older
  `for_each` / `for_each_tag` names remain for one release.

### Removed

- Nothing.

### Migration notes

- See [v0.1 to v0.2](docs/migrations/v0.1-to-v0.2.md) for generated-project
  update notes.
- See [game.ron v1 to v2](docs/migrations/game-ron-v1-to-v2.md) for the data
  file schema migration policy.

## v0.1.0

### Added

- Initial beginner framework release target with Rust and data-driven starter
  templates, text maps, prefabs, rules, sound, UI, and packaging helpers.

### Changed

- Nothing.

### Deprecated

- Nothing.

### Removed

- Nothing.

### Migration notes

- First tagged release; no previous beginner API or data schema migration is
  required.

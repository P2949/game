# Changelog

All notable beginner-facing changes should be recorded here before release.

## Unreleased

### Added

- Added generated-template CI coverage for both starter templates outside the
  engine workspace.
- Added prebuilt demo release artifact workflow coverage for Linux and Windows
  packages.

### Changed

- Moved beginner callbacks to beginner-facing wrapper contexts.

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

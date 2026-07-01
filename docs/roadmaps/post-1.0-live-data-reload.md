# Post-1.0 Live Data Reload Roadmap

Full structural `assets/game.ron` hot reload is post-1.0 work. The 1.0 contract
intentionally reloads existing values and files while requiring restart for
structural changes.

## Goals

- Build a runtime-swappable `BeginnerContentModel`.
- Support prefab registry swap when prefab lists change.
- Support map registry swap when map lists change.
- Define an action identity strategy so bindings can survive reload.
- Define an asset key insertion strategy for newly added texture, sound, music,
  and animation keys.
- Reload scene and rule registries without rebuilding unrelated runtime state.
- Specify reset/restart behavior after a successful structural reload.
- Ensure a failed reload preserves the previous running model.
- Show debug overlay reload result states for success, partial reload, restart
  required, and failed reload.

## Non-Goals For 1.0

- Rebuilding runtime systems dynamically.
- Replacing compatibility APIs.
- Changing the current F5 contract documented in
  [`12-fast-iteration.md`](../tutorials/12-fast-iteration.md).

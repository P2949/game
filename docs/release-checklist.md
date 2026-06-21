# Release Checklist

Repeatable validation before tagging or packaging a build. Run from a clean
working tree.

This checklist protects the achieved content-authoring foundation: production
content goes through `game-kit`, while lower-level runtime, backend, registry,
schedule, and raw world APIs stay behind the facade.

## Automated checks

```bash
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build -p game --release --locked
GAME_ASSET_DIR=assets cargo run -p game --release --locked
rg "World|Entity::new|ids_with|get::<|get_mut::<|world_and_|world_mut\(|world\(" crates/arena-content/src crates/testbed-content/src
rg "movement_system|chase_system|patrol_system|apply_damage" crates/arena-content/src crates/testbed-content/src
cargo deny check advisories licenses sources bans
```

The two `rg` checks should report no production content hits; raw ECS inspection
is allowed only inside advanced `#[cfg(test)]` modules via
`game_kit::advanced::testing::prelude`.

To reproduce CI exactly (SDL3 built from source), use the `game/ci-build-sdl3`
workspace feature for workspace commands and `ci-build-sdl3` on package-specific
`game` commands. A headless render/teardown smoke check:

```bash
GAME_SMOKE_FRAMES=120 cargo run -p game --locked --features ci-build-sdl3
GAME_ASSET_DIR=assets GAME_SMOKE_FRAMES=120 cargo run -p game --release --locked --features ci-build-sdl3
```

## Packaged-layout check

A release build does not use the source-tree asset fallback, so confirm the
packaged layout (binary plus adjacent `assets/`) runs:

```bash
mkdir -p /tmp/game-package
cp target/release/game /tmp/game-package/
cp -r assets /tmp/game-package/
( cd /tmp/game-package && GAME_SMOKE_FRAMES=120 ./game )
```

## crates.io preflight

Only an authorized maintainer with crates.io credentials should perform this
section. First run the package dry-runs in dependency order; they rewrite
workspace path dependencies to their published versions and catch an invalid
release graph before any real publish:

```bash
cargo publish -p game-core --dry-run
cargo publish -p game-map --dry-run
cargo publish -p game-combat --dry-run
cargo publish -p game-audio --dry-run
cargo publish -p game-platform-sdl --dry-run
cargo publish -p game-renderer-vulkan --dry-run
cargo publish -p game-ai --dry-run
cargo publish -p game-physics --dry-run
cargo publish -p game-runtime --dry-run
cargo publish -p game-kit --dry-run
cargo publish -p game-starter --dry-run
```

After every dependency is publicly available at the intended version, publish
in the same order. Finally create a clean temporary project with
`cargo add game-starter` and run its starter example before advertising the
crates.io route in the tutorials.

## Manual smoke (interactive)

Run `GAME_ASSET_DIR=assets cargo run -p game --release` and verify:

- [ ] Window opens at the requested size
- [ ] World sprites render (floor grid, solids, player)
- [ ] HUD text renders
- [ ] Audio starts; `Space`/`Enter` plays the generated blip
- [ ] Movement collides with walls and slides along them; very fast movement may still tunnel
- [ ] `P` pauses and resumes; effects freeze while paused
- [ ] `R` resets; `K` triggers the death screen
- [ ] Window resize keeps rendering and recovers (no freeze/crash)
- [ ] Clean exit (close window / `Esc`) with no validation errors in the log

## Platform notes

- Wayland and X11 are both supported via SDL3; `SDL_VIDEODRIVER` can force one.
- Debug builds require the Vulkan validation layer unless
  `GAME_DISABLE_VALIDATION=1` is set.

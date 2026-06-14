# Release Checklist

Repeatable validation before tagging or packaging a build. Run from a clean
working tree.

## Automated checks

```bash
cargo fmt --all -- --check
cargo test --locked
cargo clippy --all-targets --locked -- -D warnings
cargo build --release --locked
GAME_ASSET_DIR=assets cargo run --release --locked
cargo deny check advisories licenses sources bans
```

To reproduce CI exactly (SDL3 built from source), append `--features
ci-build-sdl3` to the cargo commands. A headless render/teardown smoke check:

```bash
GAME_SMOKE_FRAMES=120 cargo run --locked            # debug: validation layers on
GAME_ASSET_DIR=assets GAME_SMOKE_FRAMES=120 cargo run --release --locked
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

## Manual smoke (interactive)

Run `GAME_ASSET_DIR=assets cargo run --release` and verify:

- [ ] Window opens at the requested size
- [ ] World sprites render (floor grid, solids, player)
- [ ] HUD text and the frame-time graph render
- [ ] Audio starts (looping tone; `Space`/`Enter` plays the blip)
- [ ] Movement collides with walls and slides along them (no tunneling)
- [ ] `P` pauses and resumes; effects freeze while paused
- [ ] `R` resets; `K` triggers the death screen
- [ ] Window resize keeps rendering and recovers (no freeze/crash)
- [ ] Clean exit (close window / `Esc`) with no validation errors in the log

## Platform notes

- Wayland and X11 are both supported via SDL3; `SDL_VIDEODRIVER` can force one.
- Debug builds require the Vulkan validation layer unless
  `GAME_DISABLE_VALIDATION=1` is set.

# game

A small Rust game prototype built directly on SDL3 (window, input, audio) and a
hand-written Vulkan renderer via [`ash`](https://crates.io/crates/ash).

## Features

- SDL3 window, keyboard input, and a lock-free audio mixer
- Vulkan 1.3 renderer (dynamic rendering, synchronization2) through `ash`
- Sprite batching with per-frame-reused vertex buffers (no steady-state allocation)
- Fixed-timestep update loop with render interpolation
- Bitmap (ASCII) UI text rendered from a runtime-built font atlas

## Requirements

- A recent stable Rust toolchain (the crate uses the 2024 edition)
- SDL3 development libraries (`libsdl3-dev` or your platform equivalent)
- A Vulkan loader and a GPU driver with Vulkan 1.3 support
- `glslc` (from shaderc / the Vulkan SDK) on `PATH` — shaders are compiled at build time
- Vulkan validation layers for debug builds (debug builds require the
  `VK_LAYER_KHRONOS_validation` layer and will refuse to start without it)

## Build and run

```bash
cargo run            # debug build (validation layers enabled)
cargo run --release  # optimized build (LTO, single codegen unit)
```

Assets (`assets/textures/test.png`, `assets/fonts/DejaVuSans.ttf`) are loaded
relative to the executable when present, falling back to the crate manifest
directory so `cargo run` works from a source checkout.

## Packaging

The executable resolves assets relative to its own location first, so a packaged
or installed build must ship the `assets/` directory next to the binary:

```text
<install dir>/
├── game            # the executable
└── assets/
    ├── fonts/DejaVuSans.ttf
    └── textures/test.png
```

The crate manifest directory is only a development fallback (used by `cargo run`
from a source checkout); an installed build cannot rely on it.

## Controls

| Action            | Keys                          |
| ----------------- | ----------------------------- |
| Move              | `W` `A` `S` `D` / Arrow keys  |
| Action (blip)     | `Space` / `Enter`             |
| Pause             | `P`                           |
| Reset             | `R`                           |
| Debug: kill player| `K`                           |
| Zoom in / out     | `+` / `=` and `-`             |
| Quit              | `Esc` (or close the window)   |

## Development checks

```bash
cargo fmt --all -- --check
cargo test --locked
cargo clippy --all-targets --locked -- -D warnings
cargo build --release --locked
```

These commands use your system SDL3 development libraries. CI instead builds
SDL3 from source through the `ci-build-sdl3` feature so the workflow does not
depend on whether the runner image ships a `libsdl3-dev` package. To reproduce
the CI build exactly, pass that feature:

```bash
cargo test --locked --features ci-build-sdl3
cargo clippy --all-targets --locked --features ci-build-sdl3 -- -D warnings
cargo build --release --locked --features ci-build-sdl3
```

## Known limitations

- UI text is ASCII-only (unsupported characters render a fallback glyph)
- Collision is discrete AABB and can tunnel at very high speeds
- The texture set and font atlas are tuned for the bundled assets
- Debug builds depend on the Vulkan validation layer being installed

## License

This project is licensed under the GNU General Public License v3.0 or later
(GPL-3.0-or-later). See [`LICENSE`](LICENSE) for the full text.

Bundled third-party assets are covered by their own licenses; see
[`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md).

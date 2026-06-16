# game

A small Rust game prototype built directly on SDL3 (window, input, audio) and a
hand-written Vulkan renderer via [`ash`](https://crates.io/crates/ash).

## Workspace layout

The project is a Cargo workspace. The reusable engine and the gameplay content
are separate crates, and the binary (`bin/game`) only selects which content
plugin to run:

- engine/runtime: `game-core`, `game-runtime`, `game-renderer-vulkan`,
  `game-platform-sdl`, `game-audio`
- gameplay building blocks: `game-map`, `game-ai`, `game-combat`, `game-physics`
- content authoring facade: `game-kit`
- content plugins (demos): `arena-content` (default) and `testbed-content`

The binary picks a demo from the `GAME_DEMO` environment variable (`arena` by
default, or `testbed`); the runtime, renderer, audio, and platform crates are
identical for both.

## Project status

This is a small Rust/SDL3/Vulkan game prototype. It currently focuses on:

- explicit, RAII-driven Vulkan renderer lifetime handling
- 2D sprite rendering with layered, texture-batched draws
- fixed-timestep gameplay
- axis-separated AABB collision with wall sliding
- simple generated audio through a lock-free mixer

It is **not** yet:

- a full engine
- a finished game
- a general asset pipeline
- a file-backed audio pipeline; sound effects are generated today

## Features

- SDL3 window, keyboard input, and a lock-free audio mixer
- Vulkan 1.3 renderer (dynamic rendering, synchronization2) through `ash`
- Sprite batching reuses dynamic GPU vertex buffers after growth, avoiding
  steady-state GPU buffer allocation for normal sprite submission
- Fixed-timestep update loop
- Bitmap (ASCII) UI text rendered from a runtime-built font atlas

## Content Authoring Model

Content crates import the facade:

```rust
use game_kit::prelude::*;
```

This is the intended foundation milestone: content code now talks to
`game-kit`, not to runtime/backend/registry/schedule internals.

They declare assets, controls, prefabs, maps, and systems through `GameApp`.
For example:

```rust
impl GamePlugin for DemoPlugin {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        let assets = game.assets(assets::register)?;
        let input = game.input(input::register)?;
        prefabs::register(game, &assets, &input)?;
        level::register(game, &assets)?;
        systems::register(game, &assets, &input);
        Ok(())
    }
}

game.prefab("demo/player", |prefab| {
    prefab
        .spawn(|at| {
            (
                Transform::at(at),
                Velocity::default(),
                PlayerController {
                    move_axis: actions.movement,
                },
                Health::new(100),
                Sprite::new(assets.player, vec2s(20.0)),
                Collider::box_of(vec2s(20.0)),
            )
        })?
        .require::<Transform>()
        .require::<Collider>();
    Ok(())
})?;

game.fixed_active::<GameState>(player_control_system);
```

Content crates do not talk to SDL, Vulkan, audio devices, memory allocation,
swapchains, descriptor sets, renderer texture IDs, event pumps, fixed timestep
internals, raw `World`/`Input`/`TileMap`/`NavGrid`, or direct low-level AI,
physics, and combat systems. See
[`docs/content-authoring.md`](docs/content-authoring.md) for the author-facing
guide.

## Requirements

- A recent stable Rust toolchain (the crate uses the 2024 edition)
- SDL3 development libraries (`libsdl3-dev` or your platform equivalent)
- A Vulkan loader and a GPU driver with Vulkan 1.3 support
- `glslc` (from shaderc / the Vulkan SDK) on `PATH` — shaders are compiled at build time
- Vulkan validation layers for debug builds unless disabled with
  `GAME_DISABLE_VALIDATION=1`

## Build and run

```bash
cargo run -p game                                   # debug build (validation layers enabled)
GAME_ASSET_DIR=assets cargo run -p game --release   # optimized build (LTO, single codegen unit)
GAME_DEMO=testbed cargo run -p game                 # run the second (testbed) demo
```

The workspace sets `bin/game` as its default member, so a plain `cargo run`
works from the repository root today. The README uses `-p game` anyway because
that form stays unambiguous if more binaries are added later.

A debug `cargo run` from a source checkout finds `assets/` through the
source-tree fallback, but a `--release` build does **not** use that fallback (see
the discovery order below), so point it at the asset directory explicitly with
`GAME_ASSET_DIR=assets`. A packaged build instead ships `assets/` next to the
binary (see [Packaging](#packaging)).

Debug builds require Vulkan validation layers by default. On systems without
the layer installed, disable that requirement explicitly:

```bash
GAME_DISABLE_VALIDATION=1 cargo run -p game
```

Assets are discovered in this order:

1. `GAME_ASSET_DIR`, if set
2. `assets/` next to the executable
3. `assets/` under the crate manifest directory in debug builds only

Release packages should not rely on the source-tree fallback.

## Runtime Environment Variables

| Variable | Effect |
| -------- | ------ |
| `GAME_DEMO` | Selects the content plugin: `arena` (default) or `testbed`. |
| `GAME_SMOKE_FRAMES` | If set to `N`, initializes normally, renders exactly `N` frames, then exits. `0` exits after initialization before rendering. Invalid values fail early. |
| `GAME_ASSET_DIR` | Overrides runtime asset root discovery. |
| `GAME_PRESENT_MODE` | `fifo` (default), `mailbox`, or `immediate`; unavailable opt-in modes fall back to FIFO. |
| `GAME_VK_DEVICE_NAME` | Selects a suitable Vulkan GPU whose device name contains the given substring. |
| `GAME_FRAME_TIMINGS` | Set to `1`, `true`, `yes`, or `on` to emit periodic debug frame timing logs. |
| `GAME_REQUIRE_VALIDATION` | Set to `1` to require Vulkan validation layers in any build. |
| `GAME_DISABLE_VALIDATION` | Set to `1` to disable Vulkan validation layers in any build. |
| `RUST_LOG` | Controls Rust logging via `env_logger`; defaults to `info`. |
| `GLSLC` | Overrides the shader compiler path used by `build.rs`. |
| `SPIRV_VAL` | Overrides the optional SPIR-V validator path used by `build.rs`. If unset, `spirv-val` on `PATH` is used when available. Set to `0`, `off`, `none`, or `disabled` to skip validation. |

## Packaging

Runtime packages need the executable plus `assets/`. Shader source files are
build-time inputs; compiled SPIR-V is embedded into the binary. A packaged or
installed build should ship the `assets/` directory next to the binary:

```text
<install dir>/
├── game            # the executable
└── assets/
    ├── fonts/DejaVuSans.ttf
    └── textures/test.png
```

The crate manifest directory is only a debug development fallback (used by
`cargo run` from a source checkout); an installed build cannot rely on it.

## Renderer Scope

- 2D sprite renderer
- Dynamic rendering, no depth/stencil
- No render graph
- No texture atlas yet
- No bindless descriptors
- Simple bitmap text
- No asset hot reload

Sprites are batched by layer and texture. Layer order always wins. Within one
layer, texture batching groups draws by texture; same-layer cross-texture
submission order is not preserved. Use separate layers when strict ordering is
required across textures.

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
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build -p game --release --locked
GAME_SMOKE_FRAMES=120 cargo run -p game --locked
GAME_ASSET_DIR=assets GAME_SMOKE_FRAMES=120 cargo run -p game --release --locked
```

These commands use your system SDL3 development libraries. CI instead builds
SDL3 from source through the `ci-build-sdl3` feature (defined on the `game`
binary package, which forwards it to `game-platform-sdl` and `game-audio`) so
the workflow does not depend on whether the runner image ships a `libsdl3-dev`
package. The feature lives on a single package, so passing it at the workspace
root just enables it wherever it is defined:

```bash
cargo test --workspace --locked --features game/ci-build-sdl3
cargo clippy --workspace --all-targets --locked --features game/ci-build-sdl3 -- -D warnings
cargo build -p game --release --locked --features ci-build-sdl3
GAME_SMOKE_FRAMES=120 cargo run -p game --locked --features ci-build-sdl3
GAME_ASSET_DIR=assets GAME_SMOKE_FRAMES=120 cargo run -p game --release --locked --features ci-build-sdl3
```

## Known limitations

- UI text is ASCII-only (unsupported characters render a fallback glyph)
- Collision uses discrete axis-separated AABB resolution. It does not perform
  swept collision, so very fast movement can tunnel through thin solids.
  Embedded spawn positions should be avoided or validated.
- The texture set and font atlas are tuned for the bundled assets
- Debug builds require the Vulkan validation layer by default

## License

This project is licensed under the GNU General Public License v3.0 or later
(GPL-3.0-or-later). See [`LICENSE`](LICENSE) for the full text.

Bundled third-party assets are covered by their own licenses; see
[`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md).

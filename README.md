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
- content plugins (demos): `simple-content` (pure beginner example),
  `arena-content` (beginner-style playable demo), and `testbed-content`
  (advanced testbed showing manual systems, RON maps, and lower-level
  `game-kit` APIs)

The binary picks a demo from the `GAME_DEMO` environment variable (`arena` by
default, plus `simple` or `testbed`); the runtime, renderer, audio, and platform
crates are identical for all demos.

## Project status

The engine/content split and beginner authoring foundation are implemented.
Beginner Productization 1.0 is complete for `v0.2.0`: the local release gate
passed, and the GitHub Release has verified Linux/Windows demo packages.

Start with one of three tracks:

- **Track A: No Rust.** Use `templates/data-driven-demo` and edit
  `assets/game.ron`.
- **Track B: Beginner Rust.** Use `templates/simple-demo`; follow tutorials 00-12.
- **Track C: Advanced.** Use the advanced path only when beginner APIs are insufficient.

This is still a small Rust/SDL3/Vulkan game prototype. It currently focuses on:

- explicit, RAII-driven Vulkan renderer lifetime handling
- 2D sprite rendering with layered, texture-batched draws
- fixed-timestep gameplay
- axis-separated AABB collision with wall sliding
- file-backed sound effects and looping music handles through `game-kit`, plus
  optional OGG Vorbis/MP3 decoding and bounded streaming for long PCM16 WAV
  music tracks
- generated placeholder sounds through a lock-free mixer

It is **not** yet:

- a full engine
- a finished game
- a general asset pipeline
- general streamed-audio codecs; long tracks currently stream as 48 kHz stereo
  PCM16 WAV through a bounded background reader

## Features

- SDL3 window, keyboard/mouse/gamepad input, and a lock-free audio mixer
- Vulkan 1.3 renderer (dynamic rendering, synchronization2) through `ash`
- Sprite batching reuses dynamic GPU vertex buffers after growth, avoiding
  steady-state GPU buffer allocation for normal sprite submission
- Fixed-timestep update loop
- Bitmap UI text rendered from a runtime-built ASCII font atlas; unsupported
  characters use a fallback glyph. Latin-1, dynamic glyph caching, and complex
  text shaping are future work.

## Content Authoring Model

Start with one file. This is the path used throughout the beginner tutorial:

```rust
use game_starter::prelude::*;

fn main() -> Result<()> {
    run_game("My Game", |game| {
        // Add assets, prefabs, maps, and rules here.
        Ok(())
    })
}
```

When a workspace game grows into a reusable content crate, use the same
beginner vocabulary and let `content_plugin!` supply the crate glue:

```rust
use game_kit::beginner::prelude::*;

content_plugin!(MyContent, plugin, |game| {
    game.asset_bag()
        .texture("player", "textures/player.png")?
        .texture("floor", "textures/floor.png")?
        .texture("wall", "textures/wall.png")?
        .build();

    game.player_prefab("player").sprite("player").build()?;
    game.map("level_1")
        .tiles(["#####", "#P..#", "#####"])
        .simple_theme("floor", "wall")
        .legend('P', "player")
        .start();
});
```

Start a project from anywhere with:

```bash
cargo install cargo-generate
cargo generate --git https://github.com/P2949/game templates/simple-demo --name my-game
cd my-game
cargo install --git https://github.com/P2949/game game-cli
game-dev doctor
game-dev check
game-dev run
```

That creates a one-file beginner game with a pinned git dependency on
`game-starter`. The current templates pin the published `v0.2.0` release tag, so
external generated projects resolve the same checked release by default. From a
local checkout, `cargo xtask new-demo my-game` creates the same starter with a
local path dependency. Use
`cargo generate --git https://github.com/P2949/game templates/data-driven-demo --name my-game`
when you want the same first-game setup in editable `assets/game.ron` instead.
Generated projects can use the helper commands without cloning the engine
repository:

```bash
game-dev check
game-dev asset-check
game-dev validate-data
game-dev package --release --out dist/my-game --zip
```

Want to try before building? Download the latest demo package from
[Releases](https://github.com/P2949/game/releases). The prebuilt
`game-demo-linux-x86_64.zip` and `game-demo-windows-x86_64.zip` packages let
you unzip and run the bundled demo before installing Rust or SDL3. They still
require a Vulkan-capable GPU/driver, and source builds remain the main
development path.

## What Should I Copy First?

- **No Rust:** `templates/data-driven-demo`,
  `examples/data-driven-events-demo`, `examples/data-driven-waves-demo`,
  `examples/data-driven-projectiles-demo`
- **Tiled no-Rust:** `examples/data-driven-tiled-demo`
- **First Rust game:** `templates/simple-demo`
- **One-file example:** `examples/one-file-demo`
- **Full beginner feature sample:** `examples/no-rust-shapes-demo`
- **Custom behavior:** `examples/script-like-custom-rules`
- **Events:** `examples/events-demo`
- **Tiled Rust:** `examples/tiled-demo`
- **Structured beginner content:** `arena-content`
- **Advanced lab:** `crates/testbed-content` - do not copy first

`testbed-content` is intentionally advanced; see
[when to use the advanced API](docs/when-to-use-advanced-api.md) and the
[advanced authoring guide](docs/advanced-content-authoring.md) only when you
need that separate path. The [beginner guide](docs/beginner-authoring.md),
[tutorials](docs/tutorials/README.md), and [cookbook](docs/cookbook/README.md)
are the normal starting points.

## Authoring levels

- **No-Rust data-driven:** edit `assets/game.ron` and text maps. Start with
  `templates/data-driven-demo`, `examples/data-driven-events-demo`,
  `examples/data-driven-waves-demo`, `examples/data-driven-projectiles-demo`,
  or `examples/data-driven-full-demo` when the game should be made mostly from
  data files. Use `examples/data-driven-tiled-demo` for Tiled no-Rust maps.
- **Beginner Rust builder chains:** use `game_starter::prelude::*` with
  high-level builders for assets, prefabs, maps, actions, scenes, sound,
  animation, and rules. Start with `examples/one-file-demo`,
  `examples/no-rust-shapes-demo`, `examples/script-like-custom-rules`,
  `simple-content`, and `templates/simple-demo`. Use `examples/tiled-demo` for
  Tiled Rust maps.
### Advanced API

- **Advanced game-kit/testbed content:** use
  `game_kit::advanced::prelude::*` only when you need custom systems, raw
  prefabs, queries, or lower-level engine-shaped content. `testbed-content` is
  intentionally advanced and is not the beginner sample.
- **Engine/runtime API:** internal engine, runtime, renderer, platform, and
  audio crates. Not for beginner content.

| Feature | No-Rust data-driven | Beginner Rust | Advanced |
| --- | --- | --- | --- |
| Player/enemy/pickups | yes | yes | yes |
| Doors/maps/scenes | yes | yes | yes |
| Projectiles/spawners | yes | yes | yes |
| Custom countdown/explosion | yes/basic | yes | yes/manual |
| Custom ECS systems | no | no | yes |
| No Rust required | yes | no | no |

## API Stability

- **Beginner API:** stabilized first. Renamed beginner methods should keep the
  old method for one release with a deprecation note, a changelog entry, and a
  migration note.
- **Data file schema:** versioned through `assets/game.ron` and its `version`
  field. The current schema is `version: 1`; future schema changes should get a
  guide in [docs/migrations](docs/migrations/README.md).
- **Advanced API:** allowed to evolve faster for custom systems, manual
  prefabs, and low-level experiments.
- **Engine internals:** unstable. Runtime, renderer, backend, and raw world
  details are not a beginner content API.

Generated templates pin `game-starter` to the published `v0.2.0` release tag, so
new projects are not tied to a moving branch by default. See the
[distribution policy](docs/distribution-policy.md) for the current Git-based
model and future crates.io/template-repository plan.

## What should I copy?

If you are new:

1. Copy `examples/one-file-demo`.
2. Then read `examples/no-rust-shapes-demo`.
3. Then use
   `cargo generate --git https://github.com/P2949/game templates/simple-demo --name my-game`
   to create your own Rust demo, or
   `cargo generate --git https://github.com/P2949/game templates/data-driven-demo --name my-game`
   for a no-Rust demo. From a local checkout, `cargo xtask new-demo my-game`
   uses your local sources instead of the pinned release tag.

If you want a workspace content crate:

1. Copy `simple-content`.
2. Move to `arena-content` when you want a slightly more organized version
   with typed assets and separate files.

Do not copy `testbed-content` unless you want the advanced API.

Further reading: [beginner authoring](docs/beginner-authoring.md),
[when to use the advanced API](docs/when-to-use-advanced-api.md),
[advanced content authoring](docs/advanced-content-authoring.md), and the
[architecture guide](docs/architecture.md).

## Requirements

- A recent stable Rust toolchain (the crate uses the 2024 edition)
- SDL3 development libraries (`libsdl3-dev` or your platform equivalent)
- A Vulkan loader and a GPU driver with Vulkan 1.3 support
- `glslc` (from shaderc / the Vulkan SDK) on `PATH` — shaders are compiled at build time
- Vulkan validation layers for debug builds unless disabled with
  `GAME_DISABLE_VALIDATION=1`

In a generated project, run `game-dev doctor --explain` before the first
windowed run for setup-specific fixes.

## Build and run

```bash
cargo run -p game                                   # debug build (validation layers enabled)
GAME_ASSET_DIR=assets cargo run -p game --release   # optimized build (LTO, single codegen unit)
GAME_DEMO=simple cargo run -p game                  # run the tiny beginner-pressure-test demo
GAME_DEMO=testbed cargo run -p game                 # run the advanced testbed demo
```

The workspace sets `bin/game` as its default member, so a plain `cargo run`
works from the repository root today. The README uses `-p game` anyway because
that form stays unambiguous if more binaries are added later.

A debug `cargo run` from a source checkout finds the nearest useful `assets/`
folder through the discovery order below, but a `--release` build does **not**
use the source-tree fallback, so point it at the asset directory explicitly with
`GAME_ASSET_DIR=assets`. A packaged build instead ships `assets/` next to the
binary (see [Packaging](#packaging)).

Debug builds require Vulkan validation layers by default. On systems without
the layer installed, disable that requirement explicitly:

```bash
GAME_DISABLE_VALIDATION=1 cargo run -p game
```

Assets are discovered in this order:

1. `GAME_ASSET_DIR`, if set
2. `assets/` in the current working directory
3. `assets/` next to the executable
4. `assets/` under the crate manifest directory in debug builds only

Release packages should not rely on the source-tree fallback.

## Runtime Environment Variables

| Variable | Effect |
| -------- | ------ |
| `GAME_DEMO` | Selects the content plugin: `arena` (default), `simple`, or `testbed`. |
| `GAME_SMOKE_FRAMES` | If set to `N`, initializes normally, renders exactly `N` frames, then exits. `0` exits after initialization before rendering. Invalid values fail early. |
| `GAME_ASSET_DIR` | Overrides runtime asset root discovery. |
| `GAME_DEV_RELOAD` | Set to `1` in a release build to enable F5 text-map reload and optional configured tuning reload. Debug builds enable it automatically. |
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
- F5 reloads text maps, configured tuning, and registered textures/sounds in
  development. Existing voices using a replaced static sound stop; an active
  streamed track restarts from the updated file.

Sprites are batched by layer and texture. Layer order always wins. Within one
layer, texture batching groups draws by texture; same-layer cross-texture
submission order is not preserved. Use separate layers when strict ordering is
required across textures.

## Controls

| Action | Keyboard / Mouse | Gamepad |
| ------ | ---------------- | ------- |
| Move | `W` `A` `S` `D` / Arrow keys | Left stick / D-pad |
| Attack / confirm | `Space` / `Enter` / left mouse | South face button |
| Pause | `Esc` / `P` | Start |
| Reset | `R` | Select |
| Debug overlay | `F1` | North face button |
| Debug kill | `K` | West face button |
| Zoom in / out | `+` / `=` and `-` | Shoulder buttons |
| Quit | `Esc` / close window | — |

## Development checks

For the contributor release-candidate gate, run:

```bash
cargo xtask release-check --skip-smoke
```

Omit `--skip-smoke` on a machine with a working graphical backend when you want
the full local smoke gate.

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

- UI text currently uses the bundled ASCII bitmap atlas; unsupported characters
  render a fallback glyph. Latin-1/common accented characters need dynamic
  glyph caching, while complex text shaping remains out of scope for now.
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

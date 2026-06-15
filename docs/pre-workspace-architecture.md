# Current Architecture

This project is currently a single binary crate with five top-level source
areas under `src/`:

- `audio` = SDL audio stream, lock-free mixer, generated tones, and audio
  command playback.
- `engine` = app loop, world storage, tile map, camera, input facade, physics,
  navigation, asset handles, and gameplay-facing graphics/audio facades.
- `game` = arena-specific actors, AI, combat, level construction, and spawn
  rules.
- `platform` = SDL window creation, SDL event/input polling, resize policy, and
  fixed timestep helper.
- `renderer` = Vulkan renderer, texture registry, sprite batching, text atlas,
  command recording, and Vulkan resource lifetime wrappers.

The binary entry point in `src/main.rs` declares all of these modules directly
and starts the current arena with:

```rust
engine::run(game::ArenaGame::new(), "Arena")
```

The useful architectural seam today is `engine::app::Game`: arena content owns
`ArenaGame`, while `engine::app::run` drives the loop. That seam is still too
shallow because `run` also owns SDL startup, Vulkan creation, audio creation,
fixed timestep scheduling, map setup, world setup, rendering, and update
orchestration.

## Current dependency leaks

These imports are intentionally documented before the refactor so they can be
removed deliberately:

- `engine::app -> platform, renderer, audio`
- `engine::gfx -> renderer::TextureId`
- `engine::audio -> audio::AudioSystem`
- `renderer::context -> engine::camera::Camera2D`
- `game::ai` tests -> `platform::input`
- `game::combat` tests -> `platform::input`

## Future crate-boundary imports

With `src/lib.rs` in place, the following internal paths mark the module groups
that will become crate-to-crate imports during the workspace split:

- `crate::engine` -> `game-core`
- `crate::renderer` -> `game-renderer-vulkan`
- `crate::platform` -> `game-platform-sdl`
- `crate::audio` -> `game-audio`
- `crate::game` -> `arena-content`

Phase 1 leaves these paths intact on purpose. The boundary is recorded here so
the Phase 2 move can be mechanical and easy to review.

## Current ownership notes

- `engine::world` already uses a slot/generation `EntityId` model. This should
  be preserved and evolved rather than replaced wholesale.
- `game::actor::Actor` currently encodes game-specific roles as
  `Actor::Player(Player)` and `Actor::Enemy(Enemy)`, which limits content
  growth.
- `engine::tilemap::TileMap` currently owns only `Tile::Floor`, `Tile::Wall`,
  and marker spawns as `(char, usize, usize)`.
- `engine::assets::Assets` maps floor, wall, player, and enemy sprites to the
  renderer's built-in test texture.
- `engine::audio::Audio` exposes only the demo-specific `hit()` helper.
- `platform::input::InputState` stores arena gameplay actions directly instead
  of backend-neutral key events.

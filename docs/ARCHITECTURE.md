# Architecture Overview

> **Workspace layout.** The code is a Cargo workspace; the binary (`bin/game`)
> only selects a content plugin. The engine lives in `game-core` /
> `game-runtime` / `game-renderer-vulkan` / `game-platform-sdl` / `game-audio`,
> gameplay building blocks in `game-map` / `game-ai` / `game-combat` /
> `game-physics`, and demos in `arena-content` / `testbed-content` (see the
> README "Workspace layout" section).

```text
bin/game
  selects plugin and runtime config

content crates
  use game-kit only

game-kit
  friendly authoring facade over core/map/ai/combat/physics

engine-neutral crates
  game-core, game-map, game-ai, game-combat, game-physics

runtime/backend crates
  game-runtime, game-renderer-vulkan, game-platform-sdl, game-audio
```

Content authors use `game_kit::prelude::*`; lower-level builder, schedule,
registry, validator, and raw context APIs are for the runtime, facade, and
engine tests.

## Main Loop

`game-runtime`'s `runner.rs` owns platform event pumping, fixed-timestep
simulation, rendering, audio command submission, resize handling, and smoke-test
shutdown. Rendering is skipped while the drawable size is zero.

## Fixed Timestep

`game-runtime::fixed_timestep::FixedTimestep` advances gameplay at 120 Hz and
caps catch-up steps per rendered frame. Excess accumulated lag is discarded with
a rate-limited warning. The runtime extracts the current simulation state for
rendering; it does not currently interpolate between previous/current transforms.

## Input

`game-platform-sdl` records physical key state in `game_core::input::InputState`.
The runtime resolves that through the content-defined `InputRegistry` each frame,
producing `Input` keyed by `ActionId` and `Axis2dId`. Movement axes are clamped
and diagonal movement is normalized, while gameplay code asks for logical
actions/axes instead of SDL keys.

## Renderer Ownership

`game-renderer-vulkan::context::VulkanContext` owns Vulkan instance/device
state, swapchain resources, dynamic sprite buffers, pipelines, textures, and
frame resources. Frame-owned sync lives in `FrameData`; swapchain-image-owned
present sync lives in `SwapchainSync` and is recreated with the swapchain.

Vulkan physical-device selection filters for required queues, extensions,
features, and swapchain support, then scores suitable candidates. Set
`GAME_VK_DEVICE_NAME` to a case-insensitive device-name substring to choose the
highest-scoring suitable match on multi-GPU systems.

## Swapchain Recreation

Resize, suboptimal, and out-of-date paths request recreation through
`game-renderer-vulkan::recreate`. Out-of-date requests are mandatory.
Swapchain-generation resources are replaced only after `device_wait_idle`.

Hard out-of-date requests skip rendering until a nonzero extent can be
recreated. Soft resize/suboptimal requests remain pending while the renderer
continues to draw with the current swapchain until debounce/rate-limit
conditions allow recreation.

## Asset Loading

Content assets are registered in `AssetRegistry` and validated before backend
startup. Runtime assets are loaded from `GAME_ASSET_DIR`, executable-adjacent
`assets/`, or the source-tree debug fallback. Missing content assets report the
paths that were tried, and texture/font loaders attach exact path context. The
runtime also validates renderer built-in assets, currently
`assets/fonts/DejaVuSans.ttf`, before creating the Vulkan context.

## Audio Mixer

The SDL audio callback drains generated play commands from a bounded lock-free
queue and mixes generated tones into an f32 stream. Content requests generated
sound effects through `assets.generated_sound(..)`. File-backed loading,
decoding, and resampling are intentionally not implemented yet.

## Current Limitations

The renderer is a focused 2D sprite path with bitmap ASCII text, no render graph,
no depth/stencil, no texture atlas, and no hot reload. Collision is discrete AABB
resolution and can tunnel for fast movement.

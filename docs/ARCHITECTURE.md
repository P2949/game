# Architecture Overview

## Main Loop

`src/main.rs` owns platform event pumping, fixed-timestep simulation, rendering,
audio command submission, resize handling, and smoke-test shutdown. Rendering is
skipped while the drawable size is zero.

## Fixed Timestep

`platform::time::FixedTimestep` advances gameplay at 120 Hz and caps catch-up
steps per rendered frame. Excess accumulated lag is discarded with a
rate-limited warning.

## Input

`platform::input::InputState` aggregates SDL key state into sanitized movement
axes and edge-triggered frame actions. Movement axes are clamped and diagonal
movement is normalized.

## Renderer Ownership

`renderer::context::VulkanContext` owns Vulkan instance/device state, swapchain
resources, dynamic sprite buffers, pipelines, textures, and frame resources.
Frame-owned sync lives in `FrameData`; swapchain-image-owned present sync lives
in `SwapchainSync` and is recreated with the swapchain.

## Swapchain Recreation

Resize, suboptimal, and out-of-date paths request recreation through
`renderer::recreate`. Out-of-date requests are mandatory. Swapchain-generation
resources are replaced only after `device_wait_idle`.

## Asset Loading

Runtime assets are loaded from `GAME_ASSET_DIR`, executable-adjacent `assets/`,
or the source-tree debug fallback. Missing assets report the paths that were
tried, and texture/font loaders attach exact path context.

## Audio Mixer

The SDL audio callback drains play commands from a bounded lock-free queue and
mixes registered sounds into an f32 stream. Sounds must match the mixer output
format; resampling is intentionally not implemented yet.

## Current Limitations

The renderer is a focused 2D sprite path with bitmap ASCII text, no render graph,
no depth/stencil, no texture atlas, and no hot reload. Collision is discrete AABB
resolution and can tunnel for fast movement.

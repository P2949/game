# Architecture guide

The workspace has three layers:

1. Beginner and advanced content depend on `game-kit`.
2. `game-kit` depends only on gameplay/core crates, never on SDL, Vulkan, or the
   audio backend.
3. The runtime owns platform, renderer, and audio integration.

The automated architecture tests protect those boundaries and keep beginner
examples free of ECS-shaped escape hatches. For the fuller historical design
notes, see [ARCHITECTURE.md](ARCHITECTURE.md).

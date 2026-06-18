# Beginner Authoring Roadmap

This roadmap describes the next authoring milestone for the project: content
that feels like building a small game, not like carefully driving an ECS facade.
The existing `game-kit` API remains valuable as the advanced content layer; this
document defines the beginner layer that will grow on top of it.

## Current State

The low-level architecture is already split cleanly:

```text
content crates
    -> game-kit facade
    -> game-core / game-map / game-ai / game-combat / game-physics
    -> game-runtime / SDL / Vulkan / audio
```

Content does not operate SDL, Vulkan, renderer resources, raw schedules, or the
runtime directly. The remaining friction is that content still authors many
objects as component tuples and often writes manual ECS traversal for common
gameplay rules.

## Target Beginner API

Beginner content should mostly read in game terms:

```rust
game.player_prefab("player")
    .sprite(assets.player)
    .health(100)
    .moves_with(controls.movement, 130.0)
    .build()?;

game.enemy_prefab("slime")
    .sprite(assets.slime)
    .chases_player()
    .melee(26.0, 6)
    .build()?;

game.use_top_down_game()
    .controls(controls)
    .hit_sound(assets.hit)
    .with_enemy_chase()
    .with_collision()
    .with_camera_follow()
    .build();
```

The first demo should be understandable from one or two files. More advanced
demos can still split assets, maps, systems, prefabs, and tests into modules.

## Authoring Levels

- Beginner API: game-shaped helpers for players, enemies, maps, actions, combat,
  scenes, sound, animation, and simple top-down defaults.
- Advanced content API: the current `game-kit` facade for custom prefabs,
  manual systems, explicit queries, and specialized engine-facing content.
- Engine/runtime API: `game-core`, runtime, renderer, platform, and audio crates;
  not part of normal content authoring.

## Non-Goals

- Do not rewrite the ECS, renderer, runtime loop, or backend crates for the
  beginner authoring milestone.
- Do not introduce Lua, Rhai, macros, or a visual editor before the Rust API
  itself feels game-shaped.
- Do not delete the current advanced API; keep it available for custom content
  and tests.
- Do not make content depend on engine/runtime/backend crates directly.

## Forbidden Beginner-Content APIs

Pure beginner examples, starting with `simple-content`, should avoid:

```text
EntityId
Component
Transform
Velocity
Sprite::new
Collider::box_of
Health::new
MeleeAttack
Faction
AiController
ChaseTarget
PathFollow
Patrol
GameCtx<'_
StartupGameCtx<'_
component::<
component_mut::<
entities_with::<
entities_where::<
for_each
nearest_by_position
nearest_living_with
living_entities_with
fixed_active::<
fixed_systems_are_pause_guarded
```

Those APIs remain valid in the advanced layer and in tests where direct
inspection is appropriate.

## Phases

1. Add this roadmap, tutorial scaffolding, README authoring levels, and future
   boundary-test placeholders.
2. Create `simple-content` as the small beginner pressure-test demo.
3. Split beginner and advanced surfaces inside `game-kit` without breaking the
   current prelude.
4. Move common actor components such as `Name`, `Player`, `Enemy`, `Speed`, and
   `PlayerMovement` into `game-kit`.
5. Add player and enemy prefab builders backed by existing prefab validation.
6. Add high-level player/enemy query, action, and melee-combat helpers.
7. Add top-down default systems for movement, pause, reset, AI, collision,
   combat, camera, and UI.
8. Add high-level test harness helpers so beginner tests are game-shaped.
9. Expand assets, audio, sprite-sheet animation, input presets, scene/map flow,
   diagnostics, debug tools, tutorials, and demo generation.
10. Tighten architecture tests once the beginner APIs are stable.

## Definition Of Done

The beginner authoring milestone is complete when:

- A tiny demo can be created and run without editing the workspace manifest,
  binary demo selection code, or low-level crate dependencies.
- A complete small game can live in one or two files.
- Beginner content does not manually traverse ECS data.
- Common components and gameplay concepts are provided by `game-kit`.
- Common prefabs are authored through game-object builders, not tuple bundles.
- Top-down game loop setup has a preset instead of many manual system calls.
- File-backed sound, sprite-sheet animation, and input presets exist for normal
  first-game expectations.
- Tutorial docs teach the beginner path before architecture docs.

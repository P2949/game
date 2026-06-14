# `#[allow(dead_code)]` Audit

This project is a binary crate, so Rust's dead-code lint flags any `pub` item
that is never reached from `main` (it cannot see a future external consumer the
way a library crate can). Several deliberately-kept items therefore carry
`#[allow(dead_code)]`. This document classifies every one so each remaining allow
has a recorded reason and a removal condition.

Categories:

- **A** — intentional engine API / accessor kept for symmetry or diagnostics.
- **B** — exercised only by `#[cfg(test)]` code (the lint ignores test-only use).
- **C** — placeholder for a planned feature.
- **D** — genuinely dead; should be removed now.

**No item currently falls in category D.** When an allow's removal condition is
met, drop the attribute (and the item if it is truly unused).

## Audio (`src/audio/mixer.rs`)

| Item | Cat | Reason | Remove when |
| ---- | --- | ------ | ----------- |
| `Mixer::master_volume` | A/B | Read-side symmetry with `set_master_volume`; used by tests. | A caller reads master volume at runtime. |
| `Mixer::output_sample_rate` / `output_channels` | A | Diagnostics / format introspection accessors. | A runtime consumer queries the configured output format. |
| `Mixer::sound_count` / `voice_count` | A/B | Introspection accessors; used extensively by tests. | Wired into a runtime audio HUD/diagnostic. |
| `Mixer::max_voices` | A | Reports the voice cap for diagnostics. | Surfaced in diagnostics UI. |
| `Mixer::dropped_voice_count` | A/B | Reads the shared voice-drop counter; used by tests. | A runtime consumer reads it directly (today `AudioSystem` reads the shared handle). |
| `AudioSystem::dropped_voices` | A | Lets the main thread read the voice-drop total directly. | Wired into an on-screen diagnostic. |

## Game (`src/game/`)

| Item | Cat | Reason | Remove when |
| ---- | --- | ------ | ----------- |
| `Camera2D::center` (`camera.rs`) | A/B | Accessor; read by gameplay tests. | A non-test caller needs the center. |
| `collision::move_with_collision` (`collision.rs`) | B | The older discrete integrator, retained for comparison and the tunneling regression test after gameplay moved to `move_with_swept_collision`. | The discrete path is no longer needed for regression coverage. |
| `Game::mode` (`state.rs`) | A/B | Mode accessor; used by tests. | Read by a non-test caller. |
| `Shake::trauma` (`state.rs`) | A/B | Accessor; used by pause/shake tests. | Read by a non-test caller. |
| `Entity::new_solid` (`world.rs`) | C | Constructor for static solids; not used until level data spawns solids as entities. | A solid is built as an `Entity`. |
| `Entity::new_sanitized` (`world.rs`) | B/C | Repairs invalid geometry for data-driven spawns; used by tests. | Level loading adopts the sanitizing path. |
| `Entity::previous_position` (`world.rs`) | A/B | Interpolation accessor; used by tests. | Read by a non-test caller. |
| `Entity::try_set_position` (`world.rs`) | C | Fallible runtime position setter for data-driven paths (level load, scripted teleports). | A runtime/data path calls it. |
| `world::sanitize_size` / `sanitize_entity_geometry` (`world.rs`) | B/C | Support `new_sanitized`; used by tests. | `new_sanitized` is used at runtime. |

## Renderer (`src/renderer/`)

| Item | Cat | Reason | Remove when |
| ---- | --- | ------ | ----------- |
| `VulkanContext::physical_device` field (`context.rs`) | A | RAII/diagnostic field kept with the device it backs. | Needed for a feature query, then it becomes read. |
| `LogicalDevice::queues` (`device.rs`) | A | Queue-family accessor for future multi-queue work. | A caller needs queue families post-construction. |
| `FrameData::command_pool` field (`frame.rs`) | A | Owns the pool so `Drop` frees it and the command buffer; intentionally never read. | Never — required for correct ownership/teardown. |
| `OwnedHandle` accessors (`owned.rs`) | A | Handle getters on RAII wrappers, kept for symmetry. | A caller reads the wrapped handle. |
| `Font::measure_text` / `wrap_text` (`text.rs`) | C | UI layout helpers (sizing, word wrap) not yet wired into rendering. | A HUD/menu needs measured or wrapped text. |
| `TextureEntry::name` field (`texture_registry.rs`) | A | Diagnostic label retained per registered texture. | Logged/surfaced, then it becomes read. |

## Platform (`src/platform/`)

| Item | Cat | Reason | Remove when |
| ---- | --- | ------ | ----------- |
| `Platform::sdl` field (`window.rs`) | A | Keeps the SDL context alive for the window/event pump/audio; also passed to audio init. | Never — required to keep SDL alive. |

# Architectural Improvement Roadmap: Content/Engine Boundary Consolidation

## Purpose

This roadmap is for the current `P2949/game` workspace state from the uploaded `game-master(7).zip`. It turns the remaining review items and polish items into an implementation plan focused on one goal:

> The project is not trying to build a full game yet. The goal is to build the foundation so creative/content code looks and works like game code, while renderer, audio, platform, memory, runtime, ECS, validation, and backend complexity stay delegated to the technical engine layers. Beginner content should read like a simplified Bevy/Raylib-style authoring API, not like low-level engine code.

This plan assumes the existing engine/content split is real and valuable. The work here is not another major architecture rewrite. It is a consolidation pass: narrow the public API surface, split oversized modules, make map/runtime commands harder to misuse, surface runtime errors more clearly, make CLI diagnostics stricter, and make the architecture tests less brittle.

## Current code observations this plan is based on

The current workspace already has the right broad shape:

- `game-core` owns engine-neutral ECS-ish primitives, command queue, schedules, registries, input IDs, world/resources, map IDs, prefab IDs, rendering frame data, and backend traits.
- `game-runtime` owns the frame loop through `Runner<P, R, A>` and is generic over platform, renderer, and audio backends.
- `game-renderer-vulkan`, `game-platform-sdl`, `game-audio`, and `game-backend-headless` are backend/technical crates.
- `game-kit` is the content-authoring facade over `game-core`, `game-map`, `game-ai`, `game-combat`, and `game-physics`.
- `game-starter` is the standalone beginner entry point that depends on runtime and exposes `game_starter::prelude::*`.
- `simple-content` and `arena-content` use `game_kit::beginner::prelude::*`.
- `testbed-content` uses `game_kit::advanced::prelude::*` and is documented as an advanced lab.
- Beginner examples and templates mostly use `game_starter::prelude::*`.

The current code also already has many guardrails:

- `crates/game-core/tests/architecture_boundaries.rs` checks many content/engine boundary rules.
- `game_kit::beginner::prelude::*` exists and avoids raw ECS types.
- `game_kit::advanced::prelude::*` exists for deliberate lower-level content.
- `game_kit::prelude::*` is deprecated as compatibility-only.
- `game_core::internal_prelude` exists for runtime/facade/tests.
- `game-dev`, templates, packaging, validation, generated-project CI, and first-15-minutes checks already exist.
- `game-audio` already has voice-drop counters and main-thread warnings.

The remaining weaknesses are therefore specific, not vague:

1. `game-kit/src/lib.rs` still root-reexports a very broad authoring/API surface even though docs tell new code to use explicit beginner/advanced preludes.
2. `game-core/src/lib.rs` still root-reexports many raw engine internals even though `prelude` and `internal_prelude` now provide clearer groups.
3. Several files are too large and should be split before more features are added:
   - `crates/game-kit/src/data.rs` — about 4.4k lines.
   - `crates/game-audio/src/mixer.rs` — about 3k lines.
   - `crates/game-core/tests/architecture_boundaries.rs` — about 2.5k lines.
   - `crates/game-kit/src/beginner/prefabs.rs` — about 2.2k lines.
   - `crates/game-cli/src/lib.rs` — about 1.7k lines.
   - `crates/game-kit/src/app.rs` — about 1.6k lines.
   - `crates/game-kit/src/beginner/rules.rs` — about 1.6k lines.
4. Map switching currently has both a content-aware path (`GameCtx::change_map(&str)`) and lower-level `Commands::change_map(MapId)`. The content-aware path mutates `ContentRuntime`, clears/respawns the world, and queues a core active-map command. The raw command path can bypass that content-aware state change if exposed to advanced users.
5. Runtime command failures in `game-runtime/src/runner.rs` are mostly logged and then ignored. That is robust for a runtime loop, but weak for beginner diagnostics and CI proof.
6. `game-dev asset-check` currently ignores unknown asset extensions. That can hide typo mistakes like `player.pgn`.
7. `game-dev doctor` is intentionally advisory, but `game-dev check` should have clearer hard/soft prerequisite semantics.
8. Release tag/version strings are repeated in CLI, templates, docs, and architecture tests.
9. `.github/workflows/ci.yml` lacks explicit top-level permissions; `release.yml` already has `contents: write`.
10. Some docs are stale or slightly contradictory. For example, `docs/ARCHITECTURE.md` still describes audio as generated-tone-only even though file-backed sounds and music exist.

## Execution protocol for an LLM implementation agent

Use this protocol for every phase:

1. Re-read this roadmap section before editing.
2. Make the smallest coherent code change for the current item.
3. Update this roadmap by marking the exact item done only after the code and tests/docs for that item are updated.
4. Run the narrowest relevant check first.
5. Run broader checks at the end of each phase.
6. If a check fails, fix the implementation instead of weakening the boundary test unless the test is clearly outdated.
7. Do not add new gameplay features while doing this roadmap. This is consolidation work.
8. Preserve beginner examples and templates. If a breaking API cleanup is required, provide a compatibility window or explicit migration document.
9. Prefer moving code without changing behavior before changing semantics.
10. Keep the content-facing success metric visible: beginner content should use `game_starter::prelude::*` or `game_kit::beginner::prelude::*`; advanced content should use `game_kit::advanced::prelude::*`; no content crate should import engine/backend crates directly.

## Global definition of done

This roadmap is complete when all of the following are true:

- Beginner examples/templates/content still read like game authoring code.
- `simple-content` and `arena-content` production code import only `game_kit::beginner::prelude::*` plus normal local modules.
- `testbed-content` remains explicitly advanced and imports `game_kit::advanced::prelude::*`.
- `game-kit` does not encourage crate-root wildcard usage.
- `game-core` does not encourage root-level raw internal imports.
- Raw map commands cannot accidentally bypass content-runtime map transitions from beginner/advanced content paths.
- Runtime command failures are available as structured diagnostics and can fail strict tests.
- Oversized modules are split into coherent submodules without behavior regressions.
- Architecture tests are split into smaller focused files.
- `game-dev asset-check` catches unknown/likely-mistyped asset files.
- CI has explicit least-privilege permissions.
- Documentation matches the current implementation.
- The release/checklist flow includes these new gates.

Recommended final validation command set:

```bash
cargo fmt --all -- --check
cargo test --workspace --locked --features game/ci-build-sdl3
cargo test -p game-runtime --test headless_runner --no-default-features --locked
cargo clippy --workspace --all-targets --locked --features game/ci-build-sdl3 -- -D warnings
cargo build -p game --release --locked --features ci-build-sdl3
cargo run -p game-cli --features ci-build-sdl3 -- doctor --explain
cargo run -p game-cli --features ci-build-sdl3 -- asset-check
cargo run -p game-cli --features ci-build-sdl3 -- validate-data assets/game.ron
scripts/first-15-minutes.sh
```

If local graphics cannot run smoke tests, use the documented Xvfb/lavapipe path from `docs/release-checklist.md` and record the caveat.

---

# Phase 0 — Baseline and branch safety

## Goal

Create a safe starting point and make sure the current source tree is understood before changing architecture. This phase should not change behavior except for adding the new roadmap document if desired.

## Steps

### 0.1 Create an implementation branch

**Status:** Done on 2026-07-01. Created and switched to `architecture/content-boundary-consolidation`; `git status --short` was clean before edits.

- Create a branch such as:

```bash
git switch -c architecture/content-boundary-consolidation
```

- Confirm the working tree is clean:

```bash
git status --short
```

### 0.2 Run or record the baseline checks

**Status:** Done on 2026-07-01. Baseline checks passed:

- `cargo fmt --all -- --check`
- `cargo test --workspace --locked --features game/ci-build-sdl3`
- `cargo test -p game-runtime --test headless_runner --no-default-features --locked`
- `cargo clippy --workspace --all-targets --locked --features game/ci-build-sdl3 -- -D warnings`

## Baseline notes

No pre-existing failures were observed in the baseline command set above.

Run what is feasible locally:

```bash
cargo fmt --all -- --check
cargo test --workspace --locked --features game/ci-build-sdl3
cargo test -p game-runtime --test headless_runner --no-default-features --locked
cargo clippy --workspace --all-targets --locked --features game/ci-build-sdl3 -- -D warnings
```

If the local machine lacks SDL/Vulkan/system dependencies, at minimum run narrower checks that do not require the graphical path:

```bash
cargo fmt --all -- --check
cargo test -p game-core --locked
cargo test -p game-kit --locked
cargo test -p game-runtime --test headless_runner --no-default-features --locked
```

Record any existing failures in this roadmap under a temporary “Baseline notes” section. Do not mix pre-existing failures with roadmap regressions.

### 0.3 Add this roadmap to the repo

**Status:** Done on 2026-07-01. Added `docs/roadmaps/content-engine-boundary-consolidation.md` and linked it from `docs/roadmaps/README.md`.

Create:

```text
docs/roadmaps/content-engine-boundary-consolidation.md
```

Update:

```text
docs/roadmaps/README.md
```

Add a row like:

```markdown
| Content/engine boundary consolidation | Current | Narrows root APIs, splits large modules, unifies map commands, improves diagnostics, and hardens CI/docs. |
```

Do not overwrite the historical `docs/architectural-improvement-roadmap.md`. That file says the old architecture split is complete, which is true. This new roadmap is the follow-up consolidation pass.

## Done when

- The roadmap exists in `docs/roadmaps/`.
- The roadmap index links it.
- Baseline check status is known.

---

# Phase 1 — Update the architecture contract before code changes

## Goal

Make the intended API layers explicit before moving exports. This prevents the implementation agent from making random cleanup changes that fight the project’s actual public contract.

## Current code basis

Relevant files:

- `crates/game-kit/src/lib.rs`
- `crates/game-kit/src/beginner/prelude.rs`
- `crates/game-kit/src/advanced/prelude.rs`
- `crates/game-core/src/lib.rs`
- `docs/ARCHITECTURE.md`
- `docs/architecture.md`
- `docs/when-to-use-advanced-api.md`
- `docs/roadmaps/post-1.0-api-surface-cleanup.md`
- `README.md`

Current reality:

- New beginner Rust code should use `game_kit::beginner::prelude::*`.
- Standalone projects should use `game_starter::prelude::*`.
- Advanced content should use `game_kit::advanced::prelude::*`.
- `game_kit::prelude::*` exists but is deprecated compatibility surface.
- Root-level `game-kit` reexports are still broad.
- `game-core` root reexports are still broad.

## Steps

### 1.1 Write a short API boundary contract

**Status:** Done on 2026-07-01. Added `docs/api-boundary.md` with the no-Rust, beginner Rust, advanced content, facade/internal, runtime/backend, and compatibility-policy sections.

Create:

```text
docs/api-boundary.md
```

Required sections:

1. `No-Rust data path`
   - Uses `assets/game.ron`.
   - Runtime/backends remain hidden.
   - Schema is versioned through `version: 1` currently.

2. `Beginner Rust path`
   - Uses `game_starter::prelude::*` for standalone demos.
   - Uses `game_kit::beginner::prelude::*` for workspace content crates.
   - Allowed vocabulary: player, enemy, pickup, projectile, map, scene, sound, music, animation, score, UI, rule, event, tag, timer.
   - Forbidden vocabulary in beginner docs/templates: `GameCtx`, `StartupGameCtx`, `EntityId`, `Component`, `World`, `Transform`, `Velocity`, `Sprite::new`, `Collider::box_of`, raw `Commands`, raw registries, runtime/backend crates.

3. `Advanced content path`
   - Uses `game_kit::advanced::prelude::*`.
   - Allows custom ECS-style systems, queries, manual prefab composition, custom resources, and lower-level tests.
   - Still must not import `game-core`, `game-map`, `game-runtime`, `game-renderer-vulkan`, `game-platform-sdl`, or `game-audio` directly from content crates.

4. `Facade/internal path`
   - `game-kit` may use engine-neutral crates.
   - `game-kit` must not depend on runtime/backend crates.

5. `Runtime/backend path`
   - `game-runtime` owns loop orchestration.
   - `game-renderer-vulkan`, `game-platform-sdl`, `game-audio` own backend complexity.
   - Runtime/backends must not depend on content crates.

6. `Compatibility policy`
   - `game_kit::prelude::*` remains deprecated for one compatibility window only.
   - Root reexports should be reduced after migration docs exist.
   - Beginner API renames keep a deprecated alias for one release when feasible.

### 1.2 Update stale docs to point at the new contract

**Status:** Done on 2026-07-01. Linked `docs/api-boundary.md` from `README.md`, `docs/ARCHITECTURE.md`, `docs/architecture.md`, `docs/content-authoring.md`, `docs/advanced-content-authoring.md`, `docs/when-to-use-advanced-api.md`, and `docs/release-checklist.md`.

Update these files to link `docs/api-boundary.md`:

- `README.md`
- `docs/ARCHITECTURE.md`
- `docs/architecture.md`
- `docs/content-authoring.md`
- `docs/advanced-content-authoring.md`
- `docs/when-to-use-advanced-api.md`
- `docs/release-checklist.md`

### 1.3 Fix known stale doc statements

**Status:** Done on 2026-07-01. Updated the `docs/ARCHITECTURE.md` audio mixer description, revised `game-core` rustdoc to point content at explicit beginner/advanced surfaces, clarified `docs/dead-code-audit.md`, and verified with `cargo test -p game-core --test architecture_boundaries --locked`.

At minimum:

- In `docs/ARCHITECTURE.md`, update the `Audio Mixer` section. It currently describes generated-tone-only behavior even though `game-audio` has file-backed WAV/OGG/MP3 validation/loading paths and `game-kit` exposes sounds/music. Rewrite it to say:
  - generated tones exist,
  - file-backed WAV is supported by default,
  - OGG/MP3 are feature-gated where applicable,
  - streaming music exists where supported,
  - the mixer remains backend-owned and content only sees named sound/music operations.

- In `crates/game-core/src/lib.rs` rustdoc, replace references telling content to use `game_kit::prelude::*` with explicit beginner/advanced wording.

- In `docs/dead-code-audit.md`, clarify that `game_kit::prelude::*` is compatibility-only and not “the normal prelude.”

## Done when

- `docs/api-boundary.md` exists and names the allowed import surfaces.
- Stale audio documentation is corrected.
- `game-core` docs no longer teach `game_kit::prelude::*` as normal content API.
- Architecture tests still pass or are intentionally updated to the new wording.

---

# Phase 2 — Narrow `game-kit` root API without breaking the beginner path

## Goal

Stop making `game-kit` crate root look like the whole engine API. Beginner users should discover `game_kit::beginner::prelude::*`; advanced users should discover `game_kit::advanced::prelude::*`; internal implementation should use module paths.

This is the highest-priority API cleanup because `game-kit/src/lib.rs` currently root-reexports many beginner, advanced, asset, map, prefab, data, context, and system types.

## Current code basis

Relevant current root-level exports in `crates/game-kit/src/lib.rs` include:

- `GameApp`, `GamePlugin`, `Plugin`, `plugin`, `plugin_fn`
- asset authors and refs
- beginner actor components and behavior types
- animation/audio/UI/rules/events/types
- `Commands`, `GameCtx`, `StartupGameCtx`
- data-file structs and loader
- map/prefab/system helpers
- deprecated compatibility prelude

Current root use sites found in the repo:

- `crates/testbed-content/src/lib.rs` uses `game_kit::Plugin` and `game_kit::plugin`.
- `crates/game-starter/src/lib.rs` uses `game_kit::GameApp` and `game_kit::plugin_fn`.
- `crates/game-runtime/tests/headless_runner.rs` uses `game_kit::plugin_fn` and `game_kit::testing::GameTestHarness`.
- The `content_plugin!` macro currently expands to `$crate::Plugin`, `$crate::plugin`, `$crate::GamePlugin`, and `$crate::GameApp`.

## Compatibility decision

Do this in two commits:

1. **Preparation commit:** update internal repo uses and macro paths so the root exports are no longer required by the repo itself.
2. **API cleanup commit:** remove or deprecate broad root reexports according to the compatibility policy.

If this branch targets a release that must keep source compatibility, keep a temporary `game_kit::compat` module and migration docs. If this branch is allowed to be breaking, remove broad root reexports directly after docs/tests are updated.

## Steps

### 2.1 Make `content_plugin!` independent of root reexports

**Status:** Done on 2026-07-01. The macro now expands through `$crate::app::{Plugin, plugin, GamePlugin, GameApp}` paths. Verified content/starter compile with `cargo check -p simple-content -p arena-content -p testbed-content -p game-starter --locked`.

Change the macro in `crates/game-kit/src/lib.rs` from root paths to module paths.

Current shape:

```rust
pub fn $plugin_fn() -> $crate::Plugin<$plugin_ty> {
    $crate::plugin($plugin_ty)
}

impl $crate::GamePlugin for $plugin_ty {
    fn build(&self, $game: &mut $crate::GameApp<'_>) -> anyhow::Result<()> {
        ...
    }
}
```

Target shape:

```rust
pub fn $plugin_fn() -> $crate::app::Plugin<$plugin_ty> {
    $crate::app::plugin($plugin_ty)
}

impl $crate::app::GamePlugin for $plugin_ty {
    fn build(&self, $game: &mut $crate::app::GameApp<'_>) -> anyhow::Result<()> {
        ...
    }
}
```

Then confirm `simple-content` and `arena-content` still compile because they use `content_plugin!` through the beginner prelude.

### 2.2 Update root-use call sites inside the repo

**Status:** Done on 2026-07-01. Updated `testbed-content`, `game-starter`, and `game-runtime` headless tests to use explicit `game_kit::app` and `game_kit::testing` paths. `rg "game_kit::(GameApp|GamePlugin|Plugin|plugin\\(|plugin_fn|GameCtx|Commands|TextureRef|SoundRef|MapAuthor|PrefabAuthor)" crates examples templates bin xtask` now reports only an architecture-test forbidden-string fixture.

Change:

- `crates/testbed-content/src/lib.rs`
  - Current: `pub fn plugin() -> game_kit::Plugin<TestbedPlugin> { game_kit::plugin(TestbedPlugin) }`
  - Target: rely on `Plugin` and `plugin` imported from `game_kit::advanced::prelude::*`, or use explicit `game_kit::app::Plugin` / `game_kit::app::plugin`.
  - Preferred for advanced content readability:

```rust
pub fn plugin() -> Plugin<TestbedPlugin> {
    plugin(TestbedPlugin)
}
```

- `crates/game-starter/src/lib.rs`
  - Current: `use game_kit::GameApp;` and `game_kit::plugin_fn(build)`.
  - Target: `use game_kit::app::{GameApp, plugin_fn};` and call `plugin_fn(build)`.

- `crates/game-runtime/tests/headless_runner.rs`
  - Current: `use game_kit::{plugin_fn, testing::GameTestHarness};`
  - Target: `use game_kit::app::plugin_fn; use game_kit::testing::GameTestHarness;`

Search after this step:

```bash
rg "game_kit::(GameApp|GamePlugin|Plugin|plugin\(|plugin_fn|GameCtx|Commands|TextureRef|SoundRef|MapAuthor|PrefabAuthor)" crates examples templates bin xtask
```

Expected result: no production use of root-level `game_kit::...` API except intentional docs/tests/migration references.

### 2.3 Create a clear root-export policy in code

**Status:** Done on 2026-07-01. Removed broad root `pub use` exports from `game-kit`, made `helpers` internal, kept the explicit module/prelude/testing surfaces, moved the former broad root surface under deprecated `game_kit::compat` for the compatibility window, and added `docs/migrations/root-api-cleanup.md`.

In `crates/game-kit/src/lib.rs`, keep only:

```rust
pub mod advanced;
pub mod app;
pub mod assets;
pub mod beginner;
pub mod bundle;
pub mod context;
pub mod data;
pub mod input;
pub mod map;
pub mod prefab;
pub mod system;
```

Private modules remain private:

```rust
mod diagnostics;
mod harness;
mod paths;
```

Keep `content_plugin!`.

Keep one of these two compatibility strategies:

#### Option A — breaking cleanup branch

Remove broad root `pub use` exports entirely.

Pros:

- Strongest architecture signal.
- Prevents new accidental root API usage.

Cons:

- Existing external users of `game_kit::GameApp`, `game_kit::Plugin`, etc. must migrate.

Required docs:

- Add migration note in `docs/migrations/`.
- Update `CHANGELOG.md` under an unreleased/breaking section.

#### Option B — compatibility window

Move broad root exports into a deliberately named module:

```rust
#[deprecated(note = "Use game_kit::beginner::prelude::* or game_kit::advanced::prelude::*")]
pub mod compat {
    ...old broad exports...
}
```

Then either:

- remove root exports, or
- keep only a tiny root set for one release with per-item deprecation attributes.

Pros:

- Gives external users a migration path.

Cons:

- More code remains temporarily.

Preferred approach for this repo: **Option B for one release**, then Option A after the migration window.

### 2.4 Keep beginner and advanced preludes authoritative

**Status:** Done on 2026-07-01. Left `beginner::prelude` and `advanced::prelude` intact while narrowing only the root API. Verified with the existing beginner-prelude architecture guard and `cargo test -p game-kit --locked`.

Do not shrink `beginner::prelude` and `advanced::prelude` in the same commit. First narrow root exports. Then, after tests are stable, separately audit the preludes.

Beginner prelude must not export:

- `GameCtx`
- `StartupGameCtx`
- `Commands`
- `EntityId`
- `Component`
- `Transform`
- `Velocity`
- `Sprite`
- `Collider`
- `Health`
- `MeleeAttack`
- `Faction`
- `AiController`
- `ChaseTarget`
- `PathFollow`
- `Patrol`
- `PrefabAuthor`

Advanced prelude may export those deliberately.

### 2.5 Add architecture tests for root API cleanup

**Status:** Done on 2026-07-01. Added `game_kit_root_does_not_reexport_authoring_surface` to the architecture boundary tests and verified with `cargo test -p game-core --test architecture_boundaries --locked`.

Add or update tests so the root cannot grow again accidentally.

If keeping only modules/macros at root, add a test similar to:

```rust
#[test]
fn game_kit_root_does_not_reexport_authoring_surface() {
    let source = fs::read_to_string(workspace_root().join("crates/game-kit/src/lib.rs")).unwrap();
    let forbidden = [
        "pub use beginner::actors",
        "pub use beginner::rules",
        "pub use context::{Commands, GameCtx, StartupGameCtx}",
        "pub use data::{BeginnerGameFile",
        "pub use prefab::PrefabAuthor",
        "pub use system::{GameSystem, StartupSystem}",
    ];
    for item in forbidden {
        assert!(!source.contains(item), "game-kit root must not reexport {item}");
    }
}
```

If a temporary `compat` module is used, make the test allow root `pub mod compat` but reject broad root `pub use` outside that module.

## Done when

- Repo code no longer depends on broad `game-kit` root exports.
- `content_plugin!` expands through `app::` paths.
- Root API is minimal or compatibility-only by explicit policy.
- Beginner examples still compile with `game_starter::prelude::*`.
- Content crates still compile with beginner/advanced explicit preludes.
- Migration docs explain any root API removal/deprecation.

---

# Phase 3 — Narrow `game-core` root exports

## Goal

Make `game-core` look like an engine-neutral crate with explicit modules and preludes, not a flat bag of raw internals. This supports the core goal by making accidental engine-level usage less convenient.

## Current code basis

`crates/game-core/src/lib.rs` currently:

- declares public modules,
- has `prelude`,
- has `internal_prelude`,
- then root-reexports many internals, including `Ctx`, `StartCtx`, `GameBuilder`, `MapRegistry`, `PrefabRegistry`, `CommandQueue`, `Schedule`, `World`, `EntityId`, etc.

The current repo mostly uses module paths already, such as `game_core::builder::MapId`, `game_core::commands::CommandQueue`, etc. That means broad root reexports are likely no longer needed internally.

## Steps

### 3.1 Update rustdoc wording

**Status:** Done on 2026-07-01. `game-core` rustdoc now points beginner content to `game_kit::beginner::prelude::*` or `game_starter::prelude::*`, advanced content to `game_kit::advanced::prelude::*`, and runtime/facade internals to `game_core::internal_prelude` or explicit module paths.

In `crates/game-core/src/lib.rs`, change:

- “Game content should import `game_kit::prelude::*`”

To:

- “Beginner content should import `game_kit::beginner::prelude::*` or `game_starter::prelude::*`; advanced content should import `game_kit::advanced::prelude::*`. Runtime/facade internals use `game_core::internal_prelude` or explicit module paths.”

### 3.2 Remove broad root `pub use` block

**Status:** Done on 2026-07-01. Removed the broad root `pub use` block from `crates/game-core/src/lib.rs`; public modules plus `prelude` and `internal_prelude` remain.

Remove or deprecate these root exports from `game-core/src/lib.rs`:

- `pub use app::{Ctx, MapData, RenderFrame, StartCtx, ...}`
- `pub use assets::{AssetRegistry, AssetValidator}`
- `pub use audio::{Audio, AudioCommands}`
- `pub use backend::{...}`
- `pub use builder::{GameBuilder, MapId, MapRegistry, ...}`
- `pub use commands::{Command, CommandQueue}`
- `pub use query::{...}`
- `pub use schedule::{...}`
- `pub use world::{...}`

Keep public modules and the two grouped preludes.

If external compatibility is a concern, move the root exports into:

```rust
#[deprecated(note = "Use explicit game_core modules, game_core::prelude, or game_core::internal_prelude")]
pub mod compat { ... }
```

But prefer no compatibility module for `game-core` unless external users exist, because content is not supposed to consume it directly.

### 3.3 Update any failed internal imports

**Status:** Done on 2026-07-01. `cargo check --workspace --locked --features game/ci-build-sdl3` passed after the root export removal, so no internal import repairs were needed.

Run:

```bash
cargo check --workspace --locked --features game/ci-build-sdl3
```

If errors occur from removed root exports, convert them to explicit module paths. Do not re-add broad root exports just to quiet errors.

### 3.4 Add an architecture test

**Status:** Done on 2026-07-01. Added `game_core_root_does_not_reexport_internal_surface` and verified it with `cargo test -p game-core --test architecture_boundaries --locked`.

Add a test that rejects broad root `pub use` in `game-core/src/lib.rs`, while allowing:

- `pub mod prelude`
- `pub mod internal_prelude`
- public module declarations

## Done when

- `game-core` root is not a flat export surface.
- Runtime/facade crates use explicit `game_core::module::Type` paths or `internal_prelude` intentionally.
- Content crates still do not depend on `game-core`.

---

# Phase 4 — Unify map transition semantics and remove split-brain risk

## Goal

Make it impossible for beginner/advanced content to switch the runtime active map without also updating `ContentRuntime` and respawning the correct content objects.

## Current code basis

Current paths:

- `GameCtx::change_map(&str)` in `crates/game-kit/src/context.rs`:
  - calls `change_to_map_world(self.inner.world, map)`,
  - updates `ContentRuntime.current_map`,
  - clears world and command queue,
  - spawns current map objects,
  - queues `commands().change_map(map_id)`.

- `Commands::change_map(MapId)` in `crates/game-kit/src/context.rs`:
  - directly queues `CommandQueue::change_map(MapId)`.

- `Command::ChangeMap(MapId)` in `crates/game-core/src/commands.rs`:
  - consumed by `game-runtime/src/runner.rs` to switch `active_map`.

Risk:

An advanced content author can use raw `Commands::change_map(MapId)` and bypass the content-aware world respawn/current-map update. The runtime active tilemap and `ContentRuntime.current_map` can become inconsistent.

## Design decision

Keep `Command::ChangeMap(MapId)` as a core/runtime internal command because the runtime needs an active map command. But do not expose raw `Commands::change_map(MapId)` as normal content API.

The content-facing map transition should be by name:

```rust
game.change_map("level_2")?;
game.change_map_or_log("level_2");
```

Optionally add a nicer beginner alias later:

```rust
game.go_to_map("level_2")?;
```

But do not rename first; first fix semantics.

## Steps

### 4.1 Make raw `Commands::change_map(MapId)` crate-private or remove it from `game-kit::Commands`

**Status:** Done on 2026-07-01. Replaced public `Commands::change_map(MapId)` with crate-private `queue_active_map_change_unchecked(MapId)`.

In `crates/game-kit/src/context.rs`, change the public method:

```rust
pub fn change_map(&mut self, map: game_core::builder::MapId)
```

To one of:

```rust
pub(crate) fn change_active_map_unchecked(&mut self, map: game_core::builder::MapId)
```

or remove it entirely if only `GameCtx::change_map` and reload paths need it.

Preferred:

- Rename to `queue_active_map_change_unchecked`.
- Make it `pub(crate)`.
- Document that it assumes `ContentRuntime` and world spawning have already been updated.

### 4.2 Update internal call sites

**Status:** Done on 2026-07-01. `GameCtx::change_map(&str)` now updates the content runtime through `change_to_map_world(...)` and then calls the crate-private queue helper.

In `GameCtx::change_map`, do not call the public raw command method. Call the lower-level queue directly through a crate-private helper:

```rust
self.commands().queue_active_map_change_unchecked(map_id);
```

or:

```rust
self.inner.world.resource_or_insert_with(CommandQueue::new).change_map(map_id);
```

Prefer the helper to keep command queue access centralized.

### 4.3 Audit advanced prelude and docs

**Status:** Done on 2026-07-01. `advanced::prelude` still exposes `GameCtx`/`Commands` for advanced operations, but `Commands` no longer has public raw map switching. `docs/advanced-content-authoring.md` now tells advanced users to use name-based `GameCtx::change_map("level_2")`.

Ensure `game_kit::advanced::prelude::*` does not encourage raw `Commands::change_map(MapId)`.

It may still expose `GameCtx` and `Commands` for other advanced operations, but map transition docs should say:

- use `GameCtx::change_map("name")`, not raw `Commands`.
- raw active map command is runtime/facade internal only.

### 4.4 Add tests proving the raw bypass is gone

**Status:** Done on 2026-07-01. Existing `change_map_switches_content_runtime_and_queues_active_map_command` remains green, and the new architecture test `game_kit_commands_do_not_expose_raw_map_change` guards against reintroducing public raw `MapId` switching. Verified with `cargo test -p game-kit --locked` and `cargo test -p game-core --test architecture_boundaries --locked`.

Add/modify tests in either `crates/game-kit/src/context.rs` or architecture tests:

1. Existing test should remain:
   - `change_map_switches_content_runtime_and_queues_active_map_command`.

2. Add a source-boundary test:
   - `crates/game-kit/src/context.rs` must not contain `pub fn change_map(&mut self, map: game_core::builder::MapId)` inside `impl Commands`.

3. Add an advanced content compile/use test if practical:
   - Advanced content can call `game.change_map("second")`.
   - Advanced content cannot call `game.commands().change_map(MapId(...))` because the method is no longer public.

### 4.5 Consider adding a clearer beginner alias

**Status:** Done on 2026-07-01. Added beginner callback aliases `go_to_map` and `go_to_map_or_log`, while keeping `change_map` and `change_map_or_log` for compatibility.

After semantics are safe, optionally add:

```rust
impl Game<'_, '_, '_> {
    pub fn go_to_map(&mut self, map: &str) -> Result<()> { self.change_map(map) }
    pub fn go_to_map_or_log(&mut self, map: &str) { self.change_map_or_log(map) }
}
```

This is polish, not required. If added, docs should use the more game-shaped name in beginner tutorials, while `change_map` remains for compatibility.

## Done when

- Content-facing code cannot queue raw active-map changes by `MapId`.
- Name-based map changes still update `ContentRuntime`, respawn map objects, and queue the runtime active-map switch.
- Tests cover the content-aware path.
- Docs steer users to name-based map transitions.

---

# Phase 5 — Add structured runtime command errors

## Goal

A bad runtime command should not only produce a log line. It should be visible to tests, debug overlays, and strict development modes. Beginner mistakes should fail loudly in CI/harnesses.

## Current code basis

`game-runtime/src/runner.rs` has `process_core_commands(...) -> bool` that logs errors for:

- failed prefab command spawn,
- failed active map switch,
- failed map reload replacement,
- mismatched map reload data,
- missing map reload data,
- failed restart map,
- failed restart start map.

It returns only `quit: bool`.

`game-kit/src/harness.rs` handles similar command paths and often panics/expect-fails in tests, which is stricter than production runtime.

Existing core status resource pattern:

- `AssetReloadStatus` lives in `game-core::commands` so runtime status can be displayed by content-facing debug overlays without runtime depending on `game-kit`.

Use that same pattern.

## Steps

### 5.1 Add command error types to `game-core`

**Status:** Done on 2026-07-01. Added `CommandErrorKind`, `CommandError`, and `CommandErrors` to `game-core::commands`, including order/last/clear tests.

In `crates/game-core/src/commands.rs`, add:

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommandErrorKind {
    SpawnPrefab,
    ChangeMap,
    ReloadMap,
    ReloadAssets,
    RestartMap,
    RestartStartMap,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandError {
    pub kind: CommandErrorKind,
    pub message: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CommandErrors {
    errors: Vec<CommandError>,
}
```

Add methods:

```rust
impl CommandErrors {
    pub fn push(&mut self, kind: CommandErrorKind, message: impl Into<String>);
    pub fn is_empty(&self) -> bool;
    pub fn len(&self) -> usize;
    pub fn iter(&self) -> impl Iterator<Item = &CommandError>;
    pub fn clear(&mut self);
    pub fn last(&self) -> Option<&CommandError>;
}
```

Keep this in `game-core` because runtime writes it and game-kit/debug overlay may read it.

### 5.2 Add a runtime policy

**Status:** Done on 2026-07-01. Added `CommandErrorPolicy` and `RuntimeConfig::command_error_policy(...)`; the default policy is `StoreResource`.

In `game-runtime/src/runner.rs`, add:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandErrorPolicy {
    LogAndContinue,
    StoreResource,
    PanicInDebug,
    ReturnError,
}
```

Add to `RuntimeConfig`:

```rust
command_error_policy: CommandErrorPolicy,
```

Default should be:

```rust
CommandErrorPolicy::StoreResource
```

or, if you want minimal behavior change:

```rust
CommandErrorPolicy::LogAndContinue
```

Recommended for this framework: `StoreResource`. It preserves runtime resilience while making errors visible.

Add builder method:

```rust
pub fn command_error_policy(mut self, policy: CommandErrorPolicy) -> Self
```

For tests/headless generated checks, use `ReturnError` or `PanicInDebug`.

### 5.3 Change `process_core_commands` to return a result object

**Status:** Done on 2026-07-01. `process_core_commands` now returns `CommandProcessOutcome { quit, errors }`; command failures are collected and policy-handled after all commands are processed.

Replace:

```rust
fn process_core_commands(...) -> bool
```

With something like:

```rust
struct CommandProcessOutcome {
    quit: bool,
    errors: Vec<CommandError>,
}
```

or:

```rust
fn process_core_commands(...) -> Result<CommandProcessOutcome>
```

But avoid making every logged recoverable command immediately abort unless policy says so.

Implementation pattern:

1. Drain commands.
2. For each error, call a helper:

```rust
record_command_error(world, &mut errors, CommandErrorKind::ChangeMap, format!(...));
```

3. At the end, apply policy:
   - `LogAndContinue`: log only.
   - `StoreResource`: log and insert/update `CommandErrors` resource.
   - `PanicInDebug`: panic in debug builds, otherwise store/log.
   - `ReturnError`: return `Err(anyhow!(...))` after recording all errors.

### 5.4 Store errors as a resource

**Status:** Done on 2026-07-01. `StoreResource`, release-mode `PanicInDebug`, and `ReturnError` policies append failures to the `CommandErrors` world resource without clearing previous errors.

When an error occurs, insert or update:

```rust
world.resource_or_insert_with(CommandErrors::default).push(...)
```

Do not clear previous errors automatically unless there is a clear reason. Debug overlay should be able to show last error.

Optional: add `CommandErrors::take()` or `drain()` later if needed.

### 5.5 Wire policy into `Runner`

**Status:** Done on 2026-07-01. `Runner` stores the configured policy, applies it after fixed/update command processing, and applies it to asset reload command failures.

Add a field to `Runner`:

```rust
command_error_policy: CommandErrorPolicy,
```

Initialize from `RuntimeConfig`.

Update both `process_core_commands` calls in `step_frame`.

If `ReturnError` is active and command processing fails, `step_frame` should return `Err(...)`.

### 5.6 Update headless harness strictness

**Status:** Done on 2026-07-01. The game-kit harness keeps its existing strict expectations. Runtime unit tests now cover stored command errors, strict `ReturnError`, and the public `RuntimeConfig::command_error_policy(...)` builder. Verified with `cargo test -p game-runtime --locked` and `cargo test -p game-runtime --test headless_runner --no-default-features --locked`.

`game-kit/src/harness.rs` already panics/expect-fails on many command errors. Keep that behavior.

Add tests that prove production `Runner` can run with strict error policy using headless backends.

Possible test in `crates/game-runtime/tests/headless_runner.rs`:

- Create a plugin that queues a bad active map command or bad prefab command through an internal test-only path.
- Run with `RuntimeConfig::default().command_error_policy(CommandErrorPolicy::ReturnError)`.
- Assert `step_frame(...)` returns `Err` with useful message.

If generating a bad command through public APIs is impossible after Phase 4, create the test inside `game-runtime` unit tests where direct `CommandQueue` insertion is available.

### 5.7 Surface in debug overlay

**Status:** Done on 2026-07-01. The beginner debug overlay now shows the last `CommandErrors` entry as `Runtime command error: ...`; verified with `overlay_reports_runtime_command_errors` in `cargo test -p game-kit --locked`.

In `game-kit` debug overlay code, read `CommandErrors` from the world and show the last command error in development overlay if present.

Keep wording user-facing:

```text
Runtime command error: failed to change map to 'level_2': unknown map id MapId(7)
```

Do not show raw backtrace or huge `Debug` output in the overlay.

## Done when

- Runtime command failures are represented by a `CommandErrors` resource.
- Strict policy can fail tests.
- Default policy still keeps the game loop resilient unless deliberately configured otherwise.
- Debug overlay or diagnostics can surface the last command error.
- Existing runner tests and game-kit harness tests still pass.

---

# Phase 6 — Split oversized `game-kit/src/data.rs`

## Goal

Reduce `game-kit/src/data.rs` from a 4.4k-line god module into cohesive modules while preserving behavior. This file currently mixes data schema, parsing, building, runtime reload identity, validation, effect application, defaults, legacy parsing, and tests.

## Current code basis

Current logical areas in `data.rs`:

- reload identity/runtime structs near the top,
- public `load_beginner_game_file` / `validate_beginner_game_file`,
- file-shaped RON schema types,
- action parsing and unknown action diagnostics,
- map/prefab/audio/action/rule build functions,
- runtime audio/effect application,
- file validation functions,
- script rule validation,
- defaults and legacy rule names,
- many tests.

## Target module layout

Create directory:

```text
crates/game-kit/src/data/
  mod.rs
  schema.rs
  load.rs
  build.rs
  validate.rs
  effects.rs
  reload.rs
  defaults.rs
  legacy.rs
  diagnostics.rs
  tests.rs           # optional, or keep unit tests near modules
```

Recommended ownership:

- `schema.rs`
  - `BeginnerGameFile`
  - `BeginnerAssetsFile`
  - `BeginnerControlsFile`
  - `BeginnerPrefabFile`
  - `PlayerPrefabFile`, `EnemyPrefabFile`, etc.
  - `BeginnerMapFile`, `TextMapFile`, `TiledMapFile`, `LdtkMapFile`
  - `AudioFile`, `MusicPlaybackFile`
  - `BeginnerActionFile`, `RuleEffectFile`, `RuleConditionFile`, `BeginnerRuleFile`, etc.

- `reload.rs`
  - `BeginnerReloadLevel`
  - `BeginnerFileRuntime`
  - `BeginnerReloadIdentity`
  - `SceneFlowIdentity`
  - `BeginnerRuleIdentity`
  - `BeginnerActionIdentity`
  - `BeginnerRuntimeConfig`
  - `BeginnerCountdownRuleConfig`
  - `BeginnerCountdownEffectConfig`
  - reload comparison helpers like `ensure_same_list` and `ensure_same_values` if only used for reload compatibility.

- `load.rs`
  - `load_beginner_game_file`
  - `validate_beginner_game_file`
  - `load_beginner_game_text`
  - `load_beginner_game_text_with_base`
  - `LoadedBeginnerGameFile`
  - `read_beginner_game_file`
  - `parse_beginner_game_source`
  - `build_beginner_game_file`

- `build.rs`
  - `build_prefab`
  - `build_map`
  - `build_scene_flow`
  - `build_audio`
  - `build_rule_ui_text`
  - `build_actions`
  - `register_runtime_player_shoots_action`
  - `build_custom_rules`
  - `build_script_rule`
  - `apply_rule`

- `effects.rs`
  - `RuntimeAudioState`
  - `RuntimeMusicPlayback`
  - `apply_runtime_audio`
  - `fire_runtime_player_shot`
  - `script_condition_active`
  - `apply_game_rule_effects`
  - `apply_enemy_death_rule_effects`
  - effect-scope validation helpers if more cohesive there.

- `validate.rs`
  - `validate_file`
  - `validate_file_with_base`
  - `reject_duplicates`
  - `require_known`
  - `validate_map_file`
  - `validate_text_map_symbols`
  - `require_asset_file`
  - `validate_scene_flow`
  - `validate_audio`
  - `validate_actions`
  - `ValidationNames`
  - `PrefabDataIndex`
  - `validate_custom_rules`
  - `validate_countdown_key_for_tag`
  - scalar validators like `validate_radius`, `validate_text`, `validate_non_negative`, etc.

- `diagnostics.rs`
  - `joined_strings_or_none`
  - `joined_names_or_none`
  - unknown action/rule messages
  - maybe shared formatting helpers.

- `legacy.rs`
  - legacy string rule parsing.
  - `LEGACY_RULES`
  - `legacy_rule_kind`.

- `defaults.rs`
  - all default functions such as `default_beginner_game_version`, `default_player_speed`, etc.

## Steps

### 6.1 Convert file to module directory with no behavior changes

**Status:** Done on 2026-07-01. Moved `crates/game-kit/src/data.rs` to `crates/game-kit/src/data/mod.rs` unchanged and verified with `cargo test -p game-kit --locked`.

1. Create `crates/game-kit/src/data/mod.rs`.
2. Move the entire content of `data.rs` into `data/mod.rs`.
3. Delete or leave no `data.rs` file.
4. Confirm `pub mod data;` in `game-kit/src/lib.rs` resolves the directory module.
5. Run:

```bash
cargo test -p game-kit --locked
```

This is a mechanical safety step.

### 6.2 Move schema types first

**Status:** Done on 2026-07-01. Moved the file-shaped schema and action-name parsing into `crates/game-kit/src/data/schema.rs`, reexported it through `game_kit::data::*`, and verified with `cargo test -p game-kit --locked`.

Move file-shaped `pub struct`/`pub enum` definitions into `schema.rs`.

In `mod.rs`:

```rust
mod schema;
pub use schema::{BeginnerGameFile, BeginnerAssetsFile, ...};
```

Keep public reexports identical so callers do not break.

Run:

```bash
cargo test -p game-kit --locked
```

### 6.3 Move defaults and legacy parsing

**Status:** Done on 2026-07-01. Moved serde default helpers into `crates/game-kit/src/data/defaults.rs` and legacy rule parsing into `crates/game-kit/src/data/legacy.rs`, with visibility scoped to the `data` module. Verified with `cargo test -p game-kit --locked`.

Move default functions and legacy rule parsing into `defaults.rs` and `legacy.rs`.

Use `pub(crate)` for helpers unless they are public schema defaults required by serde from another module.

Run tests.

### 6.4 Move validation

**Status:** Done on 2026-07-01. Moved validation entry helpers and validation diagnostics into `crates/game-kit/src/data/validate.rs`, added module docs identifying validation as the beginner safety layer, routed `mod.rs` through `validate::validate_file_with_base`, and verified with `cargo test -p game-kit --locked`.

Move validation functions into `validate.rs`.

Keep public entry point:

```rust
pub fn validate_beginner_game_file(...)
```

in `load.rs` or `mod.rs`, but call `validate::validate_file_with_base` internally.

Add module-level documentation explaining validation is the beginner safety layer.

Run tests.

### 6.5 Move build/apply/effects

**Status:** Done on 2026-07-01. Moved content construction into `crates/game-kit/src/data/build.rs` and runtime/effect callbacks into `crates/game-kit/src/data/effects.rs`, keeping cross-module functions scoped as `pub(super)`. Verified with `cargo test -p game-kit --locked`.

Move build functions into `build.rs` and effect/runtime functions into `effects.rs`.

Be careful with visibility. Many things should be `pub(crate)`, not `pub`.

Run tests.

### 6.6 Split tests if useful

**Status:** Done on 2026-07-01. Moved the large inline `data` test module into `crates/game-kit/src/data/tests.rs` and kept `#[cfg(test)] mod tests;` in `data/mod.rs`. Verified with `cargo test -p game-kit --locked`.

If current tests are huge, either:

- keep a `#[cfg(test)] mod tests;` in `data/mod.rs`, or
- place tests near the modules they verify.

Prefer module-local tests for validation diagnostics and schema parsing.

### 6.7 Add a size guard

**Status:** Done on 2026-07-01. Added `game_kit_data_module_files_stay_split` to `crates/game-core/tests/architecture_boundaries.rs`; it rejects a restored `crates/game-kit/src/data.rs` monolith and caps each `crates/game-kit/src/data/*.rs` file at 1,500 lines. Verified with `cargo test -p game-core --test architecture_boundaries --locked`, `cargo test -p game-kit --locked`, and `cargo run -p game-cli --features ci-build-sdl3 -- validate-data assets/game.ron`.

Add an architecture test or simple script-like test that warns if any one `game-kit/src/data/*.rs` file grows past a reasonable limit, for example 1,500 lines.

Do not make this too strict at first. The purpose is to prevent returning to one 4k-line file.

## Done when

- `crates/game-kit/src/data.rs` is gone or tiny.
- Data schema, load/build/validate/effects/reload concerns are separated.
- Public API remains compatible through `game_kit::data::*` reexports.
- Existing data-driven examples and `validate-data` still pass.

---

# Phase 7 — Split oversized `game-audio/src/mixer.rs`

## Goal

Keep audio backend complexity delegated to the audio crate, but split the 3k-line mixer into coherent backend submodules. This makes future audio fixes safer without exposing audio internals to content.

## Current code basis

`mixer.rs` currently owns:

- mixer constants and voice limits,
- `PlayResult`, `Sound`, `Voice`, `VoiceSource`, fades,
- static sound playback,
- streaming music worker and ring buffers,
- `AudioSystem`, SDL callback, command queue,
- dropped-buffer and dropped-voice diagnostics,
- file sound validation/loading/decoding,
- WAV/OGG/MP3 helpers,
- resampling/channel conversion,
- many tests.

## Target module layout

Create:

```text
crates/game-audio/src/mixer/
  mod.rs
  sound.rs
  voice.rs
  mixer_core.rs
  system.rs
  callback.rs
  command.rs
  diagnostics.rs
  decode.rs
  wav.rs
  ogg.rs
  mp3.rs
  stream.rs
  tests.rs          # optional
```

Recommended ownership:

- `command.rs`
  - internal `AudioCommand`
  - command queue capacity constants if local to command submission.

- `sound.rs`
  - `Sound`
  - `SoundId`
  - `sanitize_volume` if shared.

- `voice.rs`
  - `Voice`
  - `VoiceSource`
  - `VoiceFade`
  - `MusicFade` if not better in mixer core.

- `mixer_core.rs`
  - `Mixer`
  - `PlayResult`
  - voice mixing logic
  - volume/bus/music control
  - dropped voice counter

- `stream.rs`
  - `StreamState`
  - `MusicStream`
  - `StreamedPcm16Wav`
  - worker thread logic.

- `decode.rs`
  - `validate_file_sound`
  - `load_file_sound`
  - `decode_file_sound`
  - `detect_sound_format`
  - `normalize_sound`
  - `convert_channels`
  - `resample_linear`

- `wav.rs`, `ogg.rs`, `mp3.rs`
  - format-specific decode and error helpers.

- `system.rs`
  - `AudioSystem`
  - `AudioBackend` impl
  - reload sound logic
  - bus name map
  - public diagnostics polling.

- `callback.rs`
  - `MixerCallback`
  - SDL callback implementation.

- `diagnostics.rs`
  - `crossed_power_of_two`
  - dropped frame/voice diagnostic helpers if not methods.

## Steps

### 7.1 Mechanical module conversion

**Status:** Done on 2026-07-01. Moved `crates/game-audio/src/mixer.rs` unchanged to `crates/game-audio/src/mixer/mod.rs`; `crates/game-audio/src/lib.rs` still exposes `pub mod mixer` and the root `AudioSystem`/`validate_file_sound` reexports. Verified with `cargo test -p game-audio --locked`.

1. Create `crates/game-audio/src/mixer/mod.rs`.
2. Move existing `mixer.rs` contents into `mixer/mod.rs` unchanged.
3. Remove old `mixer.rs` or leave as a tiny forwarding module only if needed.
4. Ensure `crates/game-audio/src/lib.rs` still exposes the same public API.

Run:

```bash
cargo test -p game-audio --locked
```

### 7.2 Move pure sound/voice types

**Status:** Done on 2026-07-01. Moved `Sound`, `SoundId`, `PlayResult`, and `sanitize_volume` into `crates/game-audio/src/mixer/sound.rs`, moved `Voice`, `VoiceSource`, `VoiceFade`, and `MusicFade` into `crates/game-audio/src/mixer/voice.rs`, and reexported the public types through `mixer/mod.rs`. Verified with `cargo test -p game-audio --locked`.

Move `Sound`, `SoundId`, `Voice`, `VoiceSource`, fade structs, and `PlayResult` into their modules.

Keep `pub use` in `mixer/mod.rs` so public callers still use the same paths.

Run tests.

### 7.3 Move decoding and validation

**Status:** Done on 2026-07-01. Moved file-backed sound validation, format detection, decode, normalization, channel conversion, resampling, and WAV/OGG/MP3 decode helpers into `crates/game-audio/src/mixer/decode.rs`; `game_audio::validate_file_sound` and `game_audio::mixer::validate_file_sound` remain reexported. Verified with `cargo test -p game-audio --locked` and `cargo run -p game-cli --features ci-build-sdl3 -- asset-check`.

Move file decoding functions. Keep this public function path stable:

```rust
game_audio::validate_file_sound(path)
```

If currently exported through `game_audio::mixer::validate_file_sound` or root, preserve that export.

Run:

```bash
cargo test -p game-audio --locked
cargo run -p game-cli --features ci-build-sdl3 -- asset-check
```

### 7.4 Move streaming worker

**Status:** Done on 2026-07-01. Moved `StreamId`, `StreamState`, `MusicStream`, `StreamedPcm16Wav`, streamed-music validation/opening, and the background worker into `crates/game-audio/src/mixer/stream.rs`. The mixer callback still only consumes predecoded samples from the lock-free queue; file I/O and logging remain in the worker. Verified with `cargo test -p game-audio --locked`.

Move streaming music state and worker logic into `stream.rs`.

Ensure callback does not allocate, lock unnecessarily, or log from realtime path.

Run audio tests.

### 7.5 Move `AudioSystem` and callback

**Status:** Done on 2026-07-01. Moved internal mixer commands into `crates/game-audio/src/mixer/command.rs`, SDL callback handling into `crates/game-audio/src/mixer/callback.rs`, and `AudioSystem` plus its `AudioBackend` impl into `crates/game-audio/src/mixer/system.rs`; `mixer/mod.rs` reexports `AudioSystem`. Verified with `cargo test -p game-audio --locked` and `cargo test -p game-runtime --test headless_runner --no-default-features --locked`.

Move SDL-facing `AudioSystem` to `system.rs` and `MixerCallback` to `callback.rs`.

Keep `AudioBackend for AudioSystem` unchanged.

Run:

```bash
cargo test -p game-audio --locked
cargo test -p game-runtime --test headless_runner --no-default-features --locked
```

### 7.6 Preserve and improve voice-drop diagnostics

**Status:** Done on 2026-07-01. Preserved `poll_dropped_voices` diagnostics and added an `Audio Mixer` note to `docs/ARCHITECTURE.md` explaining voice-cap drops, runtime warnings, and beginner guidance for avoiding per-frame sound spam. Verified with `cargo test -p game-audio --locked` and `cargo test -p game-core --test architecture_boundaries --locked`.

The current code already logs voice-cap drops through `poll_dropped_voices`. Keep this behavior.

Add a short docs note in `docs/ARCHITECTURE.md` or `docs/tutorials/common-errors.md`:

- If too many sounds are played at once, audio may drop requests at the voice cap.
- The runtime logs a warning with count and cap.
- Beginner content should avoid firing sound every frame.

## Done when

- `mixer.rs` is split into focused modules.
- Public audio validation/playback behavior is unchanged.
- Voice-drop diagnostics still work.
- No audio internals leak into `game-kit` or content crates.

---

# Phase 8 — Split `game-cli/src/lib.rs` and strengthen asset/check diagnostics

## Goal

Make `game-dev` easier to maintain and make beginner mistakes harder to miss.

## Current code basis

`game-cli/src/lib.rs` currently mixes:

- command parsing,
- template generation,
- xtask entrypoints,
- release checks,
- cargo command execution,
- packaging,
- asset validation,
- doctor diagnostics,
- platform install hints,
- package metadata parsing,
- path helpers.

`validate_asset_file` silently accepts unknown extensions through `_ => {}`.

## Target module layout

Create:

```text
crates/game-cli/src/
  lib.rs
  args.rs
  templates.rs
  commands/
    mod.rs
    new.rs
    run.rs
    check.rs
    package.rs
    release_check.rs
    doctor.rs
    asset_check.rs
    validate_data.rs
  assets.rs
  package.rs
  process.rs
  platform.rs
  manifest.rs
  paths.rs
  errors.rs
```

Keep `main.rs` unchanged except it calls `game_cli::run`.

## Steps

### 8.1 Mechanical split with no behavior change

**Status:** Done on 2026-07-01. Split `game-cli` into focused modules while keeping `run(...)` and `run_xtask(...)` as the dispatch API in `lib.rs`: templates/project generation moved to `templates.rs`, process helpers to `process.rs`, path helpers to `paths.rs`, package metadata parsing to `manifest.rs`, asset validation to `assets.rs`, and check/package/release-check/doctor logic to `commands/`. Verified with `cargo test -p game-cli --locked --features ci-build-sdl3` and `cargo run -p game-cli --features ci-build-sdl3 -- doctor --explain`.

First move obvious groups without semantic changes:

- template constants and `DemoTemplate` to `templates.rs`,
- command process helpers to `process.rs`,
- path helpers to `paths.rs`,
- package metadata helpers to `manifest.rs`,
- doctor functions to `commands/doctor.rs`,
- package functions to `commands/package.rs` or `package.rs`,
- asset validation to `assets.rs` or `commands/asset_check.rs`.

Keep public functions:

```rust
pub fn run(...)
pub fn run_xtask(...)
```

in `lib.rs` as the command dispatch API.

Run:

```bash
cargo test -p game-cli --locked --features ci-build-sdl3
cargo run -p game-cli --features ci-build-sdl3 -- doctor --explain
```

### 8.2 Add unknown asset extension policy

**Status:** Done on 2026-07-01. Added `AssetCheckOptions` with deny-unknown behavior as the default asset validation path. `game-dev asset-check`, `game-dev check`, and `game-dev package` now deny unknown runtime asset files by default, with `.gitignore` and `.DS_Store` ignored; package validation now runs before the release build. Unknown files report the supported beginner asset types and typo hints such as `Did you mean '.png'?`. Verified with `cargo test -p game-cli --locked --features ci-build-sdl3` and `cargo run -p game-cli --features ci-build-sdl3 -- asset-check`.

Introduce:

```rust
enum UnknownAssetPolicy {
    Warn,
    Deny,
}
```

or simpler:

```rust
struct AssetCheckOptions {
    deny_unknown: bool,
}
```

For beginner-facing commands, use deny by default:

- `game-dev asset-check`: deny unknown files.
- `game-dev check`: deny unknown files.
- `game-dev package`: deny unknown files before packaging.

Allow explicit ignore rules to avoid false positives:

- ignore dotfiles such as `.gitignore`, `.DS_Store` if desired,
- ignore known metadata files only when they are intentionally supported,
- allow a project file such as `assets/.gameignore` later if necessary, but do not add that complexity unless needed.

Supported asset extensions should include at least:

- `.png`
- `.ttf`
- `.wav`
- `.ogg`
- `.mp3`
- `.txt`
- `.tmx`
- `.ldtk`
- `game.ron`
- animation metadata `.ron` under `assets/animations/`

Unknown extension error should be beginner-friendly:

```text
unknown asset file 'assets/textures/player.pgn'

Did you mean '.png'?
Supported beginner asset types: .png, .ttf, .wav, .ogg, .mp3, .txt, .tmx, .ldtk, assets/game.ron, and animation .ron files under assets/animations/.
Move notes/source files outside assets/ or add an explicit ignore rule when ignore support exists.
```

### 8.3 Add tests for unknown extension detection

**Status:** Done on 2026-07-01. Added `game-cli` asset validator tests for `assets/textures/player.pgn` suggesting `.png`, `assets/readme.md` failing as unknown, `.gitignore` inside assets being allowed, simple/data-driven template assets passing validation, animation metadata `.ron` routing through the animation validator, and arbitrary `assets/foo.ron` failing. Verified with `cargo test -p game-cli --locked --features ci-build-sdl3` and `cargo run -p game-cli --features ci-build-sdl3 -- asset-check`.

Add tests in `game-cli` for:

- `assets/textures/player.pgn` fails with “Did you mean '.png'?”
- `assets/readme.md` fails or is ignored only if policy says so.
- `.gitignore` inside assets is allowed if you choose that exception.
- valid template assets pass.
- `assets/animations/player.ron` is validated as animation metadata.
- arbitrary `assets/foo.ron` fails unless it is `assets/game.ron` or animation metadata.

### 8.4 Clarify doctor versus check

**Status:** Done on 2026-07-01. Clarified `game-dev check` output so the doctor phase is explicitly advisory and the hard gates are assets, optional data validation, and `cargo check`. Documented the hard/soft semantics in `README.md` and `docs/tutorials/common-errors.md`. Verified with `cargo test -p game-cli --locked --features ci-build-sdl3`, `cargo run -p game-cli --features ci-build-sdl3 -- doctor --explain`, and a temporary local-path demo generated via `cargo run -p xtask --features ci-build-sdl3 -- new-demo ... --template simple` followed by `game-dev check --features ci-build-sdl3`.

Keep `game-dev doctor` advisory. It should print guidance and return success unless the command itself crashed.

Make `game-dev check` explicitly fail for hard project prerequisites:

Hard failures:

- current directory cannot be read,
- `assets/` missing,
- assets invalid,
- `assets/game.ron` invalid if present,
- `cargo check` fails,
- generated project metadata is missing if package/check needs it.

Soft warnings:

- Vulkan unavailable if only running `cargo check`,
- audio prerequisites missing if only running `cargo check`,
- optional feature tools missing.

If adding a runtime smoke option, make it explicit:

```bash
game-dev check --smoke
```

Then `--smoke` may fail for Vulkan/SDL/window problems.

### 8.5 Use one source for release dependency strings

**Status:** Done on 2026-07-01. Added root `release.toml` with `current_tag` and `game_starter_dependency`, updated architecture tests to compare the CLI release dependency, template defaults, README/docs/checklist tag mentions, and documented game-starter dependency examples against that source, and added a release-checklist gate for updating `release.toml`. Verified with `cargo test -p game-core --test architecture_boundaries --locked`, `cargo test -p game-cli --locked --features ci-build-sdl3`, and `cargo run -p game-cli --features ci-build-sdl3 -- asset-check`.

Current `v0.2.0` strings appear in:

- `crates/game-cli/src/lib.rs`
- `templates/*/cargo-generate.toml`
- architecture tests
- README/docs/checklists

Create a single release metadata file, for example:

```text
release.toml
```

Possible content:

```toml
current_tag = "v0.2.0"
game_starter_dependency = '{ git = "https://github.com/P2949/game", tag = "v0.2.0", package = "game-starter" }'
```

Then choose one of these approaches:

- parse this file in tests and CLI, or
- generate the CLI constant/templates from it through `xtask`, or
- keep constants but add tests that compare all copies to `release.toml`.

Do the simplest robust version first: tests compare all hardcoded copies against `release.toml`.

## Done when

- `game-cli` is split into modules.
- Unknown asset files fail with helpful messages.
- `game-dev check` has documented hard/soft semantics.
- Release dependency/version strings are checked against one source.
- Generated templates still pass CI-style checks.

---

# Phase 9 — Split architecture boundary tests into focused files

## Goal

Keep the valuable architecture gates, but make them maintainable. `crates/game-core/tests/architecture_boundaries.rs` is currently too large and mixes dependency, docs, template, CLI, content, release, and API surface tests.

## Current code basis

Current file:

```text
crates/game-core/tests/architecture_boundaries.rs
```

It includes many useful tests:

- docs mention beginner surface,
- `game-core` and `game-kit` have no backend dependencies,
- beginner templates hide runtime boot code,
- beginner demos avoid raw ECS surface,
- generated-template CI checks exist,
- doctor diagnostics exist,
- beginner prelude does not export advanced ECS surface,
- beginner context is a wrapper,
- docs/examples/templates avoid compatibility prelude,
- content crates depend only on `game-kit` plus common deps,
- runtime/backends do not depend on content crates,
- many release/productization assertions.

The tests are good, but the single file is brittle.

## Target layout

Cargo integration tests can be multiple files under the same tests directory:

```text
crates/game-core/tests/architecture/
  support.rs              # if using module include pattern, or use a top-level support module per file
crates/game-core/tests/architecture_dependencies.rs
crates/game-core/tests/architecture_beginner_surface.rs
crates/game-core/tests/architecture_content_crates.rs
crates/game-core/tests/architecture_docs.rs
crates/game-core/tests/architecture_templates.rs
crates/game-core/tests/architecture_cli_release.rs
crates/game-core/tests/architecture_api_surface.rs
```

Because Rust integration tests do not share helper modules automatically across separate files unless arranged carefully, use one of these approaches:

### Option A — `tests/support/mod.rs`

Create:

```text
crates/game-core/tests/support/mod.rs
```

Each test file has:

```rust
mod support;
use support::*;
```

### Option B — keep one helper include file

Create:

```text
crates/game-core/tests/architecture_support.rs
```

But note it may compile as its own test unless structured carefully. Prefer Option A.

## Steps

### 9.1 Extract helper functions first

**Status:** Done on 2026-07-01. Moved shared architecture-test helpers and constants into `crates/game-core/tests/support/mod.rs`, imported them from `architecture_boundaries.rs`, and kept all existing assertions intact for the first mechanical step. Verified with `cargo test -p game-core --test architecture_boundaries --locked`.

Move helpers such as:

- `workspace_root`
- `collect_rust_files`
- `collect_markdown_files`
- `read_code_without_comments`
- `read_manifest_without_comments`
- `strip_cfg_test_modules`
- `contains_identifier`
- `extract_pub_module_body`
- `forbidden_source_uses`
- constants such as `BEGINNER_DEMO_FORBIDDEN`, `BEGINNER_DOC_FORBIDDEN`, etc.

into `crates/game-core/tests/support/mod.rs`.

Run tests.

### 9.2 Split dependency tests

**Status:** Done on 2026-07-01. Created `crates/game-core/tests/architecture_dependencies.rs` with the manifest/source dependency boundary tests for `game-core`, `game-kit`, `game-starter`, content crates, and runtime/backends. Shared support remains in `tests/support/mod.rs`. Verified with `cargo test -p game-core --test architecture_dependencies --locked` and `cargo test -p game-core --test architecture_boundaries --locked`.

Create `architecture_dependencies.rs` with:

- `game_core_manifest_has_no_backend_dependencies`
- `game_core_source_has_no_backend_imports`
- `game_kit_has_no_backend_dependencies`
- `game_starter_is_the_only_beginner_crate_that_depends_on_runtime`
- `content_crates_depend_only_on_game_kit_and_common_deps`
- `runtime_and_backends_do_not_depend_on_content_crates`

### 9.3 Split beginner surface tests

**Status:** Done on 2026-07-01. Created `crates/game-core/tests/architecture_beginner_surface.rs` for beginner prelude/context guards, high-level demo/template checks, beginner content-crate surface checks, raw map-command protection, data-driven beginner examples, and raw-world escape-hatch checks. Verified with `cargo test -p game-core --test architecture_beginner_surface --locked` and `cargo test -p game-core --test architecture_boundaries --locked`.

Create `architecture_beginner_surface.rs` with:

- beginner prelude does not export advanced ECS surface,
- beginner context wrapper test,
- beginner demos/templates hide runtime boot code,
- every beginner demo/template stays high-level,
- beginner content uses only beginner surface,
- production content does not use raw world escape hatches.

### 9.4 Split advanced surface tests

**Status:** Done on 2026-07-01. Created `crates/game-core/tests/architecture_advanced_surface.rs` for testbed advanced documentation/source guards and the advanced transition guide boundary check. Verified with `cargo test -p game-core --test architecture_advanced_surface --locked` and `cargo test -p game-core --test architecture_boundaries --locked`.

Create `architecture_advanced_surface.rs` with:

- testbed content documented as advanced,
- testbed content remains advanced lab,
- advanced transition guide names boundary.

### 9.5 Split docs/template/release tests

**Status:** Done on 2026-07-01. Created `architecture_docs.rs`, `architecture_templates.rs`, and `architecture_cli_release.rs` for documentation, generated-template, and CLI/release/productization guards. Verified with `cargo test -p game-core --test architecture_docs --locked`, `cargo test -p game-core --test architecture_templates --locked`, `cargo test -p game-core --test architecture_cli_release --locked`, and `cargo test -p game-core --test architecture_boundaries --locked`.

Create:

- `architecture_docs.rs`
- `architecture_templates.rs`
- `architecture_cli_release.rs`

Move relevant tests into each.

### 9.6 Add root API cleanup tests from Phases 2 and 3

**Status:** Done on 2026-07-01. Created `architecture_api_surface.rs` with the root API cleanup, compatibility prelude, test harness, and data-module split guards, and created `architecture_content_crates.rs` for the remaining content-crate source/import boundary checks. The former `architecture_boundaries.rs` is now only a one-line note. Verified all split architecture integration tests with `cargo test -p game-core --locked --test architecture_dependencies --test architecture_beginner_surface --test architecture_advanced_surface --test architecture_docs --test architecture_templates --test architecture_cli_release --test architecture_api_surface --test architecture_content_crates --test architecture_boundaries`.

Create `architecture_api_surface.rs` with:

- `game_kit_root_does_not_reexport_authoring_surface`
- `game_core_root_does_not_reexport_internal_surface`
- `game_kit_compatibility_prelude_is_visibly_deprecated`
- `game_test_harness_is_not_root_reexported`

## Done when

- No single architecture test file is enormous.
- Helpers are shared cleanly.
- Existing architecture coverage remains equivalent or stronger.
- New API/root/map-command policies are enforced.

---

# Phase 10 — Split beginner prefab/rules/app modules where it helps future work

## Goal

After the critical boundary work is complete, split the next largest `game-kit` modules enough that future beginner API changes are localized.

Do not start this phase before Phases 2, 4, and 6. The public surface and data loader split are more important.

## Current code basis

Large files:

- `crates/game-kit/src/beginner/prefabs.rs` — about 2.2k lines.
- `crates/game-kit/src/app.rs` — about 1.6k lines.
- `crates/game-kit/src/beginner/rules.rs` — about 1.6k lines.
- `crates/game-kit/src/beginner/events.rs` — about 1k lines.
- `crates/game-kit/src/beginner/collections.rs` — about 870 lines.

## Suggested splits

### 10.1 Split beginner prefabs

**Status:** Done on 2026-07-01. Replaced the `beginner/prefabs.rs` monolith with `beginner/prefabs/` modules for player, enemy, pickup, projectile, door, area, spawner, and shared prefab state; `prefabs/mod.rs` reexports the public author types so existing paths stay stable. Verified with `cargo test -p game-kit --locked`.

Target layout:

```text
crates/game-kit/src/beginner/prefabs/
  mod.rs
  player.rs
  enemy.rs
  pickup.rs
  projectile.rs
  door.rs
  spawner.rs
  area.rs
  shared.rs
```

Move one prefab builder at a time.

Keep public paths stable through `pub use` in `prefabs/mod.rs`:

```rust
pub use player::PlayerPrefabAuthor;
pub use enemy::EnemyPrefabAuthor;
...
```

Run `cargo test -p game-kit --locked` after each moved builder.

### 10.2 Split beginner rules

**Status:** Done on 2026-07-01. Replaced the `beginner/rules.rs` monolith with `beginner/rules/` modules for the author, animation, combat, projectiles, pickups, doors, spawners, checkpoints, UI, win conditions, and shared imports; `rules/mod.rs` reexports `RulesAuthor` and behavior types to keep public paths stable. Verified with `cargo test -p game-kit --locked` and `cargo test -p game-core --test architecture_api_surface --locked`.

Target layout:

```text
crates/game-kit/src/beginner/rules/
  mod.rs
  author.rs
  movement.rs
  combat.rs
  projectiles.rs
  pickups.rs
  doors.rs
  spawners.rs
  checkpoints.rs
  animation.rs
  ui.rs
  win_conditions.rs
```

Keep `RulesAuthor` public path stable.

Move independent behaviors first. Avoid changing scheduling order accidentally.

For each moved behavior, preserve:

- type name,
- public builder method,
- system registration stage,
- order relative to existing systems if behavior depends on order.

### 10.3 Split `app.rs` only after prefabs/rules are stable

**Status:** Done on 2026-07-01. Replaced the `app.rs` monolith with `app/` modules: `mod.rs` keeps the central `GameApp` API, `plugin.rs` owns `Plugin`/`FnGamePlugin`/`plugin`/`plugin_fn`, `debug.rs` owns `DebugOverlayAuthor`, `validation.rs` owns prefab matching helpers, and `tests.rs` owns the app tests. Public `game_kit::app::*` paths remain stable. Verified with `cargo test -p game-kit --locked`, `cargo test -p game-core --test architecture_api_surface --locked`, and `cargo fmt --all -- --check`.

`GameApp` is central. Split cautiously.

Possible target:

```text
crates/game-kit/src/app/
  mod.rs
  plugin.rs
  authoring.rs
  systems.rs
  events.rs
  finish.rs
  debug.rs
```

But do this only if it improves clarity. A 1.6k-line central builder is less urgent than `data.rs` and `mixer.rs`.

## Done when

- The biggest beginner modules are split without public API breakage.
- Existing beginner examples compile unchanged.
- Behavior order is preserved.
- Public reexports in `beginner::prelude` remain stable.

---

# Phase 11 — Query/system unsafe-boundary documentation and tests

## Goal

Keep the custom ECS/query model simple, but make the unsafe extraction boundary and borrow-conflict rules very explicit. This supports maintainability without prematurely rewriting to an archetype ECS.

## Current code basis

`game-core/src/query.rs` uses raw pointer extraction inside `SystemParam::extract`, guarded by `ParamAccess` validation.

Existing tests:

- `crates/game-core/tests/system_param_disjoint_borrows.rs`
- `crates/game-kit/tests/system_param_disjoint_borrows.rs`

They already test read/write conflicts and filter conflicts.

## Steps

### 11.1 Expand safety docs

**Status:** Done on 2026-07-01. Added a module-level `# Safety model` section to `game-core/src/query.rs` covering raw pointer extraction, sealed params, registration-time access validation, alias rejection, frame-bound extracted values, and the beginner-surface separation.

In `game-core/src/query.rs`, add a module-level `# Safety model` section explaining:

- why raw pointers are used,
- that supported params are sealed,
- that parameter access is registered before scheduling,
- that duplicate mutable or read+mutable access to the same component/resource is rejected,
- that extracted values must not escape a frame,
- that beginner content does not use this surface.

### 11.2 Add tests for resource conflicts

**Status:** Done on 2026-07-01. Added `ParamAccess`/`SystemParam` tests for `Res<T>` + `ResMut<T>` rejection, duplicate `ResMut<T>` rejection, duplicate shared `Res<T>` allowance at the validator layer, and resource conflict messages containing the type name.

Current tests cover resource extraction mutation but not all conflicts. Add tests for:

- `Res<T>` + `ResMut<T>` in one system is rejected.
- `ResMut<T>` + `ResMut<T>` in one system is rejected.
- two `Res<T>` are allowed if the API supports duplicate shared access; if not supported by current macro impls, document that limitation.

### 11.3 Add tests for mixed query/resource systems if supported

**Status:** Done on 2026-07-01. Confirmed `Query<&mut T>` + `ResMut<R>` is supported and added a runtime adapter test that mutates both an entity component and a resource through one parameter system.

Check current macro support before adding tests. If supported, test:

- `Query<&mut Transform>` plus `ResMut<MyResource>` works.
- conflicting resource access errors mention resource type name.

### 11.4 Add docs to advanced authoring guide

**Status:** Done on 2026-07-01. Expanded `docs/advanced-content-authoring.md` with resource-borrow conflict behavior, type-name diagnostics, the current one-resource typed-function adapter limitation, and guidance for beginner content to stay on rules/hooks/builders.

In `docs/advanced-content-authoring.md`, add a short section:

- advanced systems are still validated for borrow conflicts,
- if a system tries to read and mutate the same component/resource, registration fails before runtime,
- beginner code should use hooks/rules/builders instead.

## Done when

- Unsafe query extraction has clear safety docs.
- Resource conflict tests exist.
- Advanced docs explain conflict errors.
- No ECS rewrite was attempted.

Verified with `cargo test -p game-core --test system_param_disjoint_borrows --locked`, `cargo test -p game-kit --test system_param_disjoint_borrows --locked`, `cargo test -p game-core --locked`, and `cargo fmt --all -- --check`. Also updated the beginner architecture guards to read the split `app/` and `beginner/rules/` module trees.

---

# Phase 12 — CI permissions and release gate hardening

## Goal

Make CI least-privilege and ensure the new consolidation checks are part of the release gate.

## Current code basis

- `.github/workflows/release.yml` already has `permissions: contents: write`.
- `.github/workflows/ci.yml` currently has no top-level explicit permissions.
- CI already runs workspace tests, clippy, release build, headless runtime, audit/deny, Vulkan smoke, generated templates, package checks, doctor, and first-15-minutes checks.

## Steps

### 12.1 Add least-privilege permissions to CI

**Status:** Done on 2026-07-01. Added top-level `permissions: contents: read` to `.github/workflows/ci.yml` while leaving the release workflow at `contents: write`.

At top-level of `.github/workflows/ci.yml`, add:

```yaml
permissions:
  contents: read
```

Do not change `release.yml` from `contents: write`, because it uploads release artifacts.

### 12.2 Ensure new checks are covered

**Status:** Done on 2026-07-01. Verified the normal CI `cargo test --workspace --locked --features game/ci-build-sdl3` gate covers the new architecture guards, CLI unknown-asset tests, data split tests, runtime command diagnostics, and map transition boundary tests. Added an architecture guard so CI permissions and checklist gates remain explicit.

Add or verify CI covers:

- architecture API root cleanup tests,
- CLI unknown asset extension tests,
- data module split tests,
- runtime command error policy tests,
- map transition no-raw-bypass test.

Most will be covered by `cargo test --workspace`. If any requires feature-gated code, make sure CI feature flags include it.

### 12.3 Update release checklist

**Status:** Done on 2026-07-01. Added the boundary hardening checklist items for `docs/api-boundary.md`, `game-kit`/`game-core` root export guards, unknown asset extensions, command error policy tests, map transition boundary tests, and audio/docs consistency.

In `docs/release-checklist.md`, add checklist items:

- [ ] `docs/api-boundary.md` reviewed.
- [ ] `game-kit` root export guard passed.
- [ ] `game-core` root export guard passed.
- [ ] unknown asset extension check verified.
- [ ] command error policy tests passed.
- [ ] map transition boundary tests passed.
- [ ] audio/docs consistency checked.

## Done when

- CI workflow has explicit permissions.
- Release checklist names the new boundary hardening gates.
- All new checks run in normal CI.

Verified with `cargo test -p game-core --test architecture_cli_release --locked`, `cargo test -p game-core --locked`, `cargo test -p game-cli --locked --features ci-build-sdl3`, and `cargo fmt --all -- --check`.

---

# Phase 13 — Documentation polish and migration notes

## Goal

Make the docs match the consolidated architecture and prevent future contributors from reversing the boundary cleanup.

## Steps

### 13.1 Update current/historical roadmap statuses

**Status:** Done on 2026-07-01. Kept the historical roadmaps intact, updated the roadmaps index to mark beginner productization complete for `v0.2.0`, and marked `post-1.0-api-surface-cleanup.md` as superseded by this consolidation roadmap.

Do not rewrite history. Instead:

- Keep `docs/architectural-improvement-roadmap.md` historical.
- Keep `docs/content-authoring-api-roadmap.md` historical.
- Keep `docs/beginner-authoring-roadmap.md` historical.
- Keep `docs/beginner-productization-roadmap.md` as completed `v0.2.0` productization status.
- Update `docs/roadmaps/post-1.0-api-surface-cleanup.md` if this work supersedes it.

If this roadmap implements the API cleanup now, change `post-1.0-api-surface-cleanup.md` to:

```markdown
> Status: Superseded by content-engine-boundary-consolidation.md.
```

or mark completed items.

### 13.2 Add migration guide for root API cleanup

**Status:** Done on 2026-07-01. `docs/migrations/root-api-cleanup.md` exists with old `game_kit::prelude::*` and root plugin examples, new beginner/standalone/advanced imports, safe advanced plugin-helper aliasing, explicit `game_kit::app` paths, and temporary `game_kit::compat::*` guidance.

Create:

```text
docs/migrations/root-api-cleanup.md
```

Required examples:

Old:

```rust
use game_kit::prelude::*;
```

New beginner:

```rust
use game_kit::beginner::prelude::*;
```

New standalone:

```rust
use game_starter::prelude::*;
```

New advanced:

```rust
use game_kit::advanced::prelude::*;
```

Old root plugin helper:

```rust
pub fn plugin() -> game_kit::Plugin<MyPlugin> {
    game_kit::plugin(MyPlugin)
}
```

New:

```rust
use game_kit::advanced::prelude::*;

pub fn plugin() -> Plugin<MyPlugin> {
    plugin(MyPlugin)
}
```

or:

```rust
pub fn plugin() -> game_kit::app::Plugin<MyPlugin> {
    game_kit::app::plugin(MyPlugin)
}
```

### 13.3 Update CHANGELOG

**Status:** Done on 2026-07-01. Added an Unreleased architecture/migration entry naming the root API narrowing, content-aware map transitions, runtime command diagnostics, large module splits, and the new import surfaces.

Add an unreleased section:

```markdown
## Unreleased

### Architecture
- Narrowed `game-kit` and `game-core` root APIs toward explicit beginner/advanced/internal surfaces.
- Made map transitions content-aware by preventing raw active-map commands from normal content paths.
- Added structured runtime command diagnostics.
- Split large data/audio/CLI/architecture-test modules.

### Migration
- Use `game_kit::beginner::prelude::*`, `game_kit::advanced::prelude::*`, or `game_starter::prelude::*` instead of `game_kit::prelude::*` or root `game_kit::*` imports.
```

### 13.4 Update tutorials/common errors

**Status:** Done on 2026-07-01. Expanded `docs/tutorials/common-errors.md` with notes for unknown asset extensions, prefab typo suggestions, map name typos, audio voice-cap drops, and when to stay on beginner APIs versus the advanced path.

Add short user-facing notes to `docs/tutorials/common-errors.md`:

- unknown asset file extension,
- map name typo,
- prefab name typo,
- too many sounds per frame/voice cap,
- when to use beginner versus advanced API.

## Done when

- Docs no longer conflict with current code.
- Migration guide exists for API cleanup.
- Changelog names the architectural consolidation work.
- Common beginner mistakes have friendly docs.

Verified with `cargo test -p game-core --test architecture_docs --locked`, `cargo test -p game-core --test architecture_cli_release --locked`, `cargo test -p game-core --test architecture_api_surface --locked`, `cargo test -p game-core --locked`, and `cargo fmt --all -- --check`.

---

# Phase 14 — Final validation pass

## Goal

Prove the project still satisfies the original objective after consolidation.

## Steps

### 14.1 Run the full gate

**Status:** Done on 2026-07-01. Passed `cargo fmt --all -- --check`, `cargo test --workspace --locked --features game/ci-build-sdl3`, `cargo test -p game-runtime --test headless_runner --no-default-features --locked`, `cargo clippy --workspace --all-targets --locked --features game/ci-build-sdl3 -- -D warnings`, `cargo build -p game --release --locked --features ci-build-sdl3`, `cargo run -p game-cli --features ci-build-sdl3 -- doctor --explain`, `cargo run -p game-cli --features ci-build-sdl3 -- asset-check`, and `cargo run -p game-cli --features ci-build-sdl3 -- validate-data assets/game.ron`. Attempted the graphical `scripts/first-15-minutes.sh` path with `FIRST15_USE_XVFB=1`, but this environment does not have `xvfb-run`; reran with `FIRST15_SKIP_SMOKE=1` and verified generated-project creation, `cargo check`, asset validation, release packaging, package layout, and zip creation.

Run:

```bash
cargo fmt --all -- --check
cargo test --workspace --locked --features game/ci-build-sdl3
cargo test -p game-runtime --test headless_runner --no-default-features --locked
cargo clippy --workspace --all-targets --locked --features game/ci-build-sdl3 -- -D warnings
cargo build -p game --release --locked --features ci-build-sdl3
cargo run -p game-cli --features ci-build-sdl3 -- doctor --explain
cargo run -p game-cli --features ci-build-sdl3 -- asset-check
cargo run -p game-cli --features ci-build-sdl3 -- validate-data assets/game.ron
scripts/first-15-minutes.sh
```

Use Xvfb/lavapipe if needed.

### 14.2 Run targeted source audits

**Status:** Done on 2026-07-01. Ran the requested `rg` audits. Direct engine imports from `simple-content`/`arena-content`, raw ECS vocabulary in beginner examples/templates/content, and broad root `pub use` patterns all returned no matches. `game_kit::prelude::*` matches are limited to compatibility-policy docs, migration docs, roadmap text, and architecture-test fixtures; no beginner-facing usage was found.

Run:

```bash
rg "game_kit::prelude::\*" README.md docs crates examples templates
rg "game_core::|game_map::|game_ai::|game_combat::|game_physics::|game_runtime::|game_renderer_vulkan|game_platform_sdl|game_audio" crates/simple-content/src crates/arena-content/src
rg "GameCtx|StartupGameCtx|EntityId|Component|Transform|Velocity|Sprite::new|Collider::box_of|Health::new|MeleeAttack|Faction|AiController|ChaseTarget|PathFollow|Patrol|PrefabAuthor|game\.commands\(" examples templates crates/simple-content/src crates/arena-content/src
rg "pub use beginner::|pub use context::\{Commands|pub use data::\{Beginner|pub use world::|pub use builder::" crates/game-kit/src/lib.rs crates/game-core/src/lib.rs
```

Expected:

- no compatibility prelude usage in beginner-facing docs/examples/templates,
- no direct engine crate imports from beginner content,
- no raw ECS vocabulary in beginner examples/templates,
- no broad root `pub use` in `game-kit`/`game-core` except explicit compatibility module if still kept.

### 14.3 Do a human-read acceptance test

**Status:** Done on 2026-07-01. Read the listed examples/templates/content crates. They still present game-shaped authoring: assets, controls, prefabs, maps/scenes, rules/events, sound/music/UI, and starter/beginner imports without Vulkan/SDL/audio-device/runtime loop, raw ECS/world traversal, or manual backend/resource work.

Open these files and read them like a beginner:

- `examples/one-file-demo/src/main.rs`
- `examples/no-rust-shapes-demo/src/main.rs`
- `examples/script-like-custom-rules/src/main.rs`
- `templates/simple-demo/src/main.rs`
- `templates/data-driven-demo/assets/game.ron`
- `crates/simple-content/src/lib.rs`
- `crates/arena-content/src/lib.rs`

They should still look like game code, not engine code.

The reader should see:

- assets,
- controls,
- players/enemies/pickups/projectiles,
- maps/scenes,
- rules/events,
- sound/music/UI,
- no Vulkan/SDL/audio-device/runtime loop,
- no raw ECS/world traversal,
- no manual memory/resource backend work.

### 14.4 Update roadmap statuses

**Status:** Done on 2026-07-01. Every implementation item in this roadmap has an explicit completion status. The only caveat is environmental: graphical first-15-minutes smoke needs `xvfb-run` (or another display path) on this machine; the non-graphical generated-project acceptance path passed.

Mark each completed item in this roadmap.

If any item is deliberately deferred, add:

- why it was deferred,
- what concrete signal should trigger it later,
- where it is tracked.

Do not leave vague TODOs.

## Done when

- All checks pass or failures are documented as pre-existing/environmental.
- The content examples still satisfy the game-shaped API goal.
- Docs, tests, and code agree on the architecture boundary.

---

# Suggested commit sequence

Use small commits so regressions are easy to isolate:

1. `docs: add content-engine boundary consolidation roadmap`
2. `docs: define public api boundary contract`
3. `docs: refresh architecture notes for current audio and preludes`
4. `refactor(game-kit): route plugin helpers through app module paths`
5. `refactor(game-kit): remove internal reliance on root reexports`
6. `refactor(game-kit): narrow root authoring exports`
7. `refactor(game-core): narrow root internal exports`
8. `fix(game-kit): keep map transitions content-runtime aware`
9. `feat(runtime): record structured command errors`
10. `test(runtime): add strict command error policy coverage`
11. `refactor(game-kit): split beginner data schema and loading modules`
12. `refactor(game-kit): split beginner data validation and effects modules`
13. `refactor(audio): split mixer sound voice and decode modules`
14. `refactor(audio): split audio system callback and stream modules`
15. `refactor(cli): split game-dev command modules`
16. `fix(cli): reject unknown asset extensions with friendly diagnostics`
17. `test(architecture): split boundary tests by concern`
18. `test(architecture): guard game-kit and game-core root APIs`
19. `docs: add root api cleanup migration guide`
20. `ci: set least privilege workflow permissions`
21. `docs: update release checklist for boundary consolidation`
22. `chore: run final architecture consolidation gate`

---

# Do-not-do list

Do not do these during this roadmap:

- Do not rewrite the ECS into an archetype ECS.
- Do not rewrite the Vulkan renderer.
- Do not introduce scripting, Lua/Rhai, or an editor.
- Do not add major gameplay features.
- Do not make `game-kit` depend on `game-runtime`, `game-renderer-vulkan`, `game-platform-sdl`, or `game-audio`.
- Do not make content crates depend on `game-core`, `game-map`, `game-ai`, `game-combat`, `game-physics`, runtime, renderer, platform, or audio crates.
- Do not weaken architecture tests just because they catch a real boundary leak.
- Do not silently ignore asset files in beginner tooling.
- Do not expose raw map `MapId` switching as a normal content action.
- Do not hide runtime command failures only in logs.

---

# Success statement

After this roadmap, the project should feel less like a growing pile of convenient reexports and more like a small, intentional game framework:

- beginners write `game_starter::prelude::*` or `game_kit::beginner::prelude::*`,
- advanced content opts into `game_kit::advanced::prelude::*`,
- low-level engine/runtime/backend work remains behind the facade,
- errors from content mistakes are reported in beginner-friendly terms,
- large implementation files are split enough for safe future iteration,
- tests enforce the architecture instead of relying on discipline.

That is the foundation needed before building more actual game content.

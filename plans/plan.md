I reviewed the attached snapshot directly. The current code is already far beyond the earlier architectural split work. It has `game-starter`, `game-kit` beginner/advanced preludes, beginner-only callback wrappers, `game-dev`, generated-template CI, release packaging, no-Rust `game.ron`, partial data reload, script-like hooks/rules, diagnostics, docs, and architecture tests.

So this roadmap should **not** ask for another engine split. The correct next roadmap is a **release-readiness / final polish roadmap** for making the beginner framework feel complete, hard to misuse, and genuinely productized.

# Architectural Improvement Roadmap: Beginner Framework Release Polish

## Current status

The original objective is achieved:

```text
Engine/content separation:                 achieved
Content hidden from low-level code:         achieved
Beginner API exists:                        achieved
No-Rust data path exists:                   achieved
Script-like hooks/rules exist:              achieved
Generated project flow exists:              achieved
Standalone CLI exists:                      achieved
Packaging flow exists:                      achieved
Architecture regression tests exist:        achieved
```

The remaining work is:

```text
1. Verify the current snapshot with the real Rust toolchain.
2. Update roadmap/status docs so they match the implemented code.
3. Decide whether partial game.ron reload is good enough or implement full structural reload.
4. Make misuse harder by reducing broad root-level game-kit exports.
5. Make distribution more polished: crates.io, template repo, release docs.
6. Keep the beginner learning path narrow and obvious.
7. Treat diagnostics, docs, packaging, and generated-project CI as first-class release gates.
```

The key principle:

```text
Do not do more architecture splitting.

The engine/content boundary is done. The beginner API is done enough.
The remaining work is release polish, product maturity, and making the
script-like layer feel complete.
```

---

# Phase 0 — Confirm the current snapshot really builds

## Why this phase matters

The current code looks architecturally correct, but static review is not enough. Before marking this milestone as complete, run the full toolchain locally.

## Relevant files

```text
Cargo.toml
Cargo.lock
.github/workflows/ci.yml
.github/workflows/release.yml
crates/game-kit/*
crates/game-starter/*
crates/game-cli/*
crates/game-core/tests/architecture_boundaries.rs
templates/*
examples/*
docs/*
```

## Steps

### 0.1 Run formatting

```bash
cargo fmt --all -- --check
```

If this fails, run:

```bash
cargo fmt --all
```

Then re-run the check.

### 0.2 Run workspace tests

```bash
cargo test --workspace --locked
```

This must pass before any “release candidate” claim.

### 0.3 Run clippy

```bash
cargo clippy --workspace --all-targets --locked -- -D warnings
```

Fix warnings instead of suppressing them unless the lint is genuinely wrong.

### 0.4 Run release build

```bash
cargo build -p game --release --locked
```

### 0.5 Run smoke tests

```bash
GAME_SMOKE_FRAMES=120 cargo run -p game --locked
GAME_DEMO=simple GAME_SMOKE_FRAMES=120 cargo run -p game --locked
GAME_DEMO=testbed GAME_SMOKE_FRAMES=120 cargo run -p game --locked
GAME_ASSET_DIR=assets GAME_SMOKE_FRAMES=120 cargo run -p game --release --locked
```

### 0.6 Run source-built SDL3 path

```bash
cargo test --workspace --locked --features game/ci-build-sdl3
cargo clippy --workspace --all-targets --locked --features game/ci-build-sdl3 -- -D warnings
cargo build -p game --release --locked --features ci-build-sdl3
```

### 0.7 Run CLI checks

```bash
cargo run -p game-cli -- doctor
cargo run -p game-cli -- doctor --explain
cargo run -p game-cli -- validate-data assets/game.ron
cargo run -p game-cli -- asset-check
```

### 0.8 Run generated-project checks manually

```bash
rm -rf /tmp/generated-game-smoke
mkdir -p /tmp/generated-game-smoke

cargo run -p game-cli -- new /tmp/generated-game-smoke/simple --template simple
cargo check --manifest-path /tmp/generated-game-smoke/simple/Cargo.toml
GAME_SMOKE_FRAMES=60 cargo run --manifest-path /tmp/generated-game-smoke/simple/Cargo.toml

cargo run -p game-cli -- new /tmp/generated-game-smoke/data --template data-driven
cargo check --manifest-path /tmp/generated-game-smoke/data/Cargo.toml
GAME_SMOKE_FRAMES=60 cargo run --manifest-path /tmp/generated-game-smoke/data/Cargo.toml
```

### 0.9 Run package checks

```bash
cd /tmp/generated-game-smoke/simple
game-dev package --release --out dist/simple-demo --zip

cd /tmp/generated-game-smoke/data
game-dev package --release --out dist/data-demo --zip
```

Or, if `game-dev` is not installed globally yet:

```bash
cargo run -p game-cli -- package --release --out /tmp/generated-game-smoke/simple/dist/simple-demo --zip
```

from the generated project root.

## Definition of done

```text
- fmt passes.
- tests pass.
- clippy passes.
- release build passes.
- default/simple/testbed smoke tests pass.
- generated simple project builds and smoke-runs outside workspace.
- generated data-driven project builds and smoke-runs outside workspace.
- package command creates valid output and zip.
- any failures are fixed or documented as external environment issues.
```

---

# Phase 1 — Update project-status documents to match reality

## Problem

The code has implemented most of the productization roadmap, but `docs/beginner-productization-roadmap.md` still says:

```text
Productization status: in progress.
```

The phase checklist is also not marked item-by-item as complete, partial, or remaining.

This is a documentation truth problem. The docs should not make a completed implementation look unfinished.

## Relevant files

```text
docs/beginner-productization-roadmap.md
README.md
docs/ARCHITECTURE.md
docs/release-checklist.md
docs/dead-code-audit.md
CHANGELOG.md
```

## Steps

### 1.1 Rewrite the status block

Change the current state section to something like:

```markdown
## Current State

- Architecture status: complete.
- Beginner Productization 1.0 status: release-candidate.
- Remaining release polish:
  - run full local verification,
  - decide whether partial `game.ron` reload is sufficient for 1.0,
  - optionally narrow root-level `game-kit` exports,
  - publish/version templates and crates,
  - keep tutorials focused on one beginner path.
```

### 1.2 Convert the phase checklist into a status table

Replace the plain checklist with:

```markdown
| Phase | Status | Notes |
| --- | --- | --- |
| Phase 0: Freeze baseline | Done / verify locally | Full cargo checks still need local run. |
| Phase 1: Beginner-only callback wrappers | Done | `beginner::Game` and `StartupGame` exist. |
| Phase 2: Public import surfaces | Done | Beginner/advanced preludes exist; compatibility prelude deprecated. |
| Phase 3: Generated-project CI | Done | CI checks generated templates outside workspace. |
| Phase 4: Standalone `game-dev` CLI | Done | `game-cli` crate exists. |
| Phase 5: Doctor diagnostics | Done / polish | `doctor` and `--explain` exist; keep improving messages. |
| Phase 6: Package generated projects | Done | `game-dev package --zip` exists. |
| Phase 7: Data reload | Partial by design | Existing-value reload works; structural changes require restart. |
| Phase 8: Script-like events/rules | Done / expandable | Common hooks exist. |
| Phase 9: Teaching diagnostics | Mostly done | Keep expanding known-name suggestions. |
| Phase 10: Starter assets | Done / polish | Starter assets exist; improve art pack over time. |
| Phase 11: Tutorial path | Done / polish | Numbering is mostly cleaned; keep path narrow. |
| Phase 12: Data DSL parity | Mostly done | Conditions/effects exist; expand as users need. |
| Phase 13: Packaging docs | Done / verify | Package flow exists; test on real OSes. |
| Phase 14: Prebuilt artifacts | Done in release workflow | Verify on tag. |
| Phase 15: Stability/migrations | Done initial | CHANGELOG and migrations exist. |
| Phase 16: Advanced separation | Done | `testbed-content` remains advanced. |
| Phase 17: First-15-minutes test | Done in CI | `scripts/first-15-minutes.sh` exists. |
| Phase 18: Final gate | Remaining | Run before marking 1.0 complete. |
```

### 1.3 Update README status

Add a short “Project status” section:

```markdown
## Project status

The engine/content split and beginner authoring foundation are implemented.
The project is in beginner-productization/release-candidate stage.

Beginners should start with `game_starter::prelude::*`,
`templates/simple-demo`, or `templates/data-driven-demo`.

Advanced users can use `game_kit::advanced::prelude::*`.
```

### 1.4 Update release checklist

Add explicit final release gates:

```text
- generated-template CI green
- first-15-minutes CI green
- game-dev doctor checked
- game-dev package checked
- game.ron validate-data checked
- release artifacts generated
- tutorial sequence checked
- roadmap status updated
```

### 1.5 Update dead-code audit

Make sure it does not describe already-implemented features as future work:

```text
LDtk/Tiled import
runtime map switching
file-backed sounds
beginner CLI
data-file DSL
release packaging
```

## Definition of done

```text
- Roadmap doc accurately says what is done, partial, and remaining.
- README no longer undersells the implementation.
- Release checklist contains beginner-productization gates.
- Dead-code audit has no stale “future work” claims for implemented features.
```

---

# Phase 2 — Decide the 1.0 policy for `game.ron` reload

## Problem

The current data-driven reload system is intentionally partial.

Implemented:

```text
- BeginnerFileRuntime
- BeginnerReloadLevel
- BeginnerRuntimeConfig
- validate-data command
- debug overlay reload status
- identity checks for structural lists
- reload of existing values/map file paths/rule settings
```

Current limitation:

```text
Adding, removing, or reordering structural lists requires restart:
- assets
- prefabs
- maps
- rules
- actions
- scenes
```

This is honest and reasonable, but if the goal is “fully script-like,” this is the largest remaining gap.

## Decision point

Choose one of these policies.

### Option A — Keep partial reload for 1.0

This is the safer choice.

1. Treat current reload as good enough for Beginner Productization 1.0.
2. Document it clearly.
3. Add more tests around the partial contract.
4. Move full structural reload to a 1.1/2.0 roadmap.

### Option B — Implement full structural `game.ron` reload before 1.0

This is more ambitious.

1. Rebuild beginner content model at runtime.
2. Swap runtime prefab/map/rule registries.
3. Allow adding/removing/reordering prefabs/maps/rules.
4. Reset current map or current scene after successful reload.
5. Update assets dynamically or clearly delay newly added asset handles until restart.

## Recommendation

Use **Option A** for 1.0 unless you specifically want the project to be a live-scripting environment right now.

The current partial reload is enough for beginner demos, and full structural reload risks destabilizing the already-good architecture.

## If choosing Option A

### 2A.1 Update docs to call this “partial by design”

Files:

```text
docs/tutorials/12-fast-iteration.md
docs/tutorials/13-data-driven-demo.md
templates/data-driven-demo/README.md
docs/beginner-productization-roadmap.md
```

Make the language consistent:

```markdown
F5 reload is partial by design in 1.0.

It reloads:
- text map file contents,
- existing map file paths,
- existing tuning values,
- existing prefab numeric values,
- existing custom countdown rule details,
- existing scene text/menu/audio settings,
- existing action settings when bindings stay stable,
- registered textures and sounds.

It does not reload:
- adding/removing/reordering assets,
- adding/removing/reordering prefabs,
- adding/removing/reordering maps,
- adding/removing/reordering actions,
- adding/removing/reordering scenes,
- adding new asset keys.

For those changes, restart the game.
```

### 2A.2 Add a “why partial reload?” explanation

Add:

```markdown
The runtime can safely update values and respawn the active map, but structural
changes affect asset handles, prefab registries, action ids, scene names, and
runtime systems. Those remain restart-required for 1.0 so the beginner API stays
predictable.
```

### 2A.3 Add test coverage for every reload category

In `crates/game-kit/src/data.rs` tests, ensure coverage for:

```text
- map path changes reload
- prefab value changes reload
- custom countdown values reload
- scene text reload
- audio scene values reload
- action settings reload when action identity is stable
- adding a texture key is rejected with a teaching error
- adding a prefab is rejected with a teaching error
- adding a map is rejected with a teaching error
- adding a scene is rejected with a teaching error
- adding an action is rejected with a teaching error
```

### 2A.4 Add `game-dev validate-data --reload-compatible <old> <new>`

This is optional but useful.

Command:

```bash
game-dev validate-data assets/game.ron --reload-compatible previous-game.ron
```

Or:

```bash
game-dev validate-reload old.ron new.ron
```

It should tell the user whether a change is F5-reloadable or restart-required.

## If choosing Option B

Only do this if full live scripting is a hard requirement.

### 2B.1 Extract a runtime-swappable beginner content model

Add:

```rust
pub struct BeginnerContentModel {
    pub file: BeginnerGameFile,
    pub assets: BeginnerAssetsFile,
    pub controls: BeginnerControlsFile,
    pub prefabs: Vec<BeginnerPrefabFile>,
    pub maps: Vec<BeginnerMapFile>,
    pub scenes: Option<...>,
    pub rules: Vec<BeginnerRuleFile>,
    pub script_rules: Vec<BeginnerScriptRuleFile>,
}
```

### 2B.2 Separate “build-time registration” from “runtime model”

Currently `load_beginner_file` likely registers assets/prefabs/maps/systems during `GameApp` build.

Refactor into:

```text
parse -> validate -> build runtime model -> register systems that read model/resource
```

Systems should read a runtime resource instead of capturing fixed data.

### 2B.3 Change spawned prefab commands to use names

Where possible, commands should carry:

```rust
SpawnPrefabByName { name, position, properties }
```

instead of long-lived `PrefabId`s that become stale across reloads.

### 2B.4 Rebuild and swap registries on F5

On F5:

```text
1. Parse new game.ron.
2. Validate against current asset/backend constraints.
3. Build new prefab registry.
4. Build new map registry.
5. Swap beginner runtime model.
6. Reset current map.
7. Update debug overlay.
```

### 2B.5 Asset additions remain special

Even with structural reload, adding new texture/sound keys may require renderer/audio dynamic asset insertion. Either implement that separately or clearly say:

```text
New asset keys require restart in 1.x.
```

## Definition of done for Phase 2

```text
- The project has a clear 1.0 policy for game.ron reload.
- Docs and debug overlay match the policy.
- Tests cover the policy.
- Users can tell whether a game.ron change needs F5 or restart.
```

---

# Phase 3 — Narrow root-level `game-kit` exports or document them as advanced

## Problem

The beginner and advanced preludes are clean, and the compatibility prelude is deprecated. Good.

However, the crate root still broadly re-exports many types:

```rust
pub use context::{Commands, GameCtx, StartupGameCtx};
pub use app::{GameApp, GamePlugin, ...};
pub use beginner::...
pub use data::...
```

This is not a blocker, because beginner docs use:

```rust
use game_starter::prelude::*;
use game_kit::beginner::prelude::*;
```

But if the goal is “hard to misuse,” broad root exports make it easier for users to stumble into advanced types.

## Goal

Make root exports intentional.

## Relevant files

```text
crates/game-kit/src/lib.rs
crates/game-kit/src/beginner/prelude.rs
crates/game-kit/src/advanced/prelude.rs
crates/game-kit/src/advanced/mod.rs
docs/beginner-authoring.md
docs/advanced-content-authoring.md
crates/game-core/tests/architecture_boundaries.rs
```

## Options

### Option A — Keep root exports for compatibility

Mark them as compatibility/internal in docs. This is low-risk.

### Option B — Deprecate broad root advanced exports

Add deprecation attributes to advanced root exports and tell users to import from `advanced::prelude`.

### Option C — Remove root exports in a future breaking release

This is cleanest long-term, but not for 0.1 if examples or downstream generated projects might depend on them.

## Recommendation

Use Option A for now, add tests/docs, and schedule Option C for a breaking release.

## Steps

### 3.1 Add root export documentation

In `crates/game-kit/src/lib.rs`, add a visible section:

````rust
//! ## Import surfaces
//!
//! New beginner projects should use:
//!
//! ```ignore
//! use game_starter::prelude::*;
//! ```
//!
//! Beginner content crates should use:
//!
//! ```ignore
//! use game_kit::beginner::prelude::*;
//! ```
//!
//! Advanced content should use:
//!
//! ```ignore
//! use game_kit::advanced::prelude::*;
//! ```
//!
//! The crate root and `game_kit::prelude::*` are compatibility surfaces.
````

### 3.2 Add architecture test for docs/examples

Extend architecture tests so beginner docs/examples/templates do not contain:

```text
use game_kit::{
use game_kit::GameCtx
use game_kit::Commands
use game_kit::StartupGameCtx
game_kit::prelude::*
```

Allow these only in advanced docs.

### 3.3 Add migration note

In `docs/migrations/v0.1-to-v0.2.md`, add:

```markdown
Prefer `game_kit::beginner::prelude::*` or `game_kit::advanced::prelude::*`.
Root-level `game_kit::*` imports are compatibility-only and may be narrowed later.
```

### 3.4 Optional: add deprecations for root advanced exports later

Do not do this if it creates warning spam inside the workspace.

If you do it, apply only to a future release.

## Definition of done

```text
- Import surfaces are clearly documented.
- Beginner docs/examples/templates never use root advanced exports.
- Architecture tests prevent accidental beginner use of root advanced APIs.
- Any root-export cleanup is deferred to a planned breaking release.
```

---

# Phase 4 — Finish release-quality generated-project workflow

## Current state

Generated projects are already CI-checked, packaged, and smoke-run.

The current flow includes:

```text
game-dev new
game-dev doctor
game-dev run
game-dev package
game-dev asset-check
game-dev validate-data
generated-template CI
first-15-minutes script
```

This is strong. Now turn it into a release-quality workflow.

## Relevant files

```text
crates/game-cli/src/lib.rs
templates/simple-demo/README.md
templates/data-driven-demo/README.md
scripts/first-15-minutes.sh
.github/workflows/ci.yml
docs/tutorials/00-start-here.md
docs/tutorials/01-run-the-demo.md
docs/tutorials/10-package-your-demo.md
README.md
```

## Steps

### 4.1 Make `game-dev new` output copy-pasteable

Current output should tell the user:

```text
created demo at ...
next steps:
    cd ...
    game-dev doctor
    game-dev run
```

Improve it to include fallback if `game-dev` was run through `cargo run -p game-cli`:

```text
Next steps:
  cd my-game
  game-dev doctor
  game-dev run

If game-dev is not installed globally:
  cargo run
```

### 4.2 Add `game-dev check`

Add a single command:

```bash
game-dev check
```

It should run:

```text
doctor
asset-check
validate-data if assets/game.ron exists
cargo check
```

This gives beginners one command before asking for help.

### 4.3 Add `game-dev first-run`

Optional, but useful:

```bash
game-dev first-run
```

This runs:

```text
doctor
asset-check
cargo run
```

Could be too magical. If added, document it as a convenience.

### 4.4 Improve generated README order

Template README should start with:

````markdown
# My Game

## Run

```bash
cargo run
````

## Recommended checks

```bash
game-dev doctor
game-dev asset-check
game-dev package --release --out dist/my-game --zip
```

````

Do not start with advanced explanation.

### 4.5 Add “what files do I edit?” section

In both templates:

```text
Edit these first:
- src/main.rs for Rust beginner projects
- assets/game.ron for no-Rust projects
- assets/maps/level_1.txt for the map
- assets/textures/*.png for art
- assets/sounds/*.wav for sound
````

### 4.6 Validate generated project README in architecture tests

Add checks that both template READMEs contain:

```text
cargo run
game-dev doctor
game-dev asset-check
game-dev package
assets/maps/level_1.txt
```

## Definition of done

```text
- A generated project tells the user exactly what to do.
- `game-dev check` exists or the docs explain the current equivalent.
- Template READMEs prioritize first-run simplicity.
- CI validates generated project docs stay useful.
```

---

# Phase 5 — Make diagnostics consistently teaching-oriented

## Current state

Diagnostics are already better than prototype level:

```text
unknown asset names list known options
validate-data exists
bad map symbols are detected
game.ron identity changes explain restart requirement
asset-check decodes PNG/sound/font/text maps/TMX/LDtk
```

Now make diagnostic quality a formal release gate.

## Relevant files

```text
crates/game-kit/src/diagnostics.rs
crates/game-kit/src/assets.rs
crates/game-kit/src/data.rs
crates/game-kit/src/map.rs
crates/game-kit/src/beginner/*
crates/game-cli/src/lib.rs
docs/tutorials/common-errors.md
crates/game-core/tests/architecture_boundaries.rs
```

## Steps

### 5.1 Audit all beginner-facing `anyhow!` and `bail!`

Run:

```bash
rg "anyhow!|bail!" crates/game-kit/src crates/game-cli/src
```

Classify each error:

```text
beginner-facing
internal invariant
advanced-only
test-only
```

Beginner-facing errors should include:

```text
what failed
bad name/path/value
known valid options when applicable
fix suggestion
file/path/context where possible
```

### 5.2 Create standard diagnostic helpers

If not already centralized enough, expand `crates/game-kit/src/diagnostics.rs`:

```rust
unknown_name_error(kind, requested, known)
missing_file_error(kind, path, searched_base)
bad_map_symbol_error(map, symbol, row, col, known_symbols)
restart_required_error(kind, startup, current)
bad_rule_dependency_error(rule, missing)
bad_data_version_error(found, supported)
```

### 5.3 Use suggestions everywhere names are referenced

Apply suggestions to:

```text
textures
sounds
music
prefabs
maps
scenes
tags
actions
animation sheets
rules
script-rule names
```

### 5.4 Improve CLI errors

`game-dev` should not just say:

```text
cargo run failed
```

It should say:

```text
cargo run failed. If the error mentions SDL3, Vulkan, or glslc, run:
    game-dev doctor --explain
```

For packaging:

```text
release build failed; no package was created
```

Add:

```text
Run `game-dev check` or `game-dev doctor --explain` for setup help.
```

### 5.5 Add diagnostic snapshot-style tests

Do not snapshot exact full strings; assert useful substrings.

Tests should cover:

```text
unknown texture with suggestions
unknown prefab with suggestions
bad map symbol with row/col
duplicate start map
bad game.ron version
restart-required data reload change
missing SDL/Vulkan/glslc doctor output where testable
```

### 5.6 Update common errors doc

Update:

```text
docs/tutorials/common-errors.md
```

Add sections:

```text
Unknown asset name
Unknown prefab name
Bad text map symbol
F5 says restart required
Missing glslc
Missing Vulkan loader
SDL3 build failed
Cargo cannot find game-starter
Generated project uses old tag
Borrow error in a custom callback
```

Each section should include:

```text
symptom
why it happens
fix
example
```

## Definition of done

```text
- Beginner-facing errors teach.
- Common bad names include known valid options.
- CLI failures point to doctor/check.
- Common errors doc matches actual messages.
- Tests protect diagnostic quality.
```

---

# Phase 6 — Clarify and stabilize the tutorial path

## Problem

The beginner API is now large. That is good, but it can overwhelm beginners.

The docs are much better than before, but the project needs one obvious path.

## Relevant files

```text
README.md
docs/tutorials/README.md
docs/tutorials/*
docs/beginner-authoring.md
docs/advanced-content-authoring.md
docs/when-to-use-advanced-api.md
docs/cookbook/*
examples/*
templates/*
```

## Steps

### 6.1 Define three official learning tracks

In `README.md` and `docs/tutorials/README.md`:

```text
Track A — No Rust:
  templates/data-driven-demo
  docs/tutorials/13-data-driven-demo.md

Track B — Beginner Rust:
  templates/simple-demo
  00-start-here through 12-package-your-demo

Track C — Advanced:
  testbed-content
  docs/advanced-content-authoring.md
  docs/when-to-use-advanced-api.md
```

### 6.2 Mark cookbook as “look up recipes, not first path”

In `docs/cookbook/README.md`:

```markdown
The cookbook is for recipes after you have run the beginner tutorial.
Do not read it front-to-back first.
```

### 6.3 Ensure tutorial numbering is unique and linear

Current tutorial names look mostly cleaned:

```text
00-start-here
01-run-the-demo
02-your-first-player
03-add-a-map
04-add-an-enemy
05-add-pickups-and-score
06-add-projectiles
07-add-doors-and-levels
08-add-sound-and-music
09-add-ui-and-menu
10-package-your-demo
11-custom-behavior
12-fast-iteration
13-data-driven-demo
```

The optional older files still exist:

```text
optional-add-animation.md
optional-add-combat.md
optional-add-sound-and-ui.md
optional-package-your-demo.md
```

Make sure optional files are not linked as core steps.

### 6.4 Add “do not copy this first” banner to advanced/testbed docs

In:

```text
crates/testbed-content/README.md
docs/advanced-content-authoring.md
```

Add:

```markdown
This is advanced content. If you are new, start with `templates/simple-demo`
or `templates/data-driven-demo`.
```

### 6.5 Add architecture tests for docs

Architecture tests should assert:

```text
README contains Track A / Track B / Track C
tutorial README points to generated templates
advanced docs include “advanced” and “do not start here”
beginner docs do not import game_kit::advanced::prelude
```

## Definition of done

```text
- A beginner sees one obvious first path.
- Data-driven and beginner-Rust paths are separate.
- Advanced path is clearly labeled.
- Cookbook is not the first tutorial.
```

---

# Phase 7 — Decide distribution strategy: GitHub-only vs crates.io/template repo

## Problem

Generated templates currently use a tag-pinned Git dependency:

```toml
game-starter = { git = "https://github.com/P2949/game", tag = "v0.1.0", package = "game-starter" }
```

This is acceptable for a public prototype, but a polished beginner framework usually wants:

```text
published crates
stable template repo
versioned docs
release notes
```

## Goal

Choose and document the distribution model.

## Option A — Stay GitHub-only for now

Good for early project:

```text
- no crates.io publishing pressure
- release tags pin dependencies
- GitHub release artifacts provide demos
- migration docs live in repo
```

## Option B — Publish crates

Publish:

```text
game-core
game-map
game-ai
game-combat
game-physics
game-kit
game-runtime
game-starter
game-cli
```

This is more work because internal crate versions and API guarantees matter.

## Option C — Separate template repo only

Keep crates GitHub-based but create:

```text
P2949/game-template
```

for clean `cargo generate`.

## Recommendation

For now:

```text
1. Keep GitHub tag-pinned dependencies.
2. Add a clean template repository later.
3. Defer crates.io until API has survived one or two release cycles.
```

## Steps

### 7.1 Add distribution policy doc

Create:

```text
docs/distribution-policy.md
```

Include:

```text
Current: GitHub tag-pinned generated projects.
Next: dedicated template repo.
Later: crates.io if API stabilizes.
```

### 7.2 Update README

Add:

```markdown
Generated projects currently depend on the tagged GitHub repo.
Use release tags for reproducibility.
```

### 7.3 Add release checklist item

Before tagging:

```text
- update template default tag
- update CHANGELOG
- update migration docs
- run generated-template CI
- run release workflow
```

### 7.4 Optional: create template repo later

When ready:

```bash
cargo generate gh:P2949/game-template
```

should become the primary docs command.

## Definition of done

```text
- Distribution strategy is explicit.
- Generated projects are reproducible.
- Users know whether to use GitHub tags, template repo, or crates.io.
```

---

# Phase 8 — Keep improving script-like coverage only where examples need it

## Current state

Script-like hooks are already broad:

```text
on_player_death
on_player_respawn
on_score_reaches
on_wave_cleared
on_timer
every_seconds_while_playing
on_enemy_death
on_projectile_hit
on_collect
on_door_open
on_map_enter
on_map_exit
on_map_changed
on_scene_enter
on_scene
custom_rule builders
data conditions/effects
```

Do not add random hooks forever. Add hooks when they simplify common beginner examples.

## Relevant files

```text
crates/game-kit/src/app.rs
crates/game-kit/src/beginner/events.rs
crates/game-kit/src/beginner/collections.rs
crates/game-kit/src/beginner/custom_rules.rs
crates/game-kit/src/data.rs
examples/*
docs/cookbook/*
```

## Steps

### 8.1 Audit existing examples for repeated custom code

Run:

```bash
rg "on_|custom_rule|every_seconds|after_seconds|score\\(|spawn\\(" examples crates/simple-content templates
```

Look for duplicated patterns.

### 8.2 Add hooks only for repeated patterns

Candidate hooks if not already present:

```rust
game.on_boss_phase(...)
game.on_dialog_finished(...)
game.on_button_clicked(...)
game.on_checkpoint_reached(...)
game.on_inventory_item_collected(...)
game.on_area_empty(...)
```

But only add these if examples or cookbook recipes actually need them.

### 8.3 Keep event objects high-level

Every event object should expose:

```text
player()
enemy()
projectile()
door()
pickup()
position()
map_name()
scene_name()
score()
spawn()
play_sound()
change_scene()
```

It should not expose:

```text
EntityId
Component
GameCtx
resource
commands
```

### 8.4 Mirror common new hooks into `game.ron`

For each hook that becomes a common beginner pattern, decide whether data users need it too.

Example:

```ron
When(
    condition: ButtonClicked("start"),
    effects: [ChangeScene("level_1")]
)
```

### 8.5 Add one example per major hook family

Do not add 50 demos. Keep focused examples:

```text
events-demo
projectile-hit-demo
score-gate-demo
timer-spawn-demo
data-driven-events-demo
data-driven-waves-demo
data-driven-projectiles-demo
```

Already present. Add more only if they teach a new class of behavior.

## Definition of done

```text
- Script-like API covers common small-demo needs.
- New hooks are driven by examples/docs, not speculation.
- Event objects remain beginner-safe.
- Data DSL stays reasonably parallel to Rust beginner API.
```

---

# Phase 9 — Make full release verification reproducible

## Problem

The CI workflow is strong, but users and future contributors need a single command or checklist that reproduces the release gate locally.

## Relevant files

```text
crates/game-cli/src/lib.rs
xtask/src/main.rs
docs/release-checklist.md
README.md
```

## Goal

Add a local release verification command.

## Steps

### 9.1 Add `cargo xtask release-check`

Since `xtask` is for the repo, add:

```bash
cargo xtask release-check
```

It should run:

```text
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build -p game --release --locked
cargo build -p game-cli --locked
generated template checks
game-dev package checks
architecture greps if not covered by tests
```

Make it accept:

```bash
cargo xtask release-check --skip-smoke
cargo xtask release-check --features ci-build-sdl3
```

### 9.2 Add `game-dev check` for generated projects

For generated projects:

```bash
game-dev check
```

Should run:

```text
doctor
asset-check
validate-data if assets/game.ron exists
cargo check
```

### 9.3 Document both commands

In `README.md`:

```text
Engine contributors:
  cargo xtask release-check

Generated project users:
  game-dev check
```

## Definition of done

```text
- There is one contributor release-check command.
- There is one generated-project check command.
- Release checklist references both.
```

---

# Phase 10 — Finalize roadmap and release labels

## Problem

The project has multiple roadmap docs from previous stages:

```text
docs/architectural-improvement-roadmap.md
docs/content-authoring-api-roadmap.md
docs/beginner-authoring-roadmap.md
docs/beginner-productization-roadmap.md
```

This can confuse a reader. Some are historical, some current.

## Goal

Make it obvious which roadmap is current and which are historical.

## Steps

### 10.1 Add status banners to old roadmaps

At the top of older roadmap docs:

```markdown
> Status: Historical. The work described here has been implemented.
> Current work is tracked in `docs/beginner-productization-roadmap.md`.
```

### 10.2 Update current roadmap

At the top of `docs/beginner-productization-roadmap.md`:

```markdown
> Status: Current release-candidate polish roadmap.
```

Or, after this roadmap is complete:

```markdown
> Status: Complete for Beginner Productization 1.0.
> Remaining future work: full structural `game.ron` reload, crates.io/template repo.
```

### 10.3 Add a single roadmap index

Create:

```text
docs/roadmaps/README.md
```

or add to `docs/README.md` if it exists:

```text
- Architecture split roadmap: complete
- Content authoring API roadmap: complete
- Beginner authoring roadmap: complete
- Beginner productization roadmap: current / release-candidate
```

## Definition of done

```text
- A reader knows which roadmap matters now.
- Old roadmap docs no longer imply the architecture is unfinished.
- Current roadmap has clear complete/partial/remaining sections.
```

---

# Phase 11 — Final “fully achieved” gate

Run this when all previous phases are complete.

## 11.1 Code and tests

```bash
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build -p game --release --locked
```

## 11.2 Smoke tests

```bash
GAME_SMOKE_FRAMES=120 cargo run -p game --locked
GAME_DEMO=simple GAME_SMOKE_FRAMES=120 cargo run -p game --locked
GAME_DEMO=testbed GAME_SMOKE_FRAMES=120 cargo run -p game --locked
GAME_ASSET_DIR=assets GAME_SMOKE_FRAMES=120 cargo run -p game --release --locked
```

## 11.3 CLI checks

```bash
cargo run -p game-cli -- doctor --explain
cargo run -p game-cli -- asset-check
cargo run -p game-cli -- validate-data assets/game.ron
```

## 11.4 Generated project checks

```bash
rm -rf /tmp/game-release-check
mkdir -p /tmp/game-release-check

cargo run -p game-cli -- new /tmp/game-release-check/simple --template simple
cargo check --manifest-path /tmp/game-release-check/simple/Cargo.toml
GAME_SMOKE_FRAMES=60 cargo run --manifest-path /tmp/game-release-check/simple/Cargo.toml

cargo run -p game-cli -- new /tmp/game-release-check/data --template data-driven
cargo check --manifest-path /tmp/game-release-check/data/Cargo.toml
GAME_SMOKE_FRAMES=60 cargo run --manifest-path /tmp/game-release-check/data/Cargo.toml
```

## 11.5 Package checks

```bash
cd /tmp/game-release-check/simple
game-dev package --release --out dist/simple-demo --zip

cd /tmp/game-release-check/data
game-dev package --release --out dist/data-demo --zip
```

If `game-dev` is not globally installed:

```bash
cargo run -p game-cli -- package --release --out dist/simple-demo --zip
```

from inside the generated project.

## 11.6 Beginner API grep checks

Run:

```bash
rg "GameCtx|StartupGameCtx|EntityId|Component|Transform|Velocity|Sprite::new|Collider::box_of|Health::new|MeleeAttack|Faction|AiController|ChaseTarget|PathFollow|Patrol|game.prefab\\(|fixed_active|component::<|entities_with::<|for_each|nearest_living_with|spawn_prefab_at|commands\\(\\)|resource::<|resource_mut::<|insert_resource" \
  examples templates crates/simple-content docs/tutorials docs/beginner-authoring.md
```

Expected:

```text
No beginner-facing hits, except quoted forbidden lists or advanced-warning sections.
```

## 11.7 Release artifact check

Create a tag or run the workflow manually:

```text
.github/workflows/release.yml
```

Confirm it uploads:

```text
game-demo-linux-x86_64.zip
game-demo-windows-x86_64.zip
```

## Definition of done

```text
- Full workspace verification passes.
- Generated project verification passes.
- Package verification passes.
- Beginner API grep is clean.
- Docs status is accurate.
- Release artifacts are produced.
- The only remaining future work is explicitly labeled as post-1.0.
```

---

# Final acceptance criteria

The objective is fully achieved when this is true:

```text
1. Beginners use `game_starter::prelude::*` or `game_kit::beginner::prelude::*`.
2. Advanced users use `game_kit::advanced::prelude::*`.
3. Beginner code does not expose ECS traversal, GameCtx, EntityId, Component, or runtime/backend code.
4. Beginner callbacks receive beginner-only `Game` wrappers.
5. Beginner examples/templates/docs are architecture-tested against advanced leakage.
6. A no-Rust user can edit `assets/game.ron` and make a small demo.
7. A beginner Rust user can use the simple template and make a small demo.
8. Common custom behavior uses `on_*` hooks, events, rules, and custom-rule builders.
9. Generated projects work outside the engine workspace.
10. `game-dev` supports new, doctor, run, package, asset-check, and validate-data.
11. Packaging creates a shareable folder or zip.
12. Error messages teach rather than merely fail.
13. Tutorial path is narrow and obvious.
14. Release artifacts exist for people who want to try before building.
15. The roadmap/status docs accurately say the architecture is complete.
```

---

# Recommended commit order

Use small commits in this order:

```text
1. Run full local verification and fix any compile/test/clippy failures.
2. Update docs/beginner-productization-roadmap.md with real phase statuses.
3. Add status banners to old roadmap docs.
4. Decide and document the game.ron reload policy for 1.0.
5. Add/expand tests for the partial reload contract.
6. Add game-dev check for generated projects.
7. Add cargo xtask release-check for contributors.
8. Improve beginner-facing diagnostics and common-errors docs.
9. Tighten docs/tests around game-kit root exports and import surfaces.
10. Clarify tutorial tracks and README “which path should I use?”
11. Add distribution-policy.md covering GitHub tags vs template repo vs crates.io.
12. Polish generated template READMEs and first-run instructions.
13. Run generated project/package verification locally.
14. Run release workflow manually or on a test tag.
15. Mark Beginner Productization 1.0 complete if all gates pass.
```

# Bottom line

The architecture is complete. The beginner API is real. The script-like layer is real. The no-Rust path is real.

The final roadmap should therefore be a **release-polish roadmap**, not another refactor roadmap. The only technically substantial decision left is whether “partial `game.ron` reload” is enough for 1.0 or whether you want full structural hot reload before calling the framework truly script-like.


I inspected the attached current snapshot at `/mnt/data/game-master.zip`. I still cannot run the Rust verification commands here because this sandbox has no `cargo` or `rustc`, so this is a source-level roadmap based on the current code.

The important correction from the reference text: **the compatibility prelude gap is now closed in the current snapshot.** `game_kit::prelude` now has a real Rust `#[deprecated(note = "...")]` attribute, not just a doc comment. So that item should move from “implement” to “verify/protect with tests.”

The two still-real technical gaps I found are:

```text
1. Countdown custom-rule validation is still incomplete.
   A typo'd countdown key can still default to 0.0 and trigger immediately.

2. Tiled has importer code, tests, fixture, and cookbook docs, but no beginner-facing
   runnable example/demo crate that exercises the path.
```

There are also release-polish gaps: the roadmap status doc is still generic/in-progress, `game-dev` does not yet have a single `check` command, `xtask` does not yet have a full `release-check`, and the `game.ron` reload policy should be documented as partial-by-design unless you want to implement full structural hot reload.

# Architectural Improvement Roadmap: Final Beginner Framework Polish

## Goal

Finish the remaining work needed to honestly say:

```text
The engine/content split is complete.
The beginner API is high-level.
The no-Rust path is usable.
The script-like layer is safe and beginner-facing.
The release process proves generated projects work outside the engine workspace.
```

This is **not** a renderer/runtime rewrite roadmap. It is a final polish roadmap for making the beginner framework hard to misuse and ready to present as a release candidate.

---

# Current state

The current code already has:

```text
game_starter::prelude::*              standalone beginner entry point
game_kit::beginner::prelude::*        beginner content API
game_kit::advanced::prelude::*        advanced/testbed API
templates/simple-demo                 beginner Rust template
templates/data-driven-demo            no-Rust data template
crates/game-cli                       standalone game-dev CLI
game-dev new                          generated project creation
game-dev doctor                       setup diagnostics
game-dev run                          project runner
game-dev package                      project packaging + zip
game-dev asset-check                  asset validation
game-dev validate-data                game.ron validation
generated-template CI                 outside-workspace template check/package/smoke
release workflow                      Linux/Windows demo zip artifacts
first-15-minutes script               generated beginner acceptance test
beginner::Game wrapper                beginner callbacks do not expose GameCtx directly
architecture tests                    prevent beginner API from leaking ECS concepts
```

The main architecture goal is already achieved.

The remaining work is:

```text
- run real local verification,
- update docs/status to reflect what is already implemented,
- fix countdown custom-rule validation,
- add a runnable Tiled example,
- protect the compatibility-prelude deprecation,
- add unified check/release-check commands,
- clarify partial game.ron reload policy,
- optionally reduce root-level game-kit misuse surface later,
- keep distribution and tutorial paths clean.
```

---

# Phase 0 — Verify the current snapshot locally

## Objective

Before changing anything else, confirm the current repository actually builds and passes the full test suite.

## Why

The architecture looks correct statically, but source review is not enough. This phase decides whether the project is truly release-candidate or still has compile/test failures.

## Relevant files

```text
Cargo.toml
Cargo.lock
.github/workflows/ci.yml
.github/workflows/release.yml
crates/game-kit/*
crates/game-starter/*
crates/game-cli/*
crates/game-core/tests/architecture_boundaries.rs
templates/*
examples/*
docs/*
scripts/first-15-minutes.sh
```

## Steps

### 0.1 Run formatting

```bash
cargo fmt --all -- --check
```

If it fails:

```bash
cargo fmt --all
cargo fmt --all -- --check
```

### 0.2 Run workspace tests

```bash
cargo test --workspace --locked
```

If this fails, classify failures:

```text
- compile failure
- unit/integration test failure
- architecture-boundary test failure
- environment failure
```

Fix repository-caused failures before continuing.

### 0.3 Run clippy

```bash
cargo clippy --workspace --all-targets --locked -- -D warnings
```

Do not suppress warnings unless there is a clear reason. Prefer code cleanup.

### 0.4 Run release build

```bash
cargo build -p game --release --locked
```

### 0.5 Run smoke tests

```bash
GAME_SMOKE_FRAMES=120 cargo run -p game --locked
GAME_DEMO=simple GAME_SMOKE_FRAMES=120 cargo run -p game --locked
GAME_DEMO=testbed GAME_SMOKE_FRAMES=120 cargo run -p game --locked
GAME_ASSET_DIR=assets GAME_SMOKE_FRAMES=120 cargo run -p game --release --locked
```

### 0.6 Run source-built SDL3 path

```bash
cargo test --workspace --locked --features game/ci-build-sdl3
cargo clippy --workspace --all-targets --locked --features game/ci-build-sdl3 -- -D warnings
cargo build -p game --release --locked --features ci-build-sdl3
```

### 0.7 Run CLI checks

```bash
cargo run -p game-cli -- doctor
cargo run -p game-cli -- doctor --explain
cargo run -p game-cli -- asset-check
cargo run -p game-cli -- validate-data assets/game.ron
```

### 0.8 Run generated-project checks outside the workspace

```bash
rm -rf /tmp/game-generated-check
mkdir -p /tmp/game-generated-check

cargo run -p game-cli -- new /tmp/game-generated-check/simple --template simple
cargo check --manifest-path /tmp/game-generated-check/simple/Cargo.toml
GAME_SMOKE_FRAMES=60 cargo run --manifest-path /tmp/game-generated-check/simple/Cargo.toml

cargo run -p game-cli -- new /tmp/game-generated-check/data --template data-driven
cargo check --manifest-path /tmp/game-generated-check/data/Cargo.toml
GAME_SMOKE_FRAMES=60 cargo run --manifest-path /tmp/game-generated-check/data/Cargo.toml
```

### 0.9 Run generated-project package checks

From each generated project:

```bash
game-dev package --release --out dist/demo --zip
```

If `game-dev` is not installed globally, use the local binary path or:

```bash
cargo run -p game-cli -- package --release --out dist/demo --zip
```

from inside the generated project.

## Definition of done

```text
- fmt passes.
- tests pass.
- clippy passes.
- release build passes.
- default/simple/testbed smoke tests pass.
- generated simple project builds and smoke-runs outside workspace.
- generated data-driven project builds and smoke-runs outside workspace.
- generated project packaging creates folder + zip.
- any failures are fixed or documented as external environment issues.
```

### Phase 0 status note

- status: Done
- files changed:
  - `crates/game-cli/src/lib.rs`
  - `templates/simple-demo/cargo-generate.toml`
  - `templates/data-driven-demo/cargo-generate.toml`
  - `crates/game-core/tests/architecture_boundaries.rs`
  - `README.md`
- implementation summary:
  - Fixed `game-dev validate-data assets/game.ron` by normalizing asset-root-prefixed input before calling the asset-relative data loader.
  - Added a focused `game-cli` unit test covering both `game.ron` and `assets/game.ron`.
  - Replaced generated-project dependency defaults that pointed at missing remote tag `v0.1.0` with a reproducible commit-pinned dependency at remote `master` commit `b7fa6a3dc01d185312cf0e714b5efa10201578c6`.
  - Updated the generated-template architecture test and README wording to describe revision-pinned release-candidate templates.
- validation commands run:
  - `cargo fmt --all -- --check`
  - `cargo test --workspace --locked`
  - `cargo clippy --workspace --all-targets --locked -- -D warnings`
  - `cargo build -p game --release --locked`
  - `cargo test --workspace --locked --features game/ci-build-sdl3`
  - `cargo clippy --workspace --all-targets --locked --features game/ci-build-sdl3 -- -D warnings`
  - `cargo build -p game --release --locked --features ci-build-sdl3`
  - `cargo run -p game-cli -- doctor`
  - `cargo run -p game-cli -- doctor --explain`
  - `cargo run -p game-cli -- asset-check`
  - `cargo run -p game-cli -- validate-data assets/game.ron`
  - `cargo run -p game-cli -- new /tmp/game-generated-check/simple --template simple`
  - `cargo check --manifest-path /tmp/game-generated-check/simple/Cargo.toml`
  - `cargo run -p game-cli -- new /tmp/game-generated-check/data --template data-driven`
  - `cargo check --manifest-path /tmp/game-generated-check/data/Cargo.toml`
  - `/home/p2949/Desktop/game/target/debug/game-dev package --release --out /tmp/game-generated-check/simple/dist/simple-demo --zip`
  - `/home/p2949/Desktop/game/target/debug/game-dev package --release --out /tmp/game-generated-check/data/dist/data-demo --zip`
  - `env -u MESA_LOADER_DRIVER_OVERRIDE VK_LOADER_DRIVERS_SELECT='*lvp*' DISPLAY=:99 SDL_VIDEODRIVER=x11 cargo xtask release-check --features ci-build-sdl3`
- validation result:
  - Formatting, workspace tests, clippy, release build, SDL3 feature tests, SDL3 feature clippy, SDL3 release build, CLI checks, generated-project checks, and package zip creation pass.
  - Initial CLI verification failed because `validate-data assets/game.ron` looked for `assets/assets/game.ron`; fixed and retested successfully.
  - Initial generated-project checks failed because the remote `v0.1.0` tag does not exist; fixed by pinning generated dependencies to remote commit `b7fa6a3dc01d185312cf0e714b5efa10201578c6`.
  - Follow-up verification with Xvfb/lavapipe passed the full release gate, including graphical smoke.
- remaining caveats:
  - Direct Wayland/Intel-driver smoke in the interactive shell still exits 139 in this environment, but the documented Xvfb/lavapipe path passes and matches CI's graphical-smoke intent.
  - Cargo emits non-fatal parse diagnostics for placeholder `templates/*/Cargo.toml` files inside the git dependency checkout while compiling generated projects; generated project checks and package creation still pass.

---

# Phase 1 — Update roadmap/status docs to match the current implementation

## Objective

Make the project docs reflect reality.

## Current issue

`docs/beginner-productization-roadmap.md` still says:

```text
Productization status: in progress.
```

But the code already implements many of the roadmap items:

```text
beginner callback wrappers
game-dev CLI
generated-template CI
packaging
release workflow
script-like rules
data-driven examples
architecture tests
```

The docs are underselling the project and may confuse reviewers.

## Relevant files

```text
docs/beginner-productization-roadmap.md
docs/architectural-improvement-roadmap.md
docs/content-authoring-api-roadmap.md
docs/beginner-authoring-roadmap.md
docs/roadmaps/README.md
README.md
docs/release-checklist.md
docs/dead-code-audit.md
CHANGELOG.md
```

## Steps

### 1.1 Rewrite the status section

In `docs/beginner-productization-roadmap.md`, change status to something like:

```markdown
## Current State

- Architecture status: complete.
- Beginner Productization 1.0 status: release-candidate pending verification.
- Beginner entry points:
  - `game_starter::prelude::*` for standalone beginner projects.
  - `game_kit::beginner::prelude::*` for beginner content crates.
  - `game_kit::advanced::prelude::*` for advanced systems and testbed content.
- Remaining release-candidate items:
  - run full local verification,
  - fix countdown custom-rule validation,
  - add a runnable Tiled example,
  - decide/document the `game.ron` reload policy,
  - add unified local release-check commands,
  - keep distribution docs clear.
```

### 1.2 Replace the flat phase checklist with a status table

Use a table like:

```markdown
| Phase | Status | Current note |
| --- | --- | --- |
| Phase 0: Baseline verification | Pending local run | Needs full cargo verification on real machine. |
| Phase 1: Beginner-only context wrappers | Done | `beginner::Game` and `StartupGame` exist. |
| Phase 2: Import surfaces | Done | Beginner/advanced preludes exist; compatibility prelude is deprecated. |
| Phase 3: Generated-project CI | Done | CI checks templates outside workspace. |
| Phase 4: Standalone CLI | Done | `game-dev` exists. |
| Phase 5: Doctor diagnostics | Done / polish | `doctor --explain` exists. |
| Phase 6: Packaging | Done | `game-dev package --zip` exists. |
| Phase 7: Data reload | Partial by design | Existing values can reload; structural changes require restart. |
| Phase 8: Script-like events/rules | Done / expandable | Hooks and structured rules exist. |
| Phase 9: Diagnostics | Mostly done | Keep improving known-name checks and countdown validation. |
| Phase 10: Starter assets | Done / polish | Templates include starter assets. |
| Phase 11: Tutorial path | Done / polish | Keep one obvious beginner path. |
| Phase 12: Data DSL parity | Mostly done | Structured conditions/effects exist. |
| Phase 13: Packaging docs | Done / verify | Packaging flow exists. |
| Phase 14: Prebuilt artifacts | Done in workflow | Verify on tag. |
| Phase 15: Stability/migrations | Initial done | Keep updating per release. |
| Phase 16: Advanced separation | Done | Testbed remains advanced. |
| Phase 17: First-15-minutes test | Done | Script exists and CI calls it. |
| Phase 18: Final gate | Pending | Run before marking 1.0 complete. |
```

### 1.3 Add status banners to older roadmaps

At the top of older roadmap docs:

```markdown
> Status: Historical. The work described here has been implemented.
> Current release polish is tracked in `docs/beginner-productization-roadmap.md`.
```

Apply this to:

```text
docs/architectural-improvement-roadmap.md
docs/content-authoring-api-roadmap.md
docs/beginner-authoring-roadmap.md
```

### 1.4 Update roadmap index

If `docs/roadmaps/README.md` exists, update it. If not, create it.

It should say:

```text
Architecture split: complete
Content authoring API: complete
Beginner authoring: complete
Beginner productization: release-candidate polish
```

### 1.5 Update README status

Add a concise project status section:

```markdown
## Project status

The engine/content split and beginner authoring foundation are implemented.
The project is in beginner-productization release-candidate polish.

Start with:
- `templates/data-driven-demo` if you do not know Rust,
- `templates/simple-demo` for beginner Rust,
- `game_kit::advanced::prelude::*` only when beginner APIs are not enough.
```

## Definition of done

```text
- Current roadmap status is accurate.
- Older roadmaps are clearly historical.
- README no longer implies the foundation is unfinished.
- Release checklist names the remaining release gates.
```

### Phase 1 status note

- status: Done
- files changed:
  - `docs/beginner-productization-roadmap.md`
  - `docs/architectural-improvement-roadmap.md`
  - `docs/content-authoring-api-roadmap.md`
  - `docs/beginner-authoring-roadmap.md`
  - `docs/roadmaps/README.md`
  - `README.md`
  - `docs/release-checklist.md`
  - `docs/dead-code-audit.md`
  - `docs/tutorials/00-start-here.md`
  - `CHANGELOG.md`
- implementation summary:
  - Updated the beginner productization roadmap from generic "in progress" wording to release-candidate polish status with a phase status table.
  - Added historical status banners to older roadmap docs and created a roadmap index.
  - Updated README project status to say the engine/content split and beginner authoring foundation are implemented, while keeping advanced import details out of the early beginner section.
  - Added beginner-productization gates to the release checklist.
  - Updated dead-code audit and changelog wording so implemented surfaces are not described as future architecture work.
  - Replaced stale "release tag" beginner setup wording with release-candidate revision wording where the current generated-template pin requires it.
- validation commands run:
  - `cargo test -p game-core --test architecture_boundaries --locked`
  - `cargo test -p game-cli --locked`
  - `rg -n 'Productization status: in progress|Phase Checklist|pinned \`game-starter\` release tag|tag = "v0\\.1\\.0"|default = '\\''\\{ git = "https://github.com/P2949/game", package = "game-starter" \\}'\\''' README.md docs templates crates -S`
- validation result:
  - Architecture boundary tests pass.
  - `game-cli` tests pass.
  - Stale-status scan finds only the architecture test's forbidden moving-branch fixture.
- remaining caveats:
  - The README's early project-status section intentionally says "advanced guide" instead of spelling out `game_kit::advanced::prelude::*`; the exact import remains in advanced/import-surface docs where the architecture tests allow it.

---

# Phase 2 — Fix countdown custom-rule validation

## Objective

Prevent typo’d countdown keys from triggering effects immediately.

## Current problem

`tick_countdown` currently does:

```rust
let remaining = values.get_f32(key).unwrap_or_default() - dt.max(0.0);
```

If the actor has the tag but does **not** have the named data key, `get_f32(key)` returns `None`, `unwrap_or_default()` becomes `0.0`, and the countdown expires immediately.

`validate_custom_rules` checks:

```text
- rule tag is non-empty
- rule key is non-empty
- rule tag exists in known tags
- effects reference known tags/prefabs/sounds/maps/scenes
```

But it does **not** verify:

```text
At least one prefab carrying that tag declares the countdown data key.
```

So this typo is possible:

```ron
Trigger((name: "bomb", tags: ["explosive"], data: {"fuse": 3.0}))

custom_rules: [
    Countdown((name: "explode", tag: "explosive", key: "fues", when_zero: [...]))
]
```

The typo `fues` should fail validation. Today it likely triggers immediately.

## Relevant files

```text
crates/game-kit/src/data.rs
crates/game-kit/src/beginner/custom_rules.rs
docs/tutorials/common-errors.md
templates/data-driven-demo/assets/game.ron
examples/data-driven-full-demo/assets/game.ron
examples/data-driven-events-demo/assets/game.ron
examples/data-driven-waves-demo/assets/game.ron
examples/data-driven-projectiles-demo/assets/game.ron
```

## Implementation plan

### 2.1 Build prefab tag/data metadata during validation

In `crates/game-kit/src/data.rs`, create helper metadata from `file.prefabs`.

Add a private struct:

```rust
#[derive(Default)]
struct PrefabDataIndex {
    tags: BTreeSet<String>,
    tag_to_data_keys: BTreeMap<String, BTreeSet<String>>,
    tag_to_prefabs: BTreeMap<String, Vec<String>>,
}
```

Add helper:

```rust
fn build_prefab_data_index(prefabs: &[BeginnerPrefabFile]) -> PrefabDataIndex
```

For every prefab:

1. Get its name.
2. Get its tags.
3. Get its `data` map keys.
4. Insert each tag into `tag_to_data_keys[tag]`.
5. Insert prefab name into `tag_to_prefabs[tag]`.

You need helper methods on `BeginnerPrefabFile`:

```rust
impl BeginnerPrefabFile {
    fn name(&self) -> &str;
    fn tags(&self) -> &[String];
    fn data(&self) -> &BTreeMap<String, f32>;
}
```

If those already exist, reuse them.

### 2.2 Extend `ValidationNames`

Current:

```rust
struct ValidationNames<'a> {
    prefabs: &'a [&'a str],
    sounds: &'a [&'a str],
    music: &'a [&'a str],
    maps: &'a [&'a str],
    scenes: &'a [&'a str],
    tags: &'a [&'a str],
}
```

Add either:

```rust
prefab_data: &'a PrefabDataIndex,
```

or pass `PrefabDataIndex` separately to `validate_custom_rules`.

Recommended:

```rust
fn validate_custom_rules(
    label: &str,
    custom_rules: &[CustomRuleFile],
    names: &ValidationNames<'_>,
    prefab_data: &PrefabDataIndex,
) -> Result<()>
```

Keep `ValidationNames` simple.

### 2.3 Validate countdown key exists for the selected tag

Inside:

```rust
CustomRuleFile::Countdown(rule) => { ... }
```

after validating known tag, add:

```rust
validate_countdown_key_for_tag(label, rule, prefab_data)?;
```

Implement:

```rust
fn validate_countdown_key_for_tag(
    label: &str,
    rule: &CountdownRuleFile,
    prefab_data: &PrefabDataIndex,
) -> Result<()> {
    let Some(keys) = prefab_data.tag_to_data_keys.get(&rule.tag) else {
        // This should already be caught by require_known(tag), but keep a robust message.
        anyhow::bail!(...);
    };

    if keys.contains(&rule.key) {
        return Ok(());
    }

    let prefabs = prefab_data
        .tag_to_prefabs
        .get(&rule.tag)
        .cloned()
        .unwrap_or_default();

    let known = keys.iter().map(String::as_str).collect::<Vec<_>>();

    anyhow::bail!(
        "beginner game file '{label}' custom rule '{}' counts down key '{}' on tag '{}', but no prefab with that tag declares that data key.\n\nPrefabs with tag '{}': {}\nKnown data keys for tag '{}': {}\n\nFix: add data: {{\"{}\": 3.0}} to one of those prefabs, or change the rule key to one of the known data keys.",
        rule.name,
        rule.key,
        rule.tag,
        rule.tag,
        format_list(&prefabs),
        rule.tag,
        format_known_names(&known),
        rule.key,
    );
}
```

Use existing diagnostics helpers if available.

### 2.4 Handle empty known data keys clearly

If the tag exists but none of the tagged prefabs has data keys:

```text
Known data keys for tag 'explosive': none
```

Message should be explicit:

```text
No prefab tagged 'explosive' declares any data keys.
```

### 2.5 Add suggestions for typo’d keys

If there are keys and the requested key is close to one, suggest:

```text
Did you mean 'fuse'?
```

Use existing edit-distance helper if present. If not, implement a small local helper or reuse diagnostics.

### 2.6 Decide whether all or any prefabs with tag must carry the key

For 1.0, require **at least one** prefab with the tag to declare the key.

Reason: some tags may be shared across related actors, and only some may need the countdown.

Later, if confusing, introduce stricter validation:

```text
all prefabs with countdown tag must declare the key
```

But that may be too strict for beginner data.

### 2.7 Make runtime safe even if validation is bypassed

Change `tick_countdown` to avoid immediate trigger on missing key.

Current:

```rust
let remaining = values.get_f32(key).unwrap_or_default() - dt.max(0.0);
```

Safer:

```rust
let Some(current) = values.get_f32(key) else {
    log::warn!(
        "custom countdown key '{key}' was missing on an actor; validation should have caught this"
    );
    return None;
};
let remaining = current - dt.max(0.0);
```

This prevents bad runtime behavior even if validation is skipped or a Rust-authored custom rule has a typo.

### 2.8 Add unit tests

In `crates/game-kit/src/data.rs` tests:

```rust
#[test]
fn custom_countdown_rejects_unknown_data_key_for_known_tag() { ... }

#[test]
fn custom_countdown_accepts_declared_data_key_for_known_tag() { ... }

#[test]
fn custom_countdown_error_lists_known_data_keys_and_prefabs() { ... }
```

Also add a lower-level test in `custom_rules.rs` if accessible:

```rust
#[test]
fn countdown_missing_key_does_not_expire_immediately() { ... }
```

If private function access is awkward, test through a small plugin/harness.

### 2.9 Update common errors docs

In `docs/tutorials/common-errors.md`, add:

````markdown
## Custom countdown key is unknown

Symptom:
`custom rule 'explode' counts down key 'fues' on tag 'explosive', but no prefab with that tag declares that data key`.

Fix:
Add the key to the prefab:

```ron
Trigger((name: "bomb", tags: ["explosive"], data: {"fuse": 3.0}))
````

or change the rule to use the existing key.

````

## Definition of done

```text
- Validation rejects countdown keys not declared by any prefab with the selected tag.
- Error message names the rule, tag, missing key, relevant prefabs, and known data keys.
- Runtime no longer treats missing keys as 0.0.
- Tests cover valid key, invalid key, and useful diagnostic text.
- Common errors doc explains the fix.
````

### Phase 2 status note

- status: Done
- files changed:
  - `crates/game-kit/src/data.rs`
  - `crates/game-kit/src/beginner/custom_rules.rs`
  - `docs/tutorials/common-errors.md`
  - `docs/beginner-productization-roadmap.md`
  - `CHANGELOG.md`
- implementation summary:
  - Added a prefab tag/data-key index during `game.ron` validation.
  - Countdown custom rules now require at least one prefab with the selected tag to declare the countdown key.
  - Countdown diagnostics name the rule, tag, missing key, prefabs with the tag, known data keys, close spelling suggestions, and a concrete fix.
  - Runtime countdown ticking now ignores missing keys and logs a warning instead of treating missing keys as zero.
  - Added data validation tests for valid countdown keys, typo'd keys, and tags with no data keys.
  - Added a Rust custom-rule runtime test proving a bypassed typo does not immediately expire.
  - Updated common-errors docs, changelog, and productization status wording.
- validation commands run:
  - `cargo fmt --all -- --check`
  - `cargo test -p game-kit --locked custom_countdown`
  - `cargo test -p game-kit --locked countdown_missing_key_does_not_expire_immediately`
  - `cargo test --workspace --locked`
  - `cargo clippy --workspace --all-targets --locked -- -D warnings`
  - `cargo test -p game-core --test architecture_boundaries --locked`
  - `cargo run -p game-cli -- validate-data assets/game.ron`
- validation result:
  - All commands passed.
  - `assets/game.ron` remains valid under the stricter countdown validation.
- remaining caveats:
  - Validation intentionally follows the 1.0 policy of requiring at least one prefab with the tag to declare the key, not every prefab with the tag.

---

# Phase 3 — Add a beginner-facing Tiled demo

## Objective

Make the Tiled importer visible and acceptance-tested as a real beginner workflow, not just an internal test/cookbook feature.

## Current state

Tiled exists in:

```text
crates/game-map/src/tiled.rs
crates/game-kit/src/map.rs
crates/game-kit/tests/map_flow.rs
assets/maps/tiled_demo.tmx
docs/cookbook/tiled.md
docs/future-editor-import.md
```

But there is no runnable example under:

```text
examples/tiled-demo
```

That means a beginner cannot copy a working Tiled example the way they can copy LDtk or data-driven examples.

## Relevant files

```text
Cargo.toml
examples/tiled-demo/Cargo.toml
examples/tiled-demo/src/main.rs
examples/tiled-demo/assets/*
assets/maps/tiled_demo.tmx
docs/cookbook/tiled.md
docs/tutorials/13-data-driven-demo.md
docs/future-editor-import.md
README.md
.github/workflows/ci.yml
crates/game-core/tests/architecture_boundaries.rs
```

## Implementation plan

### 3.1 Add workspace member

In root `Cargo.toml`, add:

```toml
"examples/tiled-demo",
```

near `examples/ldtk-demo`.

### 3.2 Create `examples/tiled-demo/Cargo.toml`

Use the same style as other beginner examples:

```toml
[package]
name = "tiled-demo"
version.workspace = true
edition.workspace = true
publish = false

[dependencies]
anyhow.workspace = true
game-starter = { path = "../../crates/game-starter" }

[features]
ci-build-sdl3 = ["game-starter/ci-build-sdl3"]
```

Adjust the feature wiring to match current example conventions.

### 3.3 Add example assets

Option A: reuse root assets by copying needed files into the example:

```text
examples/tiled-demo/assets/maps/tiled_demo.tmx
examples/tiled-demo/assets/textures/player.png
examples/tiled-demo/assets/textures/slime.png
examples/tiled-demo/assets/textures/floor.png
examples/tiled-demo/assets/textures/wall.png
examples/tiled-demo/assets/sounds/hit.wav
examples/tiled-demo/assets/fonts/DejaVuSans.ttf
```

Option B: reference root assets through `GAME_ASSET_DIR=assets`, but examples are clearer if self-contained.

Recommended: self-contained assets.

### 3.4 Write `examples/tiled-demo/src/main.rs`

Use `game_starter::prelude::*` only.

Example shape:

```rust
use game_starter::prelude::*;

fn main() -> Result<()> {
    run_game("Tiled Demo", |game| {
        game.assets_from_folders()
            .required_textures(["player", "slime", "floor", "wall"])?
            .required_sounds(["hit"])?
            .build();

        let controls = game.input(|input| input.top_down_controls())?;

        game.player_prefab("player")
            .sprite("player")
            .moves_with(controls.movement, 130.0)
            .health(100)
            .melee(30.0, 25)
            .build()?;

        game.enemy_prefab("slime")
            .sprite("slime")
            .chases_player()
            .health(40)
            .melee(26.0, 6)
            .build()?;

        game.map_from_tiled("level_1", "maps/tiled_demo.tmx")
            .object("Player", "player")
            .object("Slime", "slime")
            .simple_theme("floor", "wall")
            .start();

        game.use_top_down_game()
            .controls(controls)
            .with_enemy_chase()
            .with_melee_combat()
            .with_collision()
            .with_camera_follow()
            .with_pause_death_ui()
            .build();

        Ok(())
    })
}
```

If the existing `tiled_demo.tmx` object names differ, match those exactly.

### 3.5 Add smoke test to CI

In `.github/workflows/ci.yml`, add a smoke step:

```bash
GAME_SMOKE_FRAMES=60 cargo run -p tiled-demo --locked --features ci-build-sdl3
```

Use the same Xvfb/lavapipe environment as other smoke tests.

### 3.6 Add architecture test coverage

Update `crates/game-core/tests/architecture_boundaries.rs` so `examples/tiled-demo` is in beginner example scans.

It should forbid:

```text
GameCtx
EntityId
Component
Transform
Velocity
Sprite::new
Collider::box_of
Health::new
game_kit::advanced::prelude
```

It should require:

```text
use game_starter::prelude::*;
map_from_tiled(
.object(
.simple_theme(
```

### 3.7 Update docs

In `docs/cookbook/tiled.md`, add:

````markdown
Runnable example:

```bash
cargo run -p tiled-demo
````

````

In README’s examples list, add:

```text
examples/tiled-demo — beginner Tiled TMX object/collision import
````

In `docs/future-editor-import.md`, remove any implication that Tiled is only future-facing if current importer is already usable.

## Definition of done

```text
- `examples/tiled-demo` exists.
- It uses `game_starter::prelude::*`.
- It imports a `.tmx` map through `map_from_tiled`.
- It maps Tiled objects to beginner prefabs.
- It runs through the same beginner top-down rules.
- CI builds/smoke-runs it.
- Architecture tests include it in beginner-surface checks.
- Cookbook links to the runnable example.
```

Status note:

- status: `Done`
- files changed: `Cargo.toml`, `Cargo.lock`, `examples/tiled-demo/Cargo.toml`, `examples/tiled-demo/src/main.rs`, `.github/workflows/ci.yml`, `crates/game-core/tests/architecture_boundaries.rs`, `docs/cookbook/tiled.md`, `docs/tutorials/13-data-driven-demo.md`, `docs/future-editor-import.md`, `README.md`, `CHANGELOG.md`, `docs/beginner-productization-roadmap.md`, `plans/plan.md`
- implementation summary: Added `examples/tiled-demo` as a workspace member using `game_starter::prelude::*`, mapping `assets/maps/tiled_demo.tmx` objects to beginner player/enemy prefabs and the beginner top-down rules. Added CI Xvfb smoke coverage, architecture-boundary checks, README/cookbook/tutorial/future-editor docs, and roadmap/changelog status. The demo follows the existing workspace-example convention used by `examples/ldtk-demo` and reuses root assets instead of duplicating per-example assets.
- validation commands run: `cargo check -p tiled-demo --locked`; `cargo check -p tiled-demo --locked --features ci-build-sdl3`; `cargo fmt --all -- --check`; `cargo test -p game-core --test architecture_boundaries --locked`; `GAME_SMOKE_FRAMES=60 cargo run -p tiled-demo --locked`; `cargo test --workspace --locked`; `cargo clippy --workspace --all-targets --locked -- -D warnings`; `env -u MESA_LOADER_DRIVER_OVERRIDE VK_LOADER_DRIVERS_SELECT='*lvp*' DISPLAY=:99 SDL_VIDEODRIVER=x11 cargo xtask release-check --features ci-build-sdl3`
- validation result: Cargo check, formatting, architecture tests, full workspace tests, clippy, and the full release-check gate passed. The release-check run smoke-tested `tiled-demo` through Xvfb/lavapipe.
- remaining caveats: CI smoke coverage was added but not executed from this environment; the equivalent local Xvfb/lavapipe smoke path passed.

---

# Phase 4 — Protect the compatibility-prelude deprecation

## Objective

Keep the broad `game_kit::prelude::*` from creeping back into beginner code.

## Current state

In the current snapshot, this is now implemented:

```rust
#[deprecated(note = "Use game_kit::beginner::prelude::* or game_kit::advanced::prelude::*")]
pub mod prelude { ... }
```

So the old gap is closed.

The work now is to protect it.

## Relevant files

```text
crates/game-kit/src/lib.rs
crates/game-core/tests/architecture_boundaries.rs
docs/beginner-authoring.md
docs/advanced-content-authoring.md
examples/*
templates/*
crates/simple-content/*
crates/arena-content/*
```

## Steps

### 4.1 Keep the architecture test

Ensure `architecture_boundaries.rs` has a test equivalent to:

```rust
#[test]
fn game_kit_compatibility_prelude_is_visibly_deprecated() {
    let source = fs::read_to_string(root.join("crates/game-kit/src/lib.rs")).unwrap();

    assert!(
        source.contains("#[deprecated(note = \"Use game_kit::beginner::prelude::* or game_kit::advanced::prelude::*\")]")
    );
    assert!(source.contains("Compatibility prelude."));
}
```

It appears to exist in the current snapshot. Keep it.

### 4.2 Forbid compatibility prelude in beginner-facing code

Ensure beginner scans forbid:

```text
game_kit::prelude::*
```

and prefer:

```text
game_starter::prelude::*
game_kit::beginner::prelude::*
```

### 4.3 Allow advanced prelude only in advanced material

Allowed:

```text
crates/testbed-content
docs/advanced-content-authoring.md
docs/when-to-use-advanced-api.md
advanced tests
```

Forbidden:

```text
templates/simple-demo
templates/data-driven-demo
examples/beginner-*
examples/no-rust-shapes-demo
docs/beginner-authoring.md
docs/tutorials beginner sections
```

### 4.4 Optional: defer root export narrowing

The crate root still broadly exports advanced types like:

```rust
pub use context::{Commands, GameCtx, StartupGameCtx};
```

Do **not** remove these in this patch unless you want a breaking API cleanup.

Instead, document:

```text
Root-level exports are compatibility/internal-convenience surface.
New code should use beginner or advanced preludes.
```

## Definition of done

```text
- `game_kit::prelude` has a real `#[deprecated]` attribute.
- Tests protect the deprecation.
- Beginner-facing files do not use compatibility prelude.
- Root export cleanup is either deferred or documented.
```

Status note:

- status: `Done`
- files changed: `crates/game-core/tests/architecture_boundaries.rs`, `plans/plan.md`
- implementation summary: Kept the real `#[deprecated]` compatibility prelude in `crates/game-kit/src/lib.rs`, preserved the deprecation architecture test, and broadened the beginner-facing compatibility-prelude scan to include `examples/ldtk-demo` and `examples/tiled-demo`. Root-level exports remain compatibility/internal-convenience surface and are documented in `docs/ARCHITECTURE.md` rather than narrowed in this release-polish pass.
- validation commands run: `cargo fmt --all -- --check`; `cargo test -p game-core --test architecture_boundaries --locked`
- validation result: passed; all 48 architecture-boundary tests passed.
- remaining caveats: none.

---

# Phase 5 — Clarify the `game.ron` reload policy

## Objective

Make the current partial reload behavior explicit and well-tested.

## Current state

The project has:

```text
BeginnerFileRuntime
BeginnerReloadLevel
BeginnerRuntimeConfig
reload status in debug overlay
validate-data command
identity checks for structural lists
partial reload of existing values/map paths/rule settings
```

But structural changes still require restart.

This is okay, but it should be documented as intentional.

## Relevant files

```text
crates/game-kit/src/data.rs
crates/game-kit/src/beginner/debug.rs
crates/game-kit/src/beginner/defaults.rs
docs/tutorials/12-fast-iteration.md
docs/tutorials/13-data-driven-demo.md
templates/data-driven-demo/README.md
docs/beginner-productization-roadmap.md
```

## Recommended 1.0 policy

Use this policy:

```text
F5 reload is partial by design for 1.0.

Reloads:
- text map file contents,
- existing map file paths,
- existing tuning values,
- existing prefab numeric values,
- existing custom countdown rule values,
- existing scene text/menu/audio settings,
- registered texture/sound file contents.

Requires restart:
- adding/removing/reordering assets,
- adding/removing/reordering prefabs,
- adding/removing/reordering maps,
- adding/removing/reordering actions,
- adding/removing/reordering scenes,
- adding/removing/reordering rules,
- adding brand-new asset keys.
```

## Steps

### 5.1 Update fast iteration docs

In `docs/tutorials/12-fast-iteration.md`, add a table:

```markdown
| Change | F5 reload? | Notes |
| --- | --- | --- |
| Edit existing text map file | Yes | Current map can respawn. |
| Change map path for existing map | Yes | Uses existing map identity. |
| Change existing prefab values | Partial | Existing runtime config updates. |
| Add a new prefab | No | Restart required. |
| Add a new texture key | No | Restart required. |
| Replace PNG/WAV for existing key | Yes | Asset reload path. |
| Add a new action | No | Restart required because action IDs are build-time. |
```

### 5.2 Update data-driven tutorial

In `docs/tutorials/13-data-driven-demo.md`, add:

```text
When editing `assets/game.ron`, changes to existing values are reloadable where noted.
Adding/removing/reordering structural lists requires restart.
```

### 5.3 Add debug overlay text if not already visible enough

Ensure overlay says one of:

```text
game.ron reload: partial
game.ron reload: restart required for structural changes
last reload error: ...
```

### 5.4 Add tests for reload policy

Add tests in `crates/game-kit/src/data.rs` for:

```text
- existing countdown rule values reload,
- existing map path reload,
- adding prefab returns restart-required diagnostic,
- adding map returns restart-required diagnostic,
- adding action returns restart-required diagnostic,
- adding scene returns restart-required diagnostic,
- adding asset key returns restart-required diagnostic.
```

### 5.5 Decide whether to add a reload-compatibility CLI

Optional but useful:

```bash
game-dev validate-reload old.ron new.ron
```

or:

```bash
game-dev validate-data assets/game.ron --reload-compatible previous.ron
```

This can tell users:

```text
This change can reload with F5.
This change requires restart because prefabs changed.
```

If implementing this adds too much complexity, document it as future work.

## Definition of done

```text
- Docs accurately explain partial reload.
- Debug overlay matches docs.
- Tests cover reloadable and restart-required categories.
- No one expects full live scripting reload unless explicitly planned for later.
```

Status note:

- status: `Done`
- files changed: `crates/game-kit/src/data.rs`, `docs/tutorials/12-fast-iteration.md`, `docs/tutorials/13-data-driven-demo.md`, `templates/data-driven-demo/README.md`, `plans/plan.md`
- implementation summary: Added explicit restart-required F5 tests for added prefabs, added maps, added scene flow, and action identity changes, complementing existing tests for map path reload, prefab value reload, countdown value reload, scene/audio/action value reload, enabled-rule rejection, and added asset keys. Added the fast-iteration reload policy table and aligned data-driven tutorial/template wording with the partial reload contract.
- validation commands run: `cargo fmt --all -- --check`; `cargo test -p game-kit --locked f5_`; `cargo test -p game-core --test architecture_boundaries --locked`
- validation result: passed; focused reload tests and all 48 architecture-boundary tests passed.
- remaining caveats: no reload-compatibility CLI was added in this phase; the current policy remains documented and tested, and a CLI compatibility checker can remain future work unless later phases require it.

---

# Phase 6 — Add `game-dev check` for generated projects

## Objective

Give beginners one command that checks the common things before asking for help.

## Current state

`game-dev` supports:

```text
new
doctor
run
package
asset-check
validate-data
```

It does **not** currently support:

```text
game-dev check
```

## Relevant files

```text
crates/game-cli/src/lib.rs
templates/simple-demo/README.md
templates/data-driven-demo/README.md
docs/tutorials/common-errors.md
docs/tutorials/01-run-the-demo.md
.github/workflows/ci.yml
```

## Desired behavior

From a generated project:

```bash
game-dev check
```

should run:

```text
1. doctor
2. asset-check
3. validate-data assets/game.ron if it exists
4. cargo check
```

It should not package or run the game.

## Implementation steps

### 6.1 Add CLI command dispatch

In `crates/game-cli/src/lib.rs`, add:

```rust
Some("check") => {
    reject_extra(args, "check")?;
    check_project()
}
```

Update usage string:

```text
game-dev check
```

### 6.2 Implement `check_project`

Pseudo-code:

```rust
fn check_project() -> Result<()> {
    println!("checking project setup...");
    doctor(DoctorOptions { explain: false });

    println!("checking assets...");
    validate_assets_dir(&env::current_dir()?.join("assets"), false)?;

    let game_file = env::current_dir()?.join("assets").join("game.ron");
    if game_file.exists() {
        println!("checking data file...");
        game_kit::data::validate_beginner_game_file(&game_file)?;
    }

    println!("running cargo check...");
    let status = Command::new("cargo")
        .arg("check")
        .status()
        .context("could not run cargo check")?;

    if !status.success() {
        bail!(
            "cargo check failed.\n\nIf the error mentions SDL3, Vulkan, or glslc, run:\n    game-dev doctor --explain\n\nIf it mentions a missing asset or game.ron name, run:\n    game-dev asset-check\n    game-dev validate-data assets/game.ron"
        );
    }

    println!("project check passed");
    Ok(())
}
```

### 6.3 Make doctor non-fatal or fatal?

Current `doctor` appears to print diagnostics but not necessarily return an error. Decide:

```text
Option A: doctor is advisory inside check.
Option B: doctor can fail check if critical tools are missing.
```

For beginners, use Option A initially unless doctor already has clean severity support. `cargo check` will catch build-critical issues.

### 6.4 Update template READMEs

Add:

```bash
game-dev check
```

Before packaging.

### 6.5 Add CI check

In generated-template CI, after building `game-dev`, run:

```bash
target/debug/game-dev check
```

inside each generated project.

## Definition of done

```text
- `game-dev check` exists.
- It runs doctor, asset-check, validate-data when applicable, and cargo check.
- Template READMEs mention it.
- Generated-template CI runs it.
- Failure messages point to doctor/common-errors.
```

Status note:

- status: `Done`
- files changed: `crates/game-cli/src/lib.rs`, `crates/game-core/tests/architecture_boundaries.rs`, `.github/workflows/ci.yml`, `templates/simple-demo/README.md`, `templates/data-driven-demo/README.md`, `README.md`, `docs/tutorials/00-start-here.md`, `docs/tutorials/01-run-the-demo.md`, `docs/tutorials/common-errors.md`, `docs/release-checklist.md`, `docs/beginner-productization-roadmap.md`, `CHANGELOG.md`, `plans/plan.md`
- implementation summary: Added `game-dev check [--features feature-list]`, which runs advisory `doctor`, validates the configured asset root, validates `assets/game.ron` when present, then runs `cargo check`. Generated-project next steps, template docs, README/tutorial/common-errors guidance, release checklist, changelog, CI, and architecture-boundary protections now include the command.
- validation commands run: `cargo fmt --all -- --check`; `cargo test -p game-cli --locked`; `cargo test -p game-core --test architecture_boundaries --locked`; `cargo run -p game-cli -- new /tmp/game-phase6-check/simple --template simple`; `cargo run -p game-cli -- new /tmp/game-phase6-check/data --template data-driven`; `/home/p2949/Desktop/game/target/debug/game-dev check --features ci-build-sdl3` from `/tmp/game-phase6-check/simple`; `/home/p2949/Desktop/game/target/debug/game-dev check --features ci-build-sdl3` from `/tmp/game-phase6-check/data`
- validation result: passed; both generated projects completed `game-dev check`. Cargo still prints the known nonfatal placeholder-template diagnostics from the pinned git checkout before finishing successfully.
- remaining caveats: CI will run the new helper steps in GitHub Actions; this local verification used the same `ci-build-sdl3` feature path but did not execute the workflow itself.

---

# Phase 7 — Add `cargo xtask release-check` for contributors

## Objective

Give engine contributors one command that runs the release-candidate gate locally.

## Current state

`xtask` supports:

```text
new-demo
doctor
package-demo
```

It does not support:

```text
cargo xtask release-check
```

## Relevant files

```text
xtask/src/main.rs
crates/game-cli/src/lib.rs
docs/release-checklist.md
README.md
```

## Desired behavior

From the engine repo:

```bash
cargo xtask release-check
```

should run:

```text
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build -p game --release --locked
cargo run -p game-cli -- doctor --explain
cargo run -p game-cli -- asset-check
cargo run -p game-cli -- validate-data assets/game.ron
generated project checks
package checks
```

Support options:

```bash
cargo xtask release-check --skip-smoke
cargo xtask release-check --skip-generated
cargo xtask release-check --features ci-build-sdl3
```

## Implementation plan

### 7.1 Add xtask dispatch

In `run_xtask`:

```rust
Some("release-check") => release_check_command(args, &workspace),
```

Update usage string.

### 7.2 Implement command runner helper

Add helper:

```rust
fn run_command(command: &mut Command, label: &str) -> Result<()>
```

It should print:

```text
==> cargo test --workspace --locked
```

and fail with context.

### 7.3 Implement basic release check first

Start with:

```rust
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build -p game --release --locked
```

### 7.4 Add optional generated-project check

Generate into `/tmp/game-release-check` or `target/release-check/generated`.

Use `game-dev new` or internal `new_project`.

Run:

```text
cargo check
game-dev check
game-dev package --release --out ...
```

### 7.5 Add smoke toggle

Default can skip graphical smoke if this is meant to be fast, or include smoke if it is meant to be the full release gate.

Recommended:

```text
release-check: no graphical smoke by default
release-check --smoke: includes smoke
```

But if you want strict release proof, include smoke by default and offer `--skip-smoke`.

### 7.6 Update docs

In `docs/release-checklist.md`:

```bash
cargo xtask release-check
```

In README contributor section:

```bash
cargo xtask release-check --skip-smoke
```

## Definition of done

```text
- `cargo xtask release-check` exists.
- It runs the important contributor-side checks.
- It supports skip flags for slow/graphical parts.
- Release checklist references it.
```

Status note:

- status: `Done`
- files changed: `crates/game-cli/src/lib.rs`, `crates/game-core/tests/architecture_boundaries.rs`, `README.md`, `docs/release-checklist.md`, `docs/beginner-productization-roadmap.md`, `CHANGELOG.md`, `plans/plan.md`
- implementation summary: Added `cargo xtask release-check [--skip-smoke] [--skip-generated] [--features feature-list]` through the shared `game-cli` xtask implementation. The gate runs formatting, workspace tests, headless runtime tests, clippy, release build, CLI doctor/asset/data checks, generated simple/data project cargo checks, generated `game-dev check`, and generated release package zips. Graphical smoke checks run by default and are skipped only with `--skip-smoke`.
- validation commands run: `cargo test -p game-cli --locked`; `cargo fmt --all -- --check`; `cargo test -p game-core --test architecture_boundaries --locked`; `cargo xtask release-check --skip-smoke --features ci-build-sdl3`; `env -u MESA_LOADER_DRIVER_OVERRIDE VK_LOADER_DRIVERS_SELECT='*lvp*' DISPLAY=:99 SDL_VIDEODRIVER=x11 cargo xtask release-check --features ci-build-sdl3`
- validation result: passed after fixing clippy-reported needless borrows and moving generated release-check projects from workspace `target/` to `/tmp/game-release-check/generated` so Cargo treats them as outside-workspace projects. The passing release-check runs completed generated simple/data checks, zip packaging, and the full Xvfb/lavapipe graphical-smoke gate.
- remaining caveats: none for the local release-check command; remote release artifacts still require the GitHub release workflow.

---

# Phase 8 — Improve beginner diagnostics around rules and data

## Objective

Make beginner errors teach users instead of merely failing.

## Current state

Diagnostics are already decent. The countdown validation fix from Phase 2 should become the model for remaining beginner data diagnostics.

## Relevant files

```text
crates/game-kit/src/diagnostics.rs
crates/game-kit/src/data.rs
crates/game-kit/src/beginner/custom_rules.rs
crates/game-kit/src/beginner/rules.rs
crates/game-kit/src/assets.rs
crates/game-cli/src/lib.rs
docs/tutorials/common-errors.md
```

## Steps

### 8.1 Audit beginner-facing `anyhow!` and `bail!`

Run:

```bash
rg "anyhow!|bail!" crates/game-kit/src crates/game-cli/src
```

For each beginner-facing error, ensure it includes:

```text
- what failed,
- the bad name/path/value,
- known valid options if applicable,
- a suggested fix.
```

### 8.2 Improve rule dependency errors

For rule chains like:

```rust
game.rules().projectiles_damage_enemies().build();
```

ensure errors say:

```text
Rule `projectiles_damage_enemies` needs projectile prefabs or projectile rules.
Add `.projectiles()` or define a projectile prefab.
```

### 8.3 Extend suggestions to more name classes

Use suggestions for:

```text
textures
sounds
music
prefabs
maps
scenes
tags
actions
animation sheets
script-rule references
```

### 8.4 Improve CLI failure advice

For `game-dev run`, if `cargo run` fails, message should include:

```text
If this looks like a setup issue:
    game-dev doctor --explain

If this looks like an asset/data issue:
    game-dev asset-check
    game-dev validate-data assets/game.ron

See:
    docs/tutorials/common-errors.md
```

For `game-dev package`, if release build fails, do the same.

### 8.5 Add tests for diagnostic substrings

Tests should check useful substrings, not exact long messages.

Add tests for:

```text
unknown prefab suggestion
unknown texture suggestion
unknown scene suggestion
bad map symbol row/col
countdown key typo
restart-required reload change
```

## Definition of done

```text
- Beginner-facing diagnostics consistently include context and fixes.
- Countdown key typo is specifically tested.
- CLI tells users which command to run next.
- Common errors docs mirror actual diagnostic wording.
```

Status note:

- status: `Done`
- files changed: `crates/game-kit/src/diagnostics.rs`, `crates/game-kit/src/beginner/rules.rs`, `crates/game-kit/src/data.rs`, `crates/game-cli/src/lib.rs`, `crates/game-core/tests/architecture_boundaries.rs`, `docs/tutorials/common-errors.md`, `CHANGELOG.md`, `plans/plan.md`
- implementation summary: Updated rule-combination diagnostics to use beginner builder method names, added shared CLI failure advice for `game-dev run`, `game-dev check`, and generated-project package build failures, mirrored that wording in common-errors docs, and added an explicit row/column bad text-map symbol validation test. Existing data tests already cover unknown prefab/texture/scene suggestions, countdown key typo suggestions, and restart-required reload diagnostics.
- validation commands run: `cargo fmt --all -- --check`; `cargo test -p game-kit --locked validation_`; `cargo test -p game-kit --locked rules_builder_reports_missing_rule_dependencies`; `cargo test -p game-cli --locked`; `cargo test -p game-core --test architecture_boundaries --locked`
- validation result: passed.
- remaining caveats: full workspace clippy/test coverage also ran successfully inside Phase 7's `cargo xtask release-check --skip-smoke --features ci-build-sdl3` before these final diagnostic wording changes; rerun the full release-check gate in Phase 12.

---

# Phase 9 — Add a Tiled data-driven path if needed

## Objective

Decide whether Tiled should be supported only through Rust beginner API or also through `assets/game.ron`.

## Current state

`BeginnerMapFile` already includes:

```rust
Tiled(TiledMapFile)
```

So the data format appears to support Tiled maps.

## Steps

### 9.1 Add Tiled to data-driven example or create separate one

Option A: Extend `examples/data-driven-full-demo/assets/game.ron` with one Tiled map.

Option B: Create:

```text
examples/data-driven-tiled-demo
```

Recommended: Option B, to keep data-driven-full-demo from becoming too huge.

### 9.2 Add `assets/game.ron` example

Example shape:

```ron
maps: [
    Tiled((
        name: "level_1",
        path: "maps/tiled_demo.tmx",
        start: true,
        floor: "floor",
        wall: "wall",
        objects: {
            "Player": "player",
            "Slime": "slime",
        },
    )),
]
```

Adjust field names to match `TiledMapFile`.

### 9.3 Add docs

In `docs/cookbook/tiled.md`, add a “No-Rust data file” section.

### 9.4 Add validation test

Ensure bad object mappings in data-driven Tiled maps produce helpful messages.

## Definition of done

```text
- Tiled has at least one Rust beginner demo.
- Optionally, Tiled also has one data-driven demo.
- Tiled docs show both Rust and no-Rust paths if both are supported.
```

Status note:

- status: `Done`
- files changed: `Cargo.toml`, `Cargo.lock`, `examples/data-driven-tiled-demo/Cargo.toml`, `examples/data-driven-tiled-demo/build.rs`, `examples/data-driven-tiled-demo/src/main.rs`, `examples/data-driven-tiled-demo/README.md`, `examples/data-driven-tiled-demo/assets/.gitignore`, `examples/data-driven-tiled-demo/assets/game.ron`, `examples/data-driven-tiled-demo/assets/maps/tiled_demo.tmx`, `.github/workflows/ci.yml`, `crates/game-kit/src/data.rs`, `crates/game-core/tests/architecture_boundaries.rs`, `docs/cookbook/tiled.md`, `docs/tutorials/13-data-driven-demo.md`, `README.md`, `CHANGELOG.md`, `plans/plan.md`
- implementation summary: Added `examples/data-driven-tiled-demo`, a no-Rust `assets/game.ron` Tiled TMX example that maps Tiled object identifiers to beginner prefabs. Added data validation coverage for unknown Tiled object prefab mappings, improved map reference owner wording to say `map 'name'`, documented the no-Rust Tiled shape in the cookbook/tutorial/README, and added CI Xvfb smoke coverage.
- validation commands run: `cargo fmt --all -- --check`; `cargo check -p data-driven-tiled-demo --locked`; `GAME_ASSET_DIR=examples/data-driven-tiled-demo/assets cargo run -p game-cli -- validate-data game.ron`; `cargo test -p game-kit --locked validation_names_unknown_tiled_object_prefabs`; `cargo test -p game-core --test architecture_boundaries --locked`; `cargo check -p data-driven-tiled-demo --locked --features ci-build-sdl3`; `GAME_SMOKE_FRAMES=60 cargo run -p data-driven-tiled-demo --locked`; `env -u MESA_LOADER_DRIVER_OVERRIDE VK_LOADER_DRIVERS_SELECT='*lvp*' DISPLAY=:99 SDL_VIDEODRIVER=x11 GAME_SMOKE_FRAMES=60 cargo run -p data-driven-tiled-demo --locked --features ci-build-sdl3`
- validation result: Cargo check, data validation, targeted data test, architecture-boundary tests, CI-feature check, and the data-driven Tiled graphical smoke passed through Xvfb/lavapipe. A renderer/platform drop-order bug found during this smoke was fixed in Phase 12.
- remaining caveats: CI smoke coverage was added but not executed locally; the equivalent local Xvfb/lavapipe smoke path passed.

---

# Phase 10 — Clarify distribution policy

## Objective

Explain what is supported now and what is future.

## Current state

Templates use tag-pinned Git dependencies:

```toml
game-starter = { git = "https://github.com/P2949/game", tag = "v0.1.0", package = "game-starter" }
```

Release workflow creates Linux/Windows demo zips.

This is good for a prototype/release-candidate, but not as polished as crates.io plus a dedicated template repo.

## Relevant files

```text
README.md
docs/distribution-policy.md
CHANGELOG.md
docs/migrations/*
templates/*/cargo-generate.toml
.github/workflows/release.yml
```

## Steps

### 10.1 Create or update distribution policy

Create:

```text
docs/distribution-policy.md
```

Say:

```markdown
## Current distribution model

Generated projects use tag-pinned Git dependencies.

## Why

The API is still young. Git tags give reproducibility without crates.io release overhead.

## Future

- dedicated `game-template` repository,
- crates.io packages after API stabilizes,
- versioned docs per release.
```

### 10.2 Update README

Add:

```text
Generated projects are pinned to release tags.
For development against the local checkout, use `cargo xtask new-demo`.
```

### 10.3 Update release checklist

Before a release:

```text
- update template dependency tag,
- update CHANGELOG,
- update migration docs,
- run generated-template CI,
- run first-15-minutes script,
- check release artifacts.
```

### 10.4 Add future issue list

If using GitHub Issues, document future tasks:

```text
publish crates.io packages
split template repo
version docs
installer for game-dev
```

## Definition of done

```text
- Distribution strategy is explicit.
- Users know generated projects depend on Git tags.
- Crates.io/template repo work is labeled future, not missing architecture.
```

Phase 10 status note:

- status: Done.
- files changed: `docs/distribution-policy.md`, `README.md`,
  `docs/release-checklist.md`, `CHANGELOG.md`,
  `crates/game-core/tests/architecture_boundaries.rs`.
- implementation summary: Added an explicit distribution policy covering the
  current release-candidate git-revision pin, the tagged-release dependency
  policy, local-checkout `cargo xtask new-demo`, prebuilt demo artifacts, and
  future crates.io/template-repo/docs/installer tasks. README and release
  checklist now link or mirror the policy, and an architecture-boundary test
  protects the wording.
- validation commands run: `cargo fmt --all -- --check`;
  `cargo test -p game-core --test architecture_boundaries --locked distribution_policy_keeps_release_candidate_model_explicit`;
  `rg -n "distribution-policy|Distribution Policy|release-candidate templates pin|Generated projects are pinned|crates\\.io|game-template" README.md docs/distribution-policy.md docs/release-checklist.md CHANGELOG.md crates/game-core/tests/architecture_boundaries.rs`.
- validation result: Formatting passed, the focused architecture test passed,
  and search confirmed the policy/link/future-work wording is present.
- remaining caveats: Templates currently pin a release-candidate git revision
  because no release tag has been published yet; the policy documents that
  generated templates should move to tag-pinned dependencies for a tagged
  release.

---

# Phase 11 — Keep beginner tutorial path narrow

## Objective

Prevent the beginner layer from becoming overwhelming.

## Current state

Tutorial numbering is now mostly clean:

```text
00-start-here.md
01-run-the-demo.md
02-your-first-player.md
03-add-a-map.md
04-add-an-enemy.md
05-add-pickups-and-score.md
06-add-projectiles.md
07-add-doors-and-levels.md
08-add-sound-and-music.md
09-add-ui-and-menu.md
10-package-your-demo.md
11-custom-behavior.md
12-fast-iteration.md
13-data-driven-demo.md
```

There are also optional older files.

## Relevant files

```text
README.md
docs/tutorials/README.md
docs/tutorials/*
docs/cookbook/*
docs/advanced-content-authoring.md
docs/when-to-use-advanced-api.md
examples/*
templates/*
```

## Steps

### 11.1 Make three tracks explicit

In README and tutorial index:

```text
Track A: No Rust
  Use templates/data-driven-demo.
  Edit assets/game.ron.

Track B: Beginner Rust
  Use templates/simple-demo.
  Follow tutorials 00–12.

Track C: Advanced
  Use game_kit::advanced::prelude only when beginner APIs are insufficient.
```

### 11.2 Move optional files out of the main sequence

Either keep names like:

```text
optional-add-animation.md
```

or move them into:

```text
docs/cookbook/
```

Do not let optional files appear as duplicate sequence steps.

### 11.3 Add “which example should I copy?”

In README:

```text
No Rust: templates/data-driven-demo
First Rust project: templates/simple-demo
One-file example: examples/one-file-demo
Script-like custom behavior: examples/script-like-custom-rules
Events: examples/events-demo
Tiled: examples/tiled-demo
Advanced lab: crates/testbed-content — do not copy first
```

### 11.4 Add architecture docs test

Architecture test should confirm:

```text
README mentions Track A, Track B, Track C.
README warns not to start with testbed-content.
Beginner tutorial index points to templates.
```

## Definition of done

```text
- New users see one obvious start path.
- Advanced content is clearly a later path.
- Tiled example is included after Phase 3.
- Optional docs do not clutter the core path.
```

Phase 11 status note:

- status: Done.
- files changed: `README.md`, `docs/tutorials/README.md`, `CHANGELOG.md`,
  `crates/game-core/tests/architecture_boundaries.rs`.
- implementation summary: README and the tutorial index now use the same three
  entry tracks: Track A for no-Rust `templates/data-driven-demo` and
  `assets/game.ron`, Track B for beginner Rust `templates/simple-demo` and
  tutorials 00-12, and Track C for advanced API work only when beginner APIs
  are insufficient. The copy-first list now explicitly names the no-Rust
  template, first Rust template, one-file example, script-like custom behavior,
  events demo, Tiled demo, and `crates/testbed-content` as an advanced lab not
  to copy first. Optional tutorial pages remain `optional-*` follow-ups outside
  the numbered course.
- validation commands run: `cargo fmt --all -- --check`;
  `cargo test -p game-core --test architecture_boundaries --locked beginner_entry_docs_keep_three_tracks_and_copy_list_clear`;
  `cargo test -p game-core --test architecture_boundaries --locked tutorial_numbered_chapters_have_unique_prefixes`;
  `rg -n "Track A: No Rust|Track B: Beginner Rust|Track C: Advanced|Track D:|tutorials 00-12|Advanced lab|events-demo|tiled-demo|optional-" README.md docs/tutorials/README.md docs/tutorials`.
- validation result: Formatting passed, the new three-track architecture test
  passed, the numbered tutorial uniqueness test passed, and search confirmed
  Track A/B/C wording, no Track D entry, Tiled/events examples, advanced-lab
  warning, and optional follow-up naming.
- remaining caveats: None.

---

# Phase 12 — Final release gate

## Objective

Declare the objective fully achieved only after the actual release gate passes.

## Run all checks

```bash
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build -p game --release --locked
```

## Run smoke checks

```bash
GAME_SMOKE_FRAMES=120 cargo run -p game --locked
GAME_DEMO=simple GAME_SMOKE_FRAMES=120 cargo run -p game --locked
GAME_DEMO=testbed GAME_SMOKE_FRAMES=120 cargo run -p game --locked
GAME_ASSET_DIR=assets GAME_SMOKE_FRAMES=120 cargo run -p game --release --locked
```

After adding Tiled demo:

```bash
GAME_SMOKE_FRAMES=60 cargo run -p tiled-demo --locked
```

## Run CLI checks

```bash
cargo run -p game-cli -- doctor --explain
cargo run -p game-cli -- asset-check
cargo run -p game-cli -- validate-data assets/game.ron
cargo run -p game-cli -- check
```

Note: `game-dev check` must be implemented in Phase 6 first.

## Run generated-project checks

```bash
rm -rf /tmp/game-release-check
mkdir -p /tmp/game-release-check

cargo run -p game-cli -- new /tmp/game-release-check/simple --template simple
cargo check --manifest-path /tmp/game-release-check/simple/Cargo.toml
GAME_SMOKE_FRAMES=60 cargo run --manifest-path /tmp/game-release-check/simple/Cargo.toml

cargo run -p game-cli -- new /tmp/game-release-check/data --template data-driven
cargo check --manifest-path /tmp/game-release-check/data/Cargo.toml
GAME_SMOKE_FRAMES=60 cargo run --manifest-path /tmp/game-release-check/data/Cargo.toml
```

## Run package checks

```bash
cd /tmp/game-release-check/simple
game-dev package --release --out dist/simple-demo --zip

cd /tmp/game-release-check/data
game-dev package --release --out dist/data-demo --zip
```

## Run release-check

After Phase 7:

```bash
cargo xtask release-check
```

or:

```bash
cargo xtask release-check --skip-smoke
```

## Verify release artifacts

Run the release workflow manually or on a test tag:

```text
.github/workflows/release.yml
```

Confirm artifacts:

```text
game-demo-linux-x86_64.zip
game-demo-windows-x86_64.zip
```

## Definition of done

```text
- Full workspace verification passes.
- Generated project verification passes.
- Package verification passes.
- Tiled demo smoke test passes.
- Countdown validation tests pass.
- Roadmap/status docs are accurate.
- Release artifacts are produced.
- Remaining future work is explicitly post-1.0.
```

Phase 12 status note:

- status: Partial.
- files changed: `README.md`, `docs/tutorials/README.md`,
  `docs/beginner-productization-roadmap.md`, `docs/release-checklist.md`,
  `CHANGELOG.md`, `.github/workflows/release.yml`,
  `scripts/verify-release-artifact.sh`,
  `crates/game-core/tests/architecture_boundaries.rs`,
  `crates/game-runtime/src/runner.rs`,
  `crates/game-runtime/tests/headless_runner.rs`, `plans/plan.md`.
- implementation summary: Ran the final release gate after fixing a
  beginner-doc boundary regression introduced during Phase 11: README now keeps
  the first track list beginner-safe, and the literal
  `game_kit::advanced::prelude::*` import is documented under the later
  `### Advanced API` boundary. Fixed runtime teardown so the renderer drops
  before the platform/window owner, preventing Vulkan swapchain destruction from
  touching a torn-down native display. The unified full release gate passes
  through Xvfb/lavapipe, generated simple/data projects check and package
  outside the workspace, root `game-dev check` passes, countdown validation
  tests pass, data-driven Tiled smoke passes, and a local Linux prebuilt demo
  zip dry-run was produced and verified. Release workflow archives are now
  checked for the executable, SDL runtime library, launchers, README, and core
  assets before upload/release attachment.
- validation commands run: `cargo xtask release-check --skip-smoke --features ci-build-sdl3`;
  `env -u MESA_LOADER_DRIVER_OVERRIDE VK_LOADER_DRIVERS_SELECT='*lvp*' DISPLAY=:99 SDL_VIDEODRIVER=x11 vulkaninfo --summary`;
  `env -u MESA_LOADER_DRIVER_OVERRIDE VK_LOADER_DRIVERS_SELECT='*lvp*' DISPLAY=:99 SDL_VIDEODRIVER=x11 cargo xtask release-check --features ci-build-sdl3`;
  `cargo test -p game-runtime --test headless_runner --locked`;
  `GAME_SMOKE_FRAMES=120 cargo run -p game --locked`;
  `GAME_DEMO=simple GAME_SMOKE_FRAMES=120 cargo run -p game --locked`;
  `GAME_DEMO=testbed GAME_SMOKE_FRAMES=120 cargo run -p game --locked`;
  `GAME_ASSET_DIR=assets GAME_SMOKE_FRAMES=120 cargo run -p game --release --locked`;
  `GAME_SMOKE_FRAMES=60 cargo run -p tiled-demo --locked`;
  `env -u MESA_LOADER_DRIVER_OVERRIDE VK_LOADER_DRIVERS_SELECT='*lvp*' DISPLAY=:99 SDL_VIDEODRIVER=x11 GAME_SMOKE_FRAMES=60 cargo run --manifest-path /tmp/game-release-check/generated/simple/Cargo.toml --features ci-build-sdl3`;
  `env -u MESA_LOADER_DRIVER_OVERRIDE VK_LOADER_DRIVERS_SELECT='*lvp*' DISPLAY=:99 SDL_VIDEODRIVER=x11 GAME_SMOKE_FRAMES=60 cargo run --manifest-path /tmp/game-release-check/generated/data/Cargo.toml --features ci-build-sdl3`;
  `env -u MESA_LOADER_DRIVER_OVERRIDE VK_LOADER_DRIVERS_SELECT='*lvp*' DISPLAY=:99 SDL_VIDEODRIVER=x11 GAME_SMOKE_FRAMES=60 cargo run -p data-driven-tiled-demo --locked --features ci-build-sdl3`;
  `cargo run -p game-cli -- check`;
  `cargo test -p game-kit --locked countdown`;
  `rm -rf /tmp/game-demo-linux-x86_64 /tmp/game-demo-linux-x86_64.zip && cargo xtask package-demo --release --features ci-build-sdl3 --out /tmp/game-demo-linux-x86_64`;
  `cd /tmp/game-demo-linux-x86_64 && zip -r ../game-demo-linux-x86_64.zip .`;
  `scripts/verify-release-artifact.sh /tmp/game-demo-linux-x86_64.zip linux`;
  `ls -lh /tmp/game-demo-linux-x86_64.zip /tmp/game-release-check/generated/simple/dist/simple-release-check.zip /tmp/game-release-check/generated/data/dist/data-driven-release-check.zip`;
  `gh workflow list --repo P2949/game`;
  `gh workflow view "Release Artifacts" --repo P2949/game`.
- validation result: The full Xvfb/lavapipe release gate passed: formatting,
  workspace tests, headless runtime tests, clippy, release build, CLI
  doctor/asset/data checks, generated-project `cargo check`,
  generated-project `game-dev check`, generated simple/data release package
  zips, default/simple/testbed/release graphical smoke, and Tiled graphical
  smoke all completed successfully. The focused runtime drop-order regression
  test passed. Root `game-dev check` passed. Focused countdown tests passed.
  Data-driven Tiled smoke passed through the same Xvfb/lavapipe path. Local
  Linux demo artifact dry-run produced `/tmp/game-demo-linux-x86_64.zip`
  (4.1M), `scripts/verify-release-artifact.sh` confirmed the expected Linux
  package layout, and generated package zips exist at
  `/tmp/game-release-check/generated/simple/dist/simple-release-check.zip` and
  `/tmp/game-release-check/generated/data/dist/data-driven-release-check.zip`
  (4.3M each).
- remaining caveats: The GitHub release workflow was not run from this
  environment because the current changes are local and unpushed; triggering
  `workflow_dispatch` on `master` would test the remote branch rather than this
  worktree and would mutate remote Actions state. `gh workflow view` confirms
  the `Release Artifacts` workflow is active but has zero runs. The Windows
  prebuilt artifact and GitHub Release attachment still need a workflow run
  from a pushed commit or test tag.

Phase 12 follow-up verification note:

- status: Still Partial.
- review summary: Re-read Phase 12 after the full local implementation pass.
  The release workflow already verifies Linux and Windows archives before
  upload/attachment, and architecture tests already protect that workflow
  wiring. No additional local code changes are needed for artifact safety.
- validation commands run: `cargo xtask release-check --skip-smoke --features ci-build-sdl3`;
  `gh workflow list --repo P2949/game`;
  `gh workflow view "Release Artifacts" --repo P2949/game`.
- validation result: The fresh non-graphical contributor release gate passed:
  formatting, workspace tests, headless runtime tests, clippy, release build,
  CLI doctor/asset/data checks, generated simple/data project `cargo check`,
  generated `game-dev check`, and generated release package zips all completed.
  GitHub still reports the `Release Artifacts` workflow as active with zero
  runs.
- remaining caveats: The only unfinished Phase 12 item is external artifact
  publication from `.github/workflows/release.yml`. Completing that requires a
  pushed commit or test tag so GitHub Actions can build and upload the Linux and
  Windows zips from the final code.

Phase 12 artifact-verification helper note:

- status: Done.
- files changed: `scripts/verify-github-release-artifacts.sh`,
  `docs/release-checklist.md`,
  `crates/game-core/tests/architecture_boundaries.rs`, `CHANGELOG.md`,
  `plans/plan.md`.
- implementation summary: Added a post-workflow verifier that downloads the
  Linux and Windows `Release Artifacts` workflow artifacts with `gh run
  download`, then runs the existing package-layout verifier against both zips.
  The release checklist now documents how to list workflow runs and verify a
  specific run id, or the latest successful `release.yml` run. Architecture
  tests protect the helper and checklist wording.
- validation commands run: `bash -n scripts/verify-github-release-artifacts.sh scripts/verify-release-artifact.sh`;
  `scripts/verify-github-release-artifacts.sh --help`;
  `cargo test -p game-core --test architecture_boundaries --locked release_workflow_publishes_prebuilt_demo_artifacts`.
- validation result: Shell syntax passed, help output describes the helper and
  environment variables, and the focused release-workflow architecture test
  passed.
- remaining caveats: The helper can verify GitHub artifacts only after a
  successful `release.yml` run exists. GitHub still reports zero `Release
  Artifacts` runs for `P2949/game`, so artifact publication itself remains the
  final external gate.

Phase 12 release-workflow dispatch note:

- status: Fix implemented; remote verification rerun required.
- files changed: `xtask/Cargo.toml`,
  `crates/game-core/tests/architecture_boundaries.rs`, `CHANGELOG.md`,
  `plans/plan.md`.
- implementation summary: Pushed branch
  `codex/beginner-release-polish-artifacts` and dispatched
  `.github/workflows/release.yml` as workflow run `28397228106`. Both Linux and
  Windows jobs failed in `Package demo` because `cargo xtask package-demo
  --features ci-build-sdl3` enabled the feature on `xtask` without forwarding it
  to `game-cli`, so the `xtask` binary still linked against system SDL3. Added
  `xtask` feature forwarding with `ci-build-sdl3 = ["game-cli/ci-build-sdl3"]`
  and extended the release workflow architecture test to protect that wiring.
- validation commands run: `gh workflow run release.yml --repo P2949/game --ref codex/beginner-release-polish-artifacts`;
  `gh run watch 28397228106 --repo P2949/game --exit-status`;
  `gh run view 28397228106 --repo P2949/game --log-failed`;
  `cargo check -p xtask --features ci-build-sdl3 --locked`;
  `cargo test -p game-core --test architecture_boundaries --locked release_workflow_publishes_prebuilt_demo_artifacts`;
  `cargo fmt --all -- --check`.
- validation result: The first remote release workflow run failed before
  artifact upload, and the logs identified missing `SDL3`/`SDL3.lib` while
  linking `xtask`. The local feature-forwarding fix now passes focused cargo
  check, focused architecture test, and formatting.
- remaining caveats: Push the feature-forwarding fix and rerun
  `release.yml`; artifacts are not verified until a successful remote run
  uploads both zips and `scripts/verify-github-release-artifacts.sh` passes.

Final plan summary:

- phase statuses: Phase 0 Done; Phase 1 Done; Phase 2 Done; Phase 3 Done;
  Phase 4 Done; Phase 5 Done; Phase 6 Done; Phase 7 Done; Phase 8 Done; Phase
  9 Done; Phase 10 Done; Phase 11 Done; Phase 12 Partial.
- major changes implemented: roadmap/status docs updated; countdown custom-rule
  validation fixed and tested; missing countdown runtime keys no longer expire
  immediately; beginner Tiled Rust and data-driven demos added; compatibility
  prelude deprecation protected; partial `game.ron` reload policy documented
  and tested; `game-dev check` added; `cargo xtask release-check` added;
  beginner diagnostics and CLI failure advice improved; GitHub release-artifact
  download/verification helper added; distribution policy documented; tutorial
  entry paths narrowed to no-Rust, beginner Rust, and advanced tracks.
- successful verification: `cargo xtask release-check --features ci-build-sdl3`
  under Xvfb/lavapipe; root `game-dev check`; focused countdown tests;
  generated simple/data project checks, smoke commands, and package zips;
  data-driven Tiled smoke; local Linux prebuilt demo package dry-run; focused
  release-artifact helper syntax and architecture tests.
- known external blockers: GitHub release artifacts need
  `.github/workflows/release.yml` to pass from a pushed commit or test tag. The
  first branch workflow run failed before artifact upload and the local fix has
  been implemented; a rerun is still required.
- remaining future work: crates.io publication, a dedicated template repo,
  versioned docs, and a `game-dev` installer remain future distribution tasks
  rather than missing 1.0 architecture.
- Beginner Productization 1.0 completion: Not yet complete. The implementation
  and full local release gate are in place, but the milestone should not be
  marked complete until release artifacts are produced by the GitHub release
  workflow from the final pushed commit or a test tag.

---

# Final acceptance criteria

The objective is fully achieved when all of this is true:

```text
1. Engine/runtime/backend code is separate from content authoring.
2. Beginner code uses `game_starter::prelude::*` or `game_kit::beginner::prelude::*`.
3. Advanced code uses `game_kit::advanced::prelude::*`.
4. Beginner callbacks receive beginner-only `Game` wrappers, not `GameCtx`.
5. Beginner examples/templates/docs do not expose ECS traversal.
6. No-Rust users can make a small demo through `assets/game.ron`.
7. Script-like hooks and structured rules cover common small-game behavior.
8. Countdown custom rules validate their data keys correctly.
9. Tiled has a runnable beginner demo, not just importer tests.
10. Generated projects work outside the workspace.
11. `game-dev` supports new, doctor, run, package, asset-check, validate-data, and check.
12. Packaging creates a shareable folder and zip.
13. Diagnostics explain beginner mistakes with valid options and fixes.
14. Docs clearly separate no-Rust, beginner Rust, and advanced paths.
15. Roadmaps/status docs accurately say the architecture is complete.
16. Full local verification passes.
```

---

# Recommended commit order

Use small commits in this order:

```text
1. Run full local verification and fix any compile/test/clippy failures.
2. Update docs/beginner-productization-roadmap.md with accurate phase statuses.
3. Add historical/current status banners to roadmap docs.
4. Fix countdown custom-rule validation for tag/data key matching.
5. Make tick_countdown ignore missing keys instead of defaulting to 0.0.
6. Add countdown validation tests and common-errors docs.
7. Add examples/tiled-demo as a beginner `game_starter::prelude::*` demo.
8. Add Tiled demo to workspace, docs, README, CI smoke, and architecture tests.
9. Preserve/verify real `#[deprecated]` compatibility prelude behavior.
10. Document partial game.ron reload policy and add reload-policy tests.
11. Add `game-dev check`.
12. Add `cargo xtask release-check`.
13. Improve rule/data diagnostics and CLI failure advice.
14. Clarify tutorial tracks and example selection.
15. Add/update distribution-policy.md.
16. Run generated project/package checks.
17. Run release workflow or test tag.
18. Mark Beginner Productization 1.0 as complete if all gates pass.
```

# Bottom line

The project no longer needs foundational architecture work. It needs **three concrete implementation fixes** and **release polish**:

```text
- fix countdown key validation,
- add a real Tiled beginner demo,
- add unified check/release-check commands and update docs/status.
```

After that, assuming the full Rust verification passes locally, the goal can honestly be marked fully achieved.

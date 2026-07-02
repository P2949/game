# Primary No-Rust Authoring Foundation Roadmap

## Status

**Planned roadmap based on static inspection of the uploaded `game-master(9).zip`.**

## Implementation progress

- [x] 2026-07-02: Phase 0.1 complete. Created branch `architecture/primary-no-rust-authoring-foundation` from a clean working tree.
- [x] 2026-07-02: Phase 0.2 complete. Baseline checks passed: `cargo fmt --all -- --check`, `cargo test --workspace --locked --features game/ci-build-sdl3`, `cargo test -p game-runtime --test headless_runner --no-default-features --locked`, `cargo clippy --workspace --all-targets --locked --features game/ci-build-sdl3 -- -D warnings`, `cargo run -p game-cli --features ci-build-sdl3 -- doctor --explain`, `cargo run -p game-cli --features ci-build-sdl3 -- validate-data assets/game.ron`, and `cargo run -p game-cli --features ci-build-sdl3 -- asset-check`.
- [x] 2026-07-02: Phase 0.3 complete. Confirmed the current no-Rust gaps: `templates/data-driven-demo` contains `Cargo.toml`, `build.rs`, `src/main.rs`, `cargo-generate.toml`, and `assets/game.ron`; docs and CLI help still point at `assets/game.ron`; `game-kit` parses via RON-shaped schema/tests; examples use Rust wrappers that call `load_beginner_file(.../assets/game.ron)`; animation metadata and asset validation still treat `.ron` as current data.
- [x] 2026-07-02: Phase 0.4 complete. Added the roadmap as `docs/roadmaps/primary-no-rust-authoring-foundation.md` and indexed it in `docs/roadmaps/README.md`.
- [x] 2026-07-02: Phase 1.1 complete. Rewrote `docs/api-boundary.md` around primary no-Rust authoring, secondary beginner Rust authoring, and advanced Rust authoring; it now explicitly states that the Rust builder API is secondary and the primary surface is a plain data/config package run through a prebuilt player.
- [x] 2026-07-02: Phase 1.2 complete. Public entry-point docs now describe `game.toml`/prebuilt-player authoring as the primary no-Rust target and describe `assets/game.ron` as legacy/transitional compatibility rather than the no-Rust destination.
- [x] 2026-07-02: Phase 1.3 complete. Added explicit "not complete yet" status language for the stricter primary no-Rust SDK in the API boundary, README/tutorial entry points, distribution policy, and historical productization roadmap.
- [x] 2026-07-02: Phase 1.4 complete. Added explicit "Secondary Rust Authoring Path" labels to `docs/beginner-authoring.md` and `docs/content-authoring.md` while keeping the useful Rust API examples.
- [x] 2026-07-02: Phase 1.5 complete. Added primary no-Rust release gates to `docs/release-checklist.md` for Rust-free templates/packages, Rust-vocabulary scanning, docs defaults, `game.toml` runner loading, and packaged artifact verification.
- [x] 2026-07-02: Phase 1 validation complete. The broad `rg "No-Rust|no-Rust|no-rust|game\\.ron|RON" README.md docs templates` scan still reports expected legacy, migration, advanced, historical, and active-roadmap mentions; targeted stale-phrase checks only hit the roadmap's own examples of wording to replace.
- [x] 2026-07-02: Phase 2.1 complete. Added `docs/no-rust-package-layout.md` documenting root `game.toml`, conventional `assets/` folders, primary animation `*.toml` metadata, no Rust project files, and the target `game-dev check`, `game-dev preview`, and `game-dev package` workflow.
- [x] 2026-07-02: Phase 2.2 complete. Added `game_cli::project::ProjectKind` detection for no-Rust packages, Rust starter projects with `[package.metadata.game]`, and engine workspace demos, with focused CLI unit tests.
- [x] 2026-07-02: Phase 2.3 complete. Added `NoRustProjectPaths`, `NoRustPathOverrides`, default root/game/assets resolution, explicit project/file/assets override handling, and a `GAME_ASSET_DIR` compatibility resolver in `game_cli::project`.
- [x] 2026-07-02: Phase 2.4 complete. Documented primary package metadata in `game.toml` with a `[game]` table and clarified that `[package.metadata.game]` is only for secondary Rust starter compatibility.
- [x] 2026-07-02: Phase 2 validation complete. `cargo test -p game-cli --locked` passed with unit coverage for no-Rust detection, root `game.toml` resolution, asset-dir resolution, explicit project/file/assets overrides, and no-Cargo packages not being treated as Rust starters.
- [x] 2026-07-02: Phase 3.1 complete. Added `toml` as a workspace dependency, wired `toml.workspace = true` into `game-kit` and `game-cli`, and verified with `cargo check -p game-kit -p game-cli --locked`.
- [x] 2026-07-02: Phase 3.2 complete. Added `crates/game-kit/src/data/model.rs` with a format-neutral top-level `AuthoringGameFile`; legacy RON parsing now translates into that model, and build/reload/validation paths consume it while existing RON schema/tests remain compatible.
- [x] 2026-07-02: Phase 3.3 complete. Moved the old RON-shaped schema module from `schema.rs` to `legacy_ron.rs`, re-exported it for compatibility, and verified legacy data coverage with `cargo test -p game-kit data --locked`.
- [x] 2026-07-02: Phase 3.4 complete. Added primary `game.toml` schema/parser modules with kebab-case TOML authoring shapes for prefabs, maps, rules, controls, audio, actions, and custom countdown rules; TOML source `version = 2` is accepted at the authoring edge and normalized into the current runtime model, and `cargo test -p game-kit toml_authoring --locked` passed.
- [x] 2026-07-02: Phase 3.5 complete. Added `AuthoringFormat` extension detection for primary `.toml` and legacy `.ron`, routed format-aware parsing through the data reader, added primary-help diagnostics for missing/unknown extensions, and verified with `cargo test -p game-kit data --locked`.
- [x] 2026-07-02: Phase 3.6 complete. Added public `game_kit::data::load_authoring_file` and `validate_authoring_file`, added `GameApp::load_authoring_file`, kept `load_beginner_game_file`/`validate_beginner_game_file` as compatibility wrappers, resolved root `game.toml` packages against sibling `assets/`, documented the primary/legacy split, and verified with `cargo test -p game-kit data --locked`.
- [x] 2026-07-02: Phase 3.7 complete. Added primary TOML diagnostics that describe unreadable files as game config, point to `controls.preset` and `rules.enabled`, preflight unknown `kind`/`action` values with known-value lists and suggestions, and verified with `cargo test -p game-kit toml --locked` plus `cargo test -p game-kit data --locked`.
- [x] 2026-07-02: Phase 3.8 complete. Split the monolithic data tests into `crates/game-kit/src/data/tests/ron_legacy.rs` and `toml_primary.rs` under a test module directory, keeping existing RON coverage labeled as legacy and TOML package acceptance separate; `cargo test -p game-kit data --locked` passed.
- [x] 2026-07-02: Phase 3 validation complete. Added `examples/data-driven-full-demo/game.toml` as a full primary TOML fixture, validated minimal and full TOML packages through `validate_authoring_file`, checked lower-kebab primary spellings and unknown-kind suggestions, added an architecture scan that fails on `Some(`, `Player((`, and related RON/Rust-shaped syntax in primary TOML, confirmed legacy `.ron` still validates, and ran `cargo test -p game-kit data --locked` plus `cargo test -p game-core architecture --locked`.
- [x] 2026-07-02: Phase 4.1 complete. Added `AuthoringLoadContext` with `project_root`, `asset_root`, and `source_file`, and routed primary TOML loads through package-root/sibling-assets context while preserving legacy RON context.
- [x] 2026-07-02: Phase 4.2 complete. Primary TOML file reads now resolve the requested config directly and report `could not read game config ... looked for ...`; legacy RON read errors remain explicitly labeled as legacy asset-root compatibility.
- [x] 2026-07-02: Phase 4.3 complete. Kept `GAME_ASSET_DIR` compatibility for legacy RON through the existing asset-root resolver and added an isolated integration test that validates `game.ron` from a configured asset directory.
- [x] 2026-07-02: Phase 4.4 complete. Primary TOML asset references use the beginner `[assets] textures/sounds/music/animation_sheets` spelling and resolve conventional files from `asset_root` (`assets/textures/<name>.png`, `assets/sounds/<name>.*`, etc.) without exposing asset handles.
- [x] 2026-07-02: Phase 4 validation complete. Covered root `game.toml` references to `assets/maps/...`, package validation from outside the project via absolute paths, CLI absolute project/file/assets override tests, and legacy `GAME_ASSET_DIR` validation; `cargo test -p game-kit data --locked` and `cargo test -p game-cli --locked` passed.
- [x] 2026-07-02: Phase 5.1 complete. Moved starter PNG/WAV generation into internal `game-cli` module `starter_assets`, allowing the primary no-Rust template to avoid a Rust `build.rs`.
- [x] 2026-07-02: Phase 5.2 complete. Added `DemoTemplate::NoRust`, `--template no-rust`, and aliases for secondary Rust/legacy templates while keeping the default on `simple` until the prebuilt player workflow is complete.
- [x] 2026-07-02: Phase 5.3 complete. Added `templates/no-rust-demo/game.toml` with a small readable primary config covering title, assets, controls, player, enemy, pickup, a text map, and common rules.
- [x] 2026-07-02: Phase 5.4 complete. Added `templates/no-rust-demo/README.txt` with text-editor/check/preview/F5/no-Rust instructions and no `cargo run` primary workflow.
- [x] 2026-07-02: Phase 5.5 complete. Kept `templates/simple-demo` and `templates/data-driven-demo` separate and relabeled them as secondary beginner Rust and legacy/transitional RON wrapper paths in template/public docs.
- [x] 2026-07-02: Phase 5 validation complete. Added architecture coverage that `templates/no-rust-demo` has no `.rs`, `Cargo.toml`, `build.rs`, or Rust/RON-shaped `game.toml` tokens; `game-dev new --template no-rust` generation now produces no Rust project files, writes starter assets, validates the generated `game.toml`, and `game-dev check` detects no-Rust packages and validates them without running Cargo. Verified with `cargo test -p game-cli --locked` and `cargo test -p game-core architecture --locked`.
- [x] 2026-07-02: Phase 6 binary complete. Added workspace package `bin/game-player` with `--project`, `--file`, `--assets`, `--smoke-frames`, `--help`, `GAME_PROJECT_DIR`, `GAME_FILE`, `GAME_ASSET_DIR`, and `GAME_SMOKE_FRAMES` support; it runs `game_runtime` with `game_kit::app::plugin_fn` and loads `game.toml` through the primary authoring loader without importing content crates.
- [x] 2026-07-02: Phase 6 metadata complete. `game-player` reads `[game]` metadata for title, `window_width`, `window_height`, and `sim_hz` and applies it to `RuntimeConfig` without exposing renderer/backend configuration.
- [x] 2026-07-02: Phase 6 validation complete. Added architecture coverage that `game-player` is in workspace/default builds, depends on runtime/game-kit, and avoids game-specific content crates; checked `game-player --help`, ran `cargo run -p game-player -- --project templates/no-rust-demo --smoke-frames 0`, and verified with `cargo test -p game-player --locked`, `cargo test -p game-cli --locked`, and `cargo test -p game-core architecture --locked`.
- [x] 2026-07-02: Phase 7.1 complete. `game-dev check` now detects `NoRustPackage` with the Phase 2 project-kind resolver, validates no-Rust assets plus `game.toml` without invoking Cargo, and keeps Rust starter/workspace projects on the existing Cargo-backed path; tests now prove the no-Rust path succeeds with a missing Cargo executable while the Rust template path invokes a supplied Cargo executable.
- [x] 2026-07-02: Phase 7.2 complete. `game-dev check`, `asset-check`, `validate-data`, `preview`, and no-Rust `package` now resolve project root, `game.toml`, and asset root through the shared no-Rust path resolver, including explicit `--project`/`--file`/`--assets` overrides where those commands support them and `GAME_ASSET_DIR` compatibility.
- [x] 2026-07-02: Phase 7.3 complete. `game-dev validate-data` defaults to root `game.toml`, supports project/file/assets overrides, validates TOML with the resolved asset root, and auto-detects legacy `.ron` files with a warning that primary no-Rust authoring should use `game.toml`.
- [x] 2026-07-02: Phase 7.4 complete. Added `game-dev preview`, which resolves `game-player` by explicit `--player`, sibling executable, `GAME_PLAYER`, or workspace developer fallback via `cargo run -p game-player -- ...`, then launches it with resolved `--project`, `--file`, `--assets`, and optional `--smoke-frames`.
- [x] 2026-07-02: Phase 7.5 complete. Added `game-dev preview --watch`, which polls `game.toml` and the resolved assets tree, debounces restarts, stops the current child process, and prints the changed path before relaunching the player.
- [x] 2026-07-02: Phase 7.6 complete. `game-dev package` now auto-detects no-Rust packages, validates `game.toml` and assets without building user Rust, copies `game-player`, copies the current `game-dev` when available, copies `game.toml` and `assets/`, ensures the bundled font for release layouts, writes launchers plus a no-Rust README, and supports optional zip output.
- [x] 2026-07-02: Phase 7.7 complete. `game-cli` now forwards `ogg` and `mp3` to `game-audio`, `game-player` forwards matching feature flags to `game-runtime`, and architecture tests pin those feature-forwarding contracts.
- [x] 2026-07-02: Phase 7.8 complete. Asset validation now supports explicit `[asset_check] ignore = [...]` rules in `game.toml`, including simple wildcard patterns, while still rejecting unknown files by default and pointing users to the ignore setting.
- [x] 2026-07-02: Phase 7 validation complete. Added focused CLI tests for no-Rust check without Cargo, Rust-template check invoking Cargo, asset-check default/project/assets overrides, validate-data defaulting to `game.toml`, no-Rust package contents, and asset ignore rules; verified codec feature diagnostics with `cargo test -p game-audio ogg_files_explain_how_to_enable_or_convert_when_feature_is_disabled --locked` and `cargo test -p game-audio mp3_files_explain_how_to_enable_or_convert_when_feature_is_disabled --locked`; ran `cargo test -p game-cli --locked`, `cargo test -p game-core --test architecture_templates --locked`, `cargo test -p game-core --test architecture_cli_release --locked`, `cargo build -p game-player --locked`, `cargo run -p game-cli -- check` from `templates/no-rust-demo`, `cargo run -p game-cli -- asset-check --project templates/no-rust-demo`, `cargo run -p game-cli -- validate-data --project templates/no-rust-demo`, `cargo run -p game-cli -- preview --project templates/no-rust-demo --smoke-frames 0`, no-Rust `game-dev package --out ...`, packaged `run.sh --smoke-frames 0`, and no-Rust `game-dev package --out ... --zip`.
- [x] 2026-07-02: Phase 8.1 complete. Debug overlay and F5 reload bookkeeping now use the actual loaded authoring filename (`game.toml`, `game.ron`, or fallback `game file`) for reload/error labels instead of hardcoded `game.ron` wording.
- [x] 2026-07-02: Phase 8.2 complete. Renamed the reload runtime resource and identity types internally to `AuthoringFileRuntime`, `AuthoringReloadIdentity`, `AuthoringReloadLevel`, and `RebuiltAuthoringContent`, while keeping hidden `Beginner*` aliases for compatibility.
- [x] 2026-07-02: Phase 8.3 complete. The Phase 7 `game-dev preview --watch` restart path is covered by a watch snapshot test that tracks both `game.toml` and asset-tree changes.
- [x] 2026-07-02: Phase 8.4 complete. Documented the no-build reload contract in the fast-iteration tutorial, no-Rust package layout, no-Rust template README, common-errors guide, and legacy data tutorial: F5 covers in-process value/map/asset reload where supported, `game-dev preview --watch` restarts the prebuilt player for structural primary no-Rust edits, and packaged players pick up edits on relaunch.
- [x] 2026-07-02: Phase 8.5 complete. Deferred full in-process structural authoring reload as planned; the MVP requirement is satisfied by the prebuilt watch restart path.
- [x] 2026-07-02: Phase 8 validation complete. Added a primary TOML F5 reload test asserting `game.toml reload: partial` and `last reload: game.toml ok (...)`; retained legacy RON overlay coverage; added watch snapshot coverage for `game.toml` and asset changes; verified with `cargo test -p game-kit f5_reload_reports_primary_game_toml_by_name --locked`, `cargo test -p game-kit data --locked`, `cargo test -p game-cli --locked`, `cargo test -p game-core --test architecture_docs --locked`, and `cargo test -p game-core --test architecture_templates --locked`.
- [x] 2026-07-02: Phase 9A.1 complete. Extended the existing structured `CommandErrors` diagnostic resource with generic `AuthoringSpawn` and `MapTransition` kinds instead of adding a parallel diagnostics stack.
- [x] 2026-07-02: Phase 9A.2 complete. `drain_beginner_spawn_queue` now records authoring-spawn failures in `CommandErrors` while still logging them, including missing content-runtime errors and failed deferred prefab spawns.
- [x] 2026-07-02: Phase 9A.3 complete. The debug overlay now labels `AuthoringSpawn` diagnostics as `authoring error: ...` and `MapTransition` diagnostics as `map transition error: ...`, while preserving the existing runtime-command label for core command failures.
- [x] 2026-07-02: Phase 9A.4 complete. Added tests for deferred beginner spawn failures being stored as structured command errors, authoring errors appearing in the debug overlay, and the test harness `assert_no_command_errors` assertion failing when stored diagnostics exist.
- [x] 2026-07-02: Phase 9B.1 complete. Map switching now preflights the target map before clearing the real world by resolving the target, validating prefab references, and spawning map objects into a scratch world.
- [x] 2026-07-02: Phase 9B.2 complete. Added a staged preflight spawn path for map objects so prefab-spawn and collider/transform validation failures are discovered before mutating the live world.
- [x] 2026-07-02: Phase 9B.3 complete. Map switching now commits in order: preflight, clear old queued commands, clear/spawn the real world, then update `ContentRuntime.current_map`; unknown maps and preflight failures reinsert the old runtime unchanged and store a map-transition diagnostic, and unexpected real-spawn failures attempt to restore the previous map.
- [x] 2026-07-02: Phase 9B.4 complete. Added rollback tests covering unknown maps, failed target-map spawns, structured diagnostics on failure, successful map switches, active-map command queuing, and clearing stale command-queue entries from the old world.
- [x] 2026-07-02: Phase 9 validation complete. Verified with `cargo test -p game-kit context --locked`, `cargo test -p game-kit map_flow --locked`, `cargo test -p game-runtime --locked`, `cargo test -p game-kit overlay_reports_authoring_spawn_errors --locked`, `cargo test -p game-kit authoring_errors --locked`, and `cargo test -p game-core --test architecture_beginner_surface game_kit_commands_do_not_expose_raw_map_change --locked`.
- [x] 2026-07-02: Phase 10.1 complete. Added checked-in no-Rust example packages `examples/no-rust-minimal`, `examples/no-rust-events`, `examples/no-rust-waves`, `examples/no-rust-projectiles`, `examples/no-rust-full`, and `examples/no-rust-tiled`; each has a root `game.toml`, `assets/`, and no Cargo/Rust wrapper files.
- [x] 2026-07-02: Phase 10.2 complete. Created no-Rust equivalents for the current data-driven Rust-wrapper examples: events, waves, projectiles, full, and Tiled; the old `examples/data-driven-*` READMEs now label those projects as legacy Rust-wrapper examples and point to the no-Rust equivalents.
- [x] 2026-07-02: Phase 10.3 complete. Updated primary entry docs so the first no-Rust path is `game-dev new --template no-rust`, `game-dev check`, and `game-dev preview`, and updated the README, tutorial index, authoring docs, cookbook, Tiled cookbook, and no-Rust package layout to list the new no-Rust examples as the primary package proof.
- [x] 2026-07-02: Phase 10.4 complete. Kept Rust examples and legacy RON wrappers available under secondary/legacy labels, including `templates/simple-demo`, `examples/one-file-demo`, `examples/tiled-demo`, and the legacy `examples/data-driven-*` wrappers.
- [x] 2026-07-02: Phase 10 CI coverage complete. Added a CI step that loops over every `examples/no-rust-*` package, runs `game-dev check --project`, and smoke-runs `game-player --project ... --smoke-frames 0` under Xvfb/software Vulkan; architecture tests pin that workflow coverage.
- [x] 2026-07-02: Phase 10 validation complete. Added architecture coverage that every `examples/no-rust-*` package contains `game.toml`, has `assets/`, and contains no `.rs`, `Cargo.toml`, `Cargo.lock`, `build.rs`, `src/main.rs`, or `assets/game.ron`; verified all examples with `cargo run -p game-cli -- check --project examples/no-rust-{minimal,events,waves,projectiles,full,tiled}`, smoke-ran all with `cargo run -p game-player -- --project examples/no-rust-{minimal,events,waves,projectiles,full,tiled} --smoke-frames 0`, and ran `cargo test -p game-cli --locked`, `cargo test -p game-core --test architecture_templates --locked`, `cargo test -p game-core --test architecture_docs --locked`, and `cargo test -p game-core --test architecture_cli_release --locked`.
- [x] 2026-07-02: Phase 11.5 complete. Added and wired `game-dev authoring-scan [--project dir]` so release packages and external no-Rust projects can run the primary-surface scan outside the Rust test harness; focused scanner tests and `cargo fmt --all` passed.
- [x] 2026-07-02: Phase 11.1 complete. Defined the primary no-Rust corpus in architecture-test support, covering `templates/no-rust-demo`, all checked-in `examples/no-rust-*` packages, and the primary entry docs.
- [x] 2026-07-02: Phase 11.2 complete. Added `architecture_no_rust_authoring.rs` coverage that every primary package has root `game.toml`, has `assets/`, and excludes `Cargo.toml`, `Cargo.lock`, `build.rs`, `src/main.rs`, `*.rs`, and `assets/game.ron`.
- [x] 2026-07-02: Phase 11.3 complete. Added architecture scans for Rust/RON-shaped tokens in primary TOML files and changed primary projectile TOML from `lifetime` to `duration`, with a parser regression test rejecting the old key.
- [x] 2026-07-02: Phase 11.4 complete. Added architecture and CLI scans for forbidden engine/Rust vocabulary in primary no-Rust text surfaces, including docs/readmes and primary TOML.
- [x] 2026-07-02: Phase 11.6 complete. Marked primary no-Rust doc sections with `<!-- primary-no-rust:start -->` / `<!-- primary-no-rust:end -->` and added tests requiring `game.toml`, `game-dev preview`, and `prebuilt executable` while rejecting current-default `assets/game.ron`, `cargo run`, and engine vocabulary in those sections.
- [x] 2026-07-02: Phase 11 validation complete. Verified with `cargo fmt --all -- --check`, `cargo test -p game-core --test architecture_no_rust_authoring --locked`, `cargo test -p game-cli --locked`, `cargo test -p game-kit data --locked`, `cargo test -p game-core --test architecture_docs --locked`, `cargo test -p game-core --test architecture_templates --locked`, `cargo test -p game-core --test architecture_cli_release --locked`, `game-dev authoring-scan` over the no-Rust template and all no-Rust examples, `game-dev check --project` over all no-Rust examples, and `game-player --project ... --smoke-frames 0` for the projectile/full packages.
- [x] 2026-07-02: Phase 12.2 complete. Added `game-dev migrate-ron assets/game.ron --out game.toml`, backed by a `game_kit::data::migrate_legacy_ron_source_to_toml` converter that parses legacy RON, emits primary TOML, round-trips the generated TOML syntax, writes and validates the output with the CLI asset root, and prints migration notes; focused CLI and checked-in legacy RON migration tests passed.
- [x] 2026-07-02: Phase 12.1 complete. Public docs now frame RON references as legacy, migration, advanced, internal fixture, or historical roadmap material; README, start-here/tutorial docs, advanced guidance, distribution policy, animation/tuning docs, and legacy template docs now point the default path at `game.toml`, `game-dev preview`, and migration wording instead of RON-as-primary.
- [x] 2026-07-02: Phase 12.3 complete. Added `docs/migrations/ron-to-toml.md` with the required `Player(( ... Some((...)) ... ))` to `[[prefab]]` TOML example, migration command flow, and conversion notes; linked it from the migration index and pinned it with architecture coverage.
- [x] 2026-07-02: Phase 12.4 complete. Kept the RON tutorial/template as explicitly legacy material, moved the data-driven RON tutorial out of the numbered primary tutorial sequence, and added architecture coverage that start-here docs do not generate the RON template as the default path while legacy RON tests remain labeled.
- [x] 2026-07-02: Phase 12.5 deferred as directed. No `legacy-ron` feature gate was added because the roadmap says not to do this until migration docs and TOML parity are strong enough for a later default-feature change.
- [x] 2026-07-02: Phase 12 validation complete. Built `game-dev`, converted temp copies of `templates/data-driven-demo` plus all five `examples/data-driven-*` RON projects with `game-dev migrate-ron ... --out game.toml`, validated every converted TOML with `game-dev validate-data --file ... --assets ...`, confirmed stale primary-RON phrases are absent outside historical roadmaps, and verified with `cargo fmt --all -- --check`, `cargo test -p game-cli migrate_ron --locked`, `cargo test -p game-kit migrate_ron_to_toml_converts_checked_in_legacy_examples --locked`, `cargo test -p game-kit data --locked`, `cargo test -p game-core --test architecture_no_rust_authoring --locked`, and `cargo test -p game-core --test architecture_docs --locked`.
- [x] 2026-07-02: Phase 13.1 complete. Added primary TOML animation metadata parsing with `[[clip]]` entries, kept legacy RON metadata compatibility, changed `animation_sheet_auto` and data-package loading to prefer `assets/animations/*.toml`, converted the root animation demo metadata and `examples/no-rust-full` animation metadata to TOML, and updated the animation demo to load `.toml`.
- [x] 2026-07-02: Phase 13.2 complete. `game-dev asset-check` now validates `assets/animations/*.toml`, still validates legacy animation `.ron` with a warning, keeps arbitrary `.ron` files rejected, and architecture coverage now rejects any `.ron` file inside primary no-Rust packages.
- [x] 2026-07-02: Phase 13.3 complete. `TuningFile::from_file` now reads TOML tuning files using `[tuning]` numeric keys and `[tuning."name"] value = ...` tables while retaining legacy RON support; live-tuning docs and reload diagnostics now teach `tuning/*.toml`.
- [x] 2026-07-02: Phase 13.4 complete. RON map docs and rustdoc now label RON maps as legacy/advanced material, while primary no-Rust docs continue to list text maps, Tiled, and LDtk as the primary map formats.
- [x] 2026-07-02: Phase 13 validation complete. Confirmed `examples/no-rust-full/assets/animations/player.toml` is the only primary no-Rust animation metadata file, `find templates/no-rust-demo examples/no-rust-* -name '*.ron'` returns no files, stale animation/tuning RON scans only report legacy wording, root `game-dev asset-check` and `game-dev asset-check --project examples/no-rust-full` validate animation TOML, `game-dev authoring-scan --project examples/no-rust-full` passes, and focused/broader checks passed: `cargo fmt --all -- --check`, `cargo test -p game-kit assets::tests::animation --locked`, `cargo test -p game-kit tuning --locked`, `cargo test -p game-cli animation_metadata --locked`, `cargo test -p game-kit data --locked`, `cargo test -p game-cli --locked`, `cargo test -p game-core --test architecture_no_rust_authoring --locked`, and `cargo test -p game-core --test architecture_docs --locked`.
- [x] 2026-07-02: Phase 14.1 complete. Added `cargo xtask package-sdk --release --out <directory> [--features feature-list]`, which builds `game-player` and `game-dev`, copies runtime libraries, launchers, license files, `templates/no-rust-demo`, and optional no-Rust examples; `.github/workflows/release.yml` now builds, verifies, uploads, and attaches `game-sdk-linux-x86_64.zip` and `game-sdk-windows-x86_64.zip`.
- [x] 2026-07-02: Phase 14.2 complete. Extended `scripts/verify-release-artifact.sh` and `scripts/verify-github-release-artifacts.sh` to verify SDK archives, including `game-player`, `game-dev`, launcher scripts, `README.txt`, `LICENSE`, `THIRD_PARTY_NOTICES.md`, the no-Rust template `game.toml`, absence of Rust project files in that template, optional no-Rust examples, and README wording that no Rust is required; a local Linux SDK zip passed the verifier.
- [x] 2026-07-02: Phase 14.3 complete. Documented the local SDK dry-run in `docs/release-checklist.md`, including `cargo xtask package-sdk`, zip verification, `game-dev new --template no-rust`, `game-dev check`, and `game-dev preview --smoke-frames`; updated distribution/README language to name `game-sdk-linux-x86_64.zip` and `game-sdk-windows-x86_64.zip` as the no-Rust SDK artifacts.
- [x] 2026-07-02: Phase 14.4 complete. Added a CI SDK job step that packages `/tmp/game-sdk`, installs a poison `cargo` shim after the SDK is built, creates a no-Rust project from the SDK, runs SDK `game-dev check`, and smoke-previews through SDK `game-dev preview --smoke-frames 0` under Xvfb/software Vulkan without allowing Cargo calls.
- [x] 2026-07-02: Phase 14 validation complete. Confirmed SDK artifact names are consistent, and the release packaging path passed `cargo fmt --all -- --check`, `cargo test -p game-cli package --locked`, `cargo test -p game-core --test architecture_cli_release --locked`, `cargo test -p game-cli --locked`, shell syntax checks for release verifiers, and `scripts/verify-release-artifact.sh /tmp/game-sdk-linux-x86_64.zip linux sdk`; the local SDK dry-run passed through `game-dev new` and `game-dev check` with a poison Cargo shim, while local preview was deferred to the CI Xvfb smoke step because `xvfb-run` is not installed on this machine.
- [x] 2026-07-02: Phase 15.1 complete. Kept `game_starter::prelude::*`, `game_kit::beginner::prelude::*`, and `content_plugin!` available, and added an architecture guard that checks the starter reexport, beginner prelude macro export, and `content_plugin!` definition remain in place.
- [x] 2026-07-02: Phase 15.2 complete. Preserved `game_kit::advanced::prelude::*` and the explicit `testbed-content` advanced lab, and added public docs/tests requiring the exact boundary statement: "Advanced Rust authoring is not the primary no-Rust surface."
- [x] 2026-07-02: Phase 15.3 complete. Updated `docs/api-boundary.md` with the compatibility prelude removal plan for v0.2.x, v0.3.x, and v0.4.x or pre-1.0, and extended the API-surface test to enforce those policy lines.
- [x] 2026-07-02: Phase 15.4 complete. Re-ran and kept architecture coverage for narrow root exports, beginner docs/imports, advanced docs, and content crate engine-boundary rules through `architecture_api_surface`, `architecture_docs`, `architecture_advanced_surface`, and `architecture_content_crates`.
- [x] 2026-07-02: Phase 15.5 complete. Removed no-Rust/data schema type exports from `game_kit::beginner::prelude::*` and added a guard that keeps `BeginnerGameFile`, related schema structs/enums, and `RuleEffectFile` reachable through `game_kit::data` instead of the beginner prelude.
- [x] 2026-07-02: Phase 15 validation complete. Confirmed current Rust demo packages compile with `cargo check --workspace --bins --locked --features game/ci-build-sdl3`; `cargo fmt --all`, `cargo test -p game-core --test architecture_api_surface --test architecture_advanced_surface --locked`, and no-Rust/docs/template/content-crate architecture tests passed; the attempted `cargo check --workspace --examples` was a no-op because demos are workspace packages rather than Cargo example targets.
- [x] 2026-07-02: Phase 16.1 complete. Kept `crates/game-kit/src/data/tests.rs` removed in favor of focused `crates/game-kit/src/data/tests/{mod.rs,toml_primary.rs,ron_legacy.rs}` modules, with an architecture guard that rejects a restored monolithic data test file and caps each data test module at 1,500 lines.
- [x] 2026-07-02: Phase 16.2 complete. Split remaining no-Rust CLI command logic out of `crates/game-cli/src/lib.rs` by adding focused `commands/asset_check.rs`, `commands/validate_data.rs`, and `commands/package_sdk.rs`; `lib.rs` now routes commands while package, preview, check, migration, scan, validation, asset checking, and SDK packaging live under `commands/`.
- [x] 2026-07-02: Phase 16.3 complete. Kept data parsing/building/validation split across `model.rs`, `toml_schema.rs`, `toml_parse.rs`, `toml_emit.rs`, `legacy_ron.rs`, `validate.rs`, and `build.rs`, with an architecture guard requiring those focused modules to stay wired from `data/mod.rs`.
- [x] 2026-07-02: Phase 16.4 complete. Added size guard tests for data modules and data test modules at 1,500 lines, CLI command modules and architecture test files at 1,000 lines, plus a 400-line routing guard for `crates/game-cli/src/lib.rs`; any future exception must be explicit in the failing architecture check.
- [x] 2026-07-02: Phase 16 validation complete. `cargo fmt --all`, `cargo test -p game-cli package --locked`, `cargo test -p game-cli --locked`, and `cargo test -p game-core --test architecture_api_surface --locked` passed; current line counts are below the guards, with the largest data test module at 1,267 lines, largest data parser/schema module at 1,169 lines, largest CLI command module at 541 lines, largest architecture test at 884 lines, and `game-cli/src/lib.rs` at 104 lines.
- [x] 2026-07-02: Phase 17.1 complete. Final source gates passed on the current tree: `cargo fmt --all -- --check`, `cargo test --workspace --locked --features game/ci-build-sdl3`, `cargo test -p game-runtime --test headless_runner --no-default-features --locked`, `cargo clippy --workspace --all-targets --locked --features game/ci-build-sdl3 -- -D warnings`, `cargo build -p game-player --release --locked --features ci-build-sdl3`, and `cargo build -p game-cli --release --locked --features ci-build-sdl3`; clippy cleanups included whole-word scan simplification, TOML emitter argument bundling, and package helper ordering.
- [x] 2026-07-02: Phase 17.2 complete with local preview exception documented. A fresh `/tmp/no-rust-smoke` was generated through release `game-dev new --template no-rust`, verified to include `assets/fonts/DejaVuSans.ttf`, checked, and packaged to `/tmp/no-rust-package.zip` with a poison `cargo` shim first in `PATH`; `game-player --project /tmp/no-rust-smoke` reached SDL/Vulkan window creation and then failed locally because this machine lacks Xvfb/display surface support (`VK_KHR_surface`), matching the CI-only Xvfb/lavapipe preview requirement.
- [x] 2026-07-02: Phase 17.3 complete. Ran the plan's `cargo test -p game-core architecture_no_rust --locked` filter, then the stronger `cargo test -p game-core --test architecture_no_rust_authoring --locked`; release `game-dev authoring-scan --project /tmp/no-rust-smoke` and `game-dev authoring-scan --project examples/no-rust-full` both reported clean authoring surfaces under the poison Cargo shim.
- [x] 2026-07-02: Phase 17.4 complete with local preview exception documented. Built `/tmp/game-sdk-linux-x86_64`, zipped it, and verified it with `scripts/verify-release-artifact.sh /tmp/game-sdk-linux-x86_64.zip linux sdk`; the verifier now requires SDK template/example fonts. From the SDK, `game-dev new /tmp/sdk-smoke --template no-rust` and `game-dev check --project /tmp/sdk-smoke` passed with Cargo poisoned, while SDK `game-player --project /tmp/sdk-smoke` hit the same local missing `VK_KHR_surface` display limitation.
- [x] 2026-07-02: Phase 17.5 complete with local preview exception documented. Manually edited `/tmp/no-rust-smoke/game.toml` to change player speed, edited the text map to add another pickup, confirmed `game-dev check` passed without Cargo, intentionally typoed the coin texture to `coiin`, and confirmed `game-dev check` produced a readable `game.toml` error naming the unknown `coin` texture, known textures, and the suggestion; local preview attempts remain blocked by the machine's missing SDL/Vulkan surface support.
- [x] 2026-07-02: Final acceptance complete for implemented code and all non-display local gates. The no-Rust SDK/package path is implemented, verified, documented, and protected by tests/CI; local player preview smoke cannot complete on this host without Xvfb or another Vulkan-capable SDL surface, but the CI SDK job runs that preview path under Xvfb/software Vulkan and the local failures occur after assets/config validation at window-surface creation.

## Baseline notes

- 2026-07-02: No pre-existing failures were observed in the Phase 0.2 baseline command set.

This document is intentionally stricter than the earlier “content code should look like game code” roadmap. The previous roadmap was mostly about making Rust content code use a clean beginner API. That work is now largely done. This roadmap treats that Rust API as a **secondary/advanced tier**, not the final goal.

The new goal is a true primary authoring surface that is:

- not Rust source code,
- not Rust-shaped serialized syntax,
- editable in any plain text editor,
- readable by a non-programmer,
- runnable through a prebuilt executable,
- previewable/reloadable without compiling,
- mechanically protected from leaking engine/runtime/backend/Rust concepts.

Until this roadmap is complete, gameplay/game-design expansion is explicitly out of scope. Every change should strengthen the foundation rather than add new game content.

---

## Target objective

The project’s objective, until fully met, is not to build a game but to build a Rust-based technical foundation — using `ash`, SDL, and surrounding low-level libraries — that fully owns and hides all rendering, audio, platform/windowing, memory management, runtime/game-loop, ECS, validation, and backend complexity.

The primary content-authoring surface must:

1. **Not expose engine concepts**
   - No renderer, swapchain, Vulkan, SDL, audio backend, memory allocator, runtime loop, ECS, component storage, schedules, registries, raw commands, raw resources, entity IDs, or backend handles.

2. **Not be Rust**
   - No compilation step to read, write, or preview.
   - No Rust syntax, semantics, or idioms.
   - No ownership/borrowing concepts.
   - No traits/generics/lifetimes/macros.
   - No `Some(...)`, enum-constructor serialization, tuple wrappers, `::`, `?`, `Ok`, `Result`, `fn`, `impl`, `struct`, `enum`, `pub`, `use`, `match`, or compiler-facing coercions.

3. **Be plain declarative data/configuration**
   - A non-programmer should be able to scan it and understand the objects, maps, rules, sounds, and scenes.
   - The canonical primary file should look like normal configuration, not Rust source and not RON.

4. **Be runnable without a Rust toolchain**
   - A user should be able to download a packaged executable, edit `game.toml` or equivalent, and run/preview the result.
   - The user must not need `cargo`, `rustc`, `cargo-generate`, or a generated Rust project for the primary path.

5. **Support preview/reload without build**
   - Editing the authoring files should be reflected by a reload, soft restart, or prebuilt preview restart.
   - Structural changes may restart the prebuilt runner, but they must not require recompilation.

6. **Be mechanically enforced**
   - Tests and tooling must scan primary authoring files, examples, templates, and primary docs for forbidden Rust and engine vocabulary.
   - Enforcement cannot rely on informal review.

Only after those conditions hold verifiably should the project scope expand to building an actual small game.

---

## Current code facts this roadmap is based on

The uploaded project already has a strong technical foundation and a good secondary Rust authoring tier.

### Existing architecture that should be preserved

Current workspace structure:

- `game-core` owns engine-neutral primitives: ECS-ish world/resource/query storage, commands, schedules, input, IDs, backend traits, asset registries, map data, render-frame data.
- `game-runtime` owns `Runner<P, R, A>`, the fixed timestep, platform/renderer/audio orchestration, active map state, runtime command processing, and command error policy.
- `game-renderer-vulkan`, `game-platform-sdl`, `game-audio`, and `game-backend-headless` own technical backends.
- `game-kit` is the content authoring facade over `game-core`, `game-map`, `game-ai`, `game-combat`, and `game-physics`.
- `game-starter` exposes `game_starter::prelude::*` and `run_game` for standalone beginner Rust projects.
- `simple-content` and `arena-content` use the beginner API.
- `testbed-content` is explicitly advanced.
- Current architecture tests already enforce many import/boundary rules.

Those layers are good. Do **not** collapse them.

### Current primary-like data path is not enough

The current no-Rust-like path is `assets/game.ron`, loaded by a Rust wrapper such as:

```rust
use game_starter::prelude::*;

fn main() -> Result<()> {
    run_game("Title", |game| {
        let _controls = game.load_beginner_file("game.ron")?;
        Ok(())
    })
}
```

That is useful, but it does **not** satisfy the new objective because:

- The project template still contains `Cargo.toml`, `src/main.rs`, and `build.rs`.
- Running the generated project still requires a Rust toolchain.
- The data file is RON and contains Rust-shaped syntax:
  - `Player(( ... ))`
  - `Enemy(( ... ))`
  - `TextMap(( ... ))`
  - `Some(( ... ))`
  - enum-like names such as `TopDownControls`
- Animation metadata currently uses `.ron`.
- Tuning files are documented as RON.
- Several docs describe RON as the no-Rust path.
- CI proves generated Rust templates compile, but does not yet prove a no-Rust package can be edited and run through only a prebuilt executable.
- The current architecture tests protect beginner Rust APIs, but do not yet enforce “primary authoring files are not Rust-like.”

### Specific current implementation details to address

The current source tree contains these relevant implementation details:

- `crates/game-kit/src/data/schema.rs` defines the current RON-shaped schema with enums such as `BeginnerPrefabFile::Player`, `BeginnerMapFile::TextMap`, `BeginnerRuleKind`, `RuleEffectFile`, and `RuleConditionFile`.
- `crates/game-kit/src/data/mod.rs` parses authoring files through `ron::from_str` in `parse_beginner_game_source`.
- `crates/game-kit/src/data/mod.rs` exposes `load_beginner_game_file`, `validate_beginner_game_file`, and `rebuild_beginner_content_runtime`.
- `crates/game-kit/src/context.rs` reloads `game.ron` through `reload_beginner_file`.
- `crates/game-kit/src/context.rs` still drains `BeginnerSpawnQueue` failures by logging errors instead of routing them through structured diagnostics.
- `crates/game-kit/src/map.rs` contains `switch_world_to_map`, which currently mutates `ContentRuntime.current_map`, clears the world, then spawns map objects. If spawning fails, the transition is not fully transactional.
- `crates/game-cli/src/lib.rs` has `asset-check` hardcoded to `current_dir().join("assets")` instead of consistently using configured asset-root logic.
- `crates/game-cli/Cargo.toml` only forwards `ci-build-sdl3`; it does not forward optional `ogg`/`mp3` validation features from `game-audio`.
- `templates/data-driven-demo` is still a Rust project template with `Cargo.toml`, `build.rs`, `src/main.rs`, and `assets/game.ron`.
- `docs/api-boundary.md`, `docs/beginner-authoring.md`, and `docs/content-authoring.md` mention a “No-Rust data path,” but they currently point to RON.
- `docs/distribution-policy.md` describes prebuilt demo zips, not a complete no-Rust authoring SDK.
- `.github/workflows/ci.yml` has strong generated-template and smoke coverage, but it does not yet validate a no-Rust project generated without `Cargo.toml` and run through a prebuilt player.

---

## Execution protocol for an implementation agent

Use this protocol for every phase.

1. Re-read the target objective before editing.
2. Re-read the current phase and its definition of done.
3. Make the smallest coherent code change for one checklist item.
4. Update this roadmap only after the item is actually implemented.
5. Run the narrowest relevant check first.
6. Then run the phase-level checks.
7. Do not add new gameplay features while implementing this roadmap.
8. Do not weaken architecture tests to make code pass.
9. Do not rename the Rust builder API into “No-Rust” and call the goal done.
10. Do not keep RON as the primary surface.
11. Do not require `cargo`, `rustc`, or a generated Rust wrapper for the primary workflow.
12. When in doubt, prefer a stricter boundary and a smaller public surface.

---

## Global definition of done

This roadmap is complete only when all of these are true.

### Authoring surface

- [x] The canonical primary file is not RON.
- [x] The canonical primary file contains no Rust-shaped enum constructors.
- [x] The canonical primary file contains no `Some(...)`, tuple wrappers, `::`, Rust keywords, generics, lifetimes, macros, or compiler concepts.
- [x] The canonical primary file uses ordinary declarative keys such as `kind = "player"`, `sprite = "player"`, `map = "level-1"`, `when = "all-enemies-dead"`, or equivalent non-Rust config syntax.
- [x] The canonical primary file is understandable without programming background.

### Runtime/distribution

- [x] A no-Rust project can be created without `Cargo.toml`, `src/main.rs`, or `build.rs`.
- [x] A no-Rust project can be checked without Rust installed.
- [x] A no-Rust project can be previewed through a prebuilt executable. Local smoke preview requires Xvfb or another SDL/Vulkan surface; this host lacks `VK_KHR_surface`, while CI runs the Xvfb/software Vulkan smoke path.
- [x] A no-Rust project can be packaged through a prebuilt CLI or release artifact.
- [x] Editing authoring data can be reflected through reload, soft restart, or preview restart without a build.

### Enforcement

- [x] Architecture tests scan primary authoring files for Rust syntax and engine vocabulary.
- [x] Architecture tests scan primary docs for Rust-first language in primary sections.
- [x] Architecture tests ensure primary templates contain no Rust project files.
- [x] CI runs the no-Rust package path.
- [x] CI runs a prebuilt-runner smoke test using no user Rust code.
- [x] CI validates no-Rust authoring files and asset packages.

### Secondary Rust API

- [x] `game_starter::prelude::*` and `game_kit::beginner::prelude::*` remain available as secondary Rust APIs.
- [x] `game_kit::advanced::prelude::*` remains available for deliberate advanced Rust authoring.
- [x] Docs clearly state that those Rust APIs are not the primary no-Rust surface.
- [x] Compatibility surfaces have a removal/gating plan.

---

# Phase 0 — Baseline current repo state

## Goal

Capture the current state before making the objective stricter. This phase should not change runtime behavior.

## Steps

### 0.1 Create a branch

```bash
git switch -c architecture/primary-no-rust-authoring-foundation
git status --short
```

Expected result:

- working tree is clean before edits,
- all roadmap work lands on the branch.

### 0.2 Run baseline checks

Run what is feasible locally:

```bash
cargo fmt --all -- --check
cargo test --workspace --locked --features game/ci-build-sdl3
cargo test -p game-runtime --test headless_runner --no-default-features --locked
cargo clippy --workspace --all-targets --locked --features game/ci-build-sdl3 -- -D warnings
cargo run -p game-cli --features ci-build-sdl3 -- doctor --explain
cargo run -p game-cli --features ci-build-sdl3 -- validate-data assets/game.ron
cargo run -p game-cli --features ci-build-sdl3 -- asset-check
```

If graphics dependencies are not available, run at least:

```bash
cargo fmt --all -- --check
cargo test -p game-core --locked
cargo test -p game-kit --locked
cargo test -p game-runtime --test headless_runner --no-default-features --locked
cargo test -p game-cli --locked
```

Record any pre-existing failures in this roadmap under a “Baseline notes” section before changing code.

### 0.3 Confirm current no-Rust gaps

Manually inspect and record:

```bash
rg "game\.ron|RON|Some\(|Player\(\(|Enemy\(\(|TextMap\(\(" \
  templates examples docs crates/game-kit/src crates/game-cli/src

find templates/data-driven-demo -maxdepth 3 -type f | sort
```

Expected current findings:

- `templates/data-driven-demo` contains `Cargo.toml`, `build.rs`, `src/main.rs`, and `assets/game.ron`.
- docs call RON the no-Rust data path.
- data parser uses `ron::from_str`.
- data schema uses Rust-like enum constructors.
- animation metadata examples use `.ron`.

### 0.4 Add this roadmap to the repo

Create:

```text
docs/roadmaps/primary-no-rust-authoring-foundation.md
```

Update:

```text
docs/roadmaps/README.md
```

Add a row such as:

```markdown
| Primary no-Rust authoring foundation | Current | Replaces RON-as-primary with a true no-Rust data/package/runtime workflow. |
```

Do not mark anything complete until the code exists.

---

# Phase 1 — Scope reset: make the docs tell the truth

## Goal

Make the repository’s public language match the stricter objective. The project should stop describing ergonomic Rust code or RON as the completed primary goal.

## Required principle

The current Rust builder API is good and should stay, but it is now a **secondary Rust authoring tier**. RON is a **legacy/transitional data format**, not the official primary surface.

## Files to update

- `README.md`
- `docs/api-boundary.md`
- `docs/beginner-authoring.md`
- `docs/content-authoring.md`
- `docs/beginner-productization-roadmap.md`
- `docs/distribution-policy.md`
- `docs/release-checklist.md`
- `docs/future-editor-import.md`
- `docs/tutorials/*.md`
- `docs/cookbook/*.md` where they mention no-Rust data
- `templates/data-driven-demo/README.md`
- `CHANGELOG.md` once implementation begins

## Steps

### 1.1 Rewrite the API boundary document around three tiers

Update `docs/api-boundary.md` to define these tiers:

1. **Primary no-Rust authoring**
   - `game.toml` or equivalent canonical config.
   - Text maps, image/audio/font files, optional Tiled/LDtk.
   - No Rust project files.
   - Prebuilt runner and CLI.

2. **Secondary beginner Rust authoring**
   - `game_starter::prelude::*`
   - `game_kit::beginner::prelude::*`
   - Builder-style Rust API for users who choose Rust.

3. **Advanced Rust authoring**
   - `game_kit::advanced::prelude::*`
   - ECS/query/system escape hatches through the facade.
   - Still no direct backend imports from content crates.

Make the document explicitly say:

```text
The Rust builder API is not the primary authoring surface. It is a secondary tier.
The primary surface is a plain data/config package that runs through the prebuilt player.
```

### 1.2 Stop calling RON the no-Rust target

Replace language like:

```text
No-Rust data path: edit assets/game.ron
```

with transitional language:

```text
Legacy data path: assets/game.ron exists today.
Primary no-Rust target: game.toml / declarative config, implemented by this roadmap.
```

Once the new format lands, update again to:

```text
Primary no-Rust path: edit game.toml and assets/.
Legacy RON path: supported for migration only.
```

### 1.3 Add a “not done yet” warning where appropriate

In roadmap/status docs, do not claim that no-Rust authoring is complete until:

- the canonical non-RON format exists,
- the prebuilt player exists,
- the no-Rust template has no Rust files,
- CI runs the no-Rust package without compiling user code.

### 1.4 Keep the Rust API docs, but demote their status

In `docs/beginner-authoring.md` and `docs/content-authoring.md`, keep the Rust API examples but label them clearly:

```markdown
## Secondary Rust authoring path
Use this only if you want to write Rust.
```

Do not remove useful Rust examples. They are still valuable. They just cannot be the proof of the primary objective.

### 1.5 Update release checklist language

Add release checklist gates:

- [x] primary no-Rust template contains no Rust files,
- [x] primary no-Rust package runs without `cargo`,
- [x] primary authoring files pass Rust-vocabulary scanner,
- [x] primary docs do not teach Rust/RON as the default path,
- [x] prebuilt runner loads `game.toml`,
- [x] packaged artifact verification includes no-Rust project.

## Phase 1 validation

```bash
rg "No-Rust|no-Rust|no-rust|game\.ron|RON" README.md docs templates
```

Expected:

- RON may appear only as legacy/migration/advanced wording.
- Primary path wording points to the new roadmap until implemented.
- No doc claims the stricter objective is already complete.

---

# Phase 2 — Choose and freeze the primary package layout

## Goal

Define a concrete no-Rust project structure that can be generated, edited, checked, previewed, and packaged without a Rust toolchain.

## Recommended canonical layout

Use this as the primary no-Rust package layout:

```text
my-game/
  game.toml
  assets/
    textures/
      player.png
      slime.png
      floor.png
      wall.png
    sounds/
      hit.wav
      coin.wav
    music/
      theme.ogg
    fonts/
      DejaVuSans.ttf
    maps/
      level-1.txt
    animations/
      player.toml
  README.txt
```

Rationale:

- `game.toml` at the project root is obvious to a non-programmer.
- `assets/` contains media and map files.
- There is no `Cargo.toml`.
- There is no `src/main.rs`.
- There is no `build.rs`.
- There is no hidden Rust project shape.

The engine may internally map this to the existing asset-root conventions, but the user-facing shape should not look like a Rust crate.

## Steps

### 2.1 Add a package-layout document

Create:

```text
docs/no-rust-package-layout.md
```

It should explain:

- `game.toml` is the primary file.
- media files live under `assets/`.
- text maps live under `assets/maps/`.
- animation metadata uses `assets/animations/*.toml`.
- no Rust toolchain is needed.
- `game-dev check` validates the package.
- `game-dev preview` runs the prebuilt player.
- `game-dev package` creates a shareable folder/zip.

### 2.2 Define project-type detection

Add a helper in `crates/game-cli/src/paths.rs` or a new `crates/game-cli/src/project.rs`:

```rust
enum ProjectKind {
    NoRustPackage,
    RustStarterProject,
    WorkspaceDemo,
}
```

Detection rules:

- If `game.toml` exists and no `Cargo.toml` exists: `NoRustPackage`.
- If `Cargo.toml` exists and has `[package.metadata.game]`: `RustStarterProject`.
- If inside the engine workspace: `WorkspaceDemo` where appropriate.
- If both `game.toml` and `Cargo.toml` exist: allow, but `game-dev check --primary` should check only the no-Rust package and warn that Rust files are secondary.

### 2.3 Define canonical path resolution

Add one source of truth for no-Rust packages:

```rust
struct NoRustProjectPaths {
    root: PathBuf,
    game_file: PathBuf,
    asset_dir: PathBuf,
}
```

Rules:

- Default root is current directory.
- Default game file is `root/game.toml`.
- Default asset dir is `root/assets`.
- CLI flags may override:
  - `--project <dir>`
  - `--file <path>`
  - `--assets <dir>`
- `GAME_ASSET_DIR` remains supported for compatibility, but CLI flags and project layout should be clearer for the primary path.

### 2.4 Do not reuse Cargo metadata for primary packages

Current Rust starter packaging reads `Cargo.toml` metadata such as:

```toml
[package.metadata.game]
title = "..."
asset_dir = "assets"
```

For no-Rust packages, this metadata must move into `game.toml`, for example:

```toml
[game]
title = "My Game"
window_width = 1280
window_height = 720
```

## Phase 2 validation

Add CLI unit tests for:

- no-Rust package detection,
- root `game.toml` resolution,
- asset dir resolution,
- explicit `--project`, `--file`, and `--assets`,
- package with no `Cargo.toml` is not treated as a Rust starter project.

---

# Phase 3 — Replace RON-as-primary with a non-Rust declarative format

## Goal

Introduce a canonical non-Rust file format while preserving existing RON as legacy compatibility.

## Format recommendation

Use TOML for the first canonical format:

```text
game.toml
```

Reasons:

- It looks like ordinary configuration.
- It is widely recognizable.
- It avoids Rust-style enum constructors and tuple wrappers.
- It supports comments.
- It is easy to hand-edit.
- It maps cleanly to `serde`.
- It is much less footgun-prone than YAML for a small game config.
- It avoids making the project invent a custom parser too early.

Do not use RON as the primary format. Do not use JSON as the main hand-authored format unless TOML proves unsuitable, because JSON is noisy and comments are not standard.

## Canonical authoring style

The canonical style should use:

- lowercase or kebab-case strings for kinds,
- ordinary tables,
- ordinary arrays,
- no Rust enum constructors,
- no `Some`,
- no tuple fields,
- no `TopDownControls`/`PlayerCollectsPickups` PascalCase as the primary spelling.

Example target syntax:

```toml
version = 2

[game]
title = "Coin Collector"
start_map = "level-1"

[assets]
textures = ["player", "slime", "coin", "floor", "wall", "door", "bolt"]
sounds = ["hit", "coin", "shoot"]

[controls]
preset = "top-down"

[[prefab]]
kind = "player"
name = "player"
sprite = "player"
speed = 130
health = 100
tags = ["player"]

[prefab.melee]
range = 30
damage = 25

[[prefab]]
kind = "enemy"
name = "slime"
sprite = "slime"
speed = 80
health = 30
chase_player = true

[prefab.melee]
range = 26
damage = 6

[[prefab]]
kind = "pickup"
name = "coin"
sprite = "coin"
score = 1
sound = "coin"

[[map]]
kind = "text"
name = "level-1"
file = "assets/maps/level-1.txt"
floor = "floor"
wall = "wall"
start = true

[map.legend]
P = "player"
E = "slime"
C = "coin"

[rules]
enabled = [
  "top-down-controls",
  "player-collects-pickups",
  "enemies-damage-player",
  "camera-follows-player",
  "show-score",
  "show-player-health",
]
```

For effects and conditions, prefer explicit object arrays:

```toml
[[rule]]
name = "win after enemies"
when = "all-enemies-dead"

[[rule.then]]
action = "change-scene"
scene = "win"
```

or:

```toml
[[custom_rule]]
kind = "countdown"
name = "bomb timer"
tag = "bomb"
key = "seconds"

[[custom_rule.when_zero]]
action = "damage-player"
amount = 10
radius = 80

[[custom_rule.when_zero]]
action = "play-sound"
sound = "explosion"
```

Avoid positional values such as:

```toml
theme = ["floor", "wall"]
```

for the official docs if named fields are clearer:

```toml
floor = "floor"
wall = "wall"
```

Arrays are acceptable for lists of names, but not for values where field names are important to non-programmers.

## Steps

### 3.1 Add TOML parsing dependency

Add a workspace dependency for TOML parsing.

Recommended mechanical approach:

```bash
cargo add toml --workspace
```

Then add it to crates that parse/validate no-Rust files:

```toml
game-kit = ...
game-cli = ...
```

Do not hardcode a version in this roadmap; let Cargo select the compatible current version.

### 3.2 Split wire formats from internal game data

Currently `BeginnerGameFile` is both:

- the RON deserialization shape,
- the internal normalized game data shape used by the builder.

That is no longer good enough.

Add a format-neutral internal model, for example:

```text
crates/game-kit/src/data/model.rs
```

Suggested types:

```rust
pub(crate) struct AuthoringGameFile {
    version: u32,
    game: GameMetadata,
    assets: AuthoringAssets,
    controls: AuthoringControls,
    prefabs: Vec<AuthoringPrefab>,
    maps: Vec<AuthoringMap>,
    scene_flow: Option<AuthoringSceneFlow>,
    audio: AuthoringAudio,
    actions: Vec<AuthoringAction>,
    custom_rules: Vec<AuthoringCustomRule>,
    rules: Vec<AuthoringRule>,
}
```

Keep this model free from RON-specific enum constructor assumptions.

### 3.3 Move current RON schema into legacy module

Rename or reorganize:

```text
crates/game-kit/src/data/schema.rs
```

into something like:

```text
crates/game-kit/src/data/legacy_ron/schema.rs
crates/game-kit/src/data/legacy_ron/parse.rs
```

or keep `schema.rs` temporarily but introduce:

```text
crates/game-kit/src/data/v1_ron.rs
crates/game-kit/src/data/v2_toml.rs
crates/game-kit/src/data/model.rs
```

The important rule:

- TOML primary schema must not inherit Rust-looking names from the RON schema.
- RON may translate into the internal model.
- TOML may translate into the internal model.
- Build/validate code should consume the internal model.

### 3.4 Implement `game.toml` schema

Create:

```text
crates/game-kit/src/data/toml_schema.rs
crates/game-kit/src/data/toml_parse.rs
```

Suggested serde strategy:

- `#[serde(rename_all = "kebab-case")]` for string enum values where appropriate.
- `#[serde(tag = "kind", rename_all = "kebab-case")]` for arrays of tables such as prefabs and maps.
- `#[serde(tag = "action", rename_all = "kebab-case")]` for effects.
- `#[serde(tag = "kind", rename_all = "kebab-case")]` for custom rules.
- Use named fields instead of tuple fields.
- Use `deny_unknown_fields` selectively once the diagnostics are good.

Example shapes:

```rust
#[derive(Deserialize)]
struct GameTomlFile {
    version: u32,
    #[serde(default)]
    game: GameTomlMetadata,
    #[serde(default)]
    assets: AssetsToml,
    #[serde(default)]
    controls: ControlsToml,
    #[serde(default)]
    prefab: Vec<PrefabToml>,
    #[serde(default)]
    map: Vec<MapToml>,
    #[serde(default)]
    rule: Vec<RuleToml>,
}
```

TOML table arrays naturally read as:

```toml
[[prefab]]
kind = "player"
```

instead of:

```ron
Player((
```

### 3.5 Add format detection

Add:

```rust
enum AuthoringFormat {
    Toml,
    RonLegacy,
}
```

Rules:

- `.toml` => primary parser.
- `.ron` => legacy parser.
- no extension / unknown extension => clear error:
  - “Use `game.toml` for primary no-Rust authoring.”
  - “RON is legacy; use `game-dev migrate-ron` if needed.”

### 3.6 Add public primary functions

Add new public functions in `game-kit` data module:

```rust
pub fn load_authoring_file(game: &mut GameApp<'_>, path: impl AsRef<Path>) -> Result<TopDownControls>;
pub fn validate_authoring_file(path: impl AsRef<Path>) -> Result<()>;
```

Possible naming options:

- `load_game_file`
- `load_project_file`
- `load_authoring_file`

Pick one and use it consistently. Avoid keeping `load_beginner_file` as the only official name because it still points to the old RON era.

Keep compatibility:

```rust
pub fn load_beginner_game_file(...) -> Result<TopDownControls> {
    load_authoring_file(...)
}
```

but mark docs carefully:

- `load_beginner_file("game.ron")` is legacy compatibility.
- `load_authoring_file("game.toml")` is primary.

### 3.7 Add diagnostics that speak config language

Replace messages such as:

```text
not valid RON
Use controls like TopDown and rules like TopDownControls
```

with primary-format messages:

```text
game.toml could not be read as game config.
Use controls.preset = "top-down".
Use rules.enabled = ["top-down-controls", "show-score"].
```

For unknown kind values, include suggestions:

```text
unknown prefab kind "plaer"
Known prefab kinds: player, enemy, pickup, door, projectile, spawner, trigger, checkpoint.
Did you mean "player"?
```

### 3.8 Keep existing RON tests, but move them to legacy coverage

Current `crates/game-kit/src/data/tests.rs` has extensive coverage around RON. Do not delete it blindly.

Split tests into:

```text
crates/game-kit/src/data/tests/
  mod.rs
  toml_primary.rs
  ron_legacy.rs
  validation.rs
  reload.rs
  diagnostics.rs
```

or equivalent integration tests if the current module structure makes that easier.

The primary acceptance tests should use TOML. Legacy RON tests should exist but be labeled legacy.

## Phase 3 validation

Add tests:

- valid minimal `game.toml` loads,
- full existing `data-driven-full-demo` equivalent loads from TOML,
- TOML uses lower-kebab strings,
- unknown kind gives suggestion,
- `Some(` in primary data fixture fails architecture scan,
- `Player((` in primary data fixture fails architecture scan,
- `.ron` still validates through legacy path.

Run:

```bash
cargo test -p game-kit data --locked
cargo test -p game-core architecture --locked
```

---

# Phase 4 — Make primary files relative to their package, not to Cargo

## Goal

A no-Rust package should resolve files relative to the package layout, not via Rust crate assumptions.

## Current issue

Current path helpers in `crates/game-kit/src/paths.rs` resolve authoring assets through `GAME_ASSET_DIR` and search roots around the executable/current directory. That works for the current Rust demos, but the primary no-Rust package needs a simpler mental model:

```text
game.toml lives here
assets/ lives next to it
paths in game.toml resolve from this package
```

## Steps

### 4.1 Introduce explicit authoring roots

Add a data-load context:

```rust
pub(crate) struct AuthoringLoadContext {
    project_root: PathBuf,
    asset_root: PathBuf,
    source_file: PathBuf,
}
```

Rules:

- If loading root `game.toml`, `project_root` is `game.toml` parent.
- Default `asset_root` is `project_root/assets`.
- Paths in `game.toml` may be:
  - project-relative,
  - asset-relative where documented,
  - absolute for advanced/manual cases.

### 4.2 Stop assuming all authoring files are under `assets/`

Current `read_beginner_game_file` formats errors as:

```text
could not read beginner game file 'assets/...'
```

Primary path should say:

```text
could not read game config 'game.toml'
looked for '/path/to/my-game/game.toml'
```

### 4.3 Keep `GAME_ASSET_DIR` as compatibility

Do not break existing examples immediately. Instead:

- New primary CLI passes explicit project/asset roots.
- Existing Rust examples may continue to use `GAME_ASSET_DIR`.
- Docs should teach primary package paths first.

### 4.4 Update asset references in the model

For primary TOML, choose one of these patterns and enforce it consistently.

Preferred beginner spelling:

```toml
[assets]
textures = ["player", "slime", "floor", "wall"]
```

This means:

```text
assets/textures/player.png
assets/textures/slime.png
assets/textures/floor.png
assets/textures/wall.png
```

For custom paths:

```toml
[[asset.texture]]
name = "player"
file = "assets/art/hero.png"
```

Do not force beginners to write Rust-like asset handles.

## Phase 4 validation

Add tests:

- root `game.toml` can refer to `assets/maps/level-1.txt`,
- package can be validated from a different current directory,
- absolute `--project` path works,
- `GAME_ASSET_DIR` compatibility still works for existing examples.

---

# Phase 5 — Create the true no-Rust template

## Goal

Create a primary starter template that contains no Rust files and no Cargo project.

## Files to add

```text
templates/no-rust-demo/
  game.toml
  README.txt
  assets/
    maps/level-1.txt
    textures/... starter PNGs
    sounds/... starter WAVs
```

Do not include:

```text
Cargo.toml
build.rs
src/main.rs
cargo-generate.toml
```

If starter assets are too bulky to store directly, generate them through `game-dev new` rather than through a Rust `build.rs`.

## Steps

### 5.1 Move starter asset generation out of template `build.rs`

Current templates generate placeholder PNG/WAV files through long `build.rs` files. That is acceptable for Rust templates but bad for the primary no-Rust template.

Choose one implementation:

1. Store small starter assets directly in `templates/no-rust-demo/assets/...`.
2. Or move placeholder generation into `game-cli` and have `game-dev new --template no-rust` write the files.
3. Or add a tiny internal asset generator module in `game-cli`.

Recommended: move generation into `game-cli`, because it keeps templates light and avoids binary assets in source if desired.

Suggested file:

```text
crates/game-cli/src/starter_assets.rs
```

Use the existing `build.rs` generation code as the source, but keep it hidden inside the tool.

### 5.2 Add `DemoTemplate::NoRust`

Update:

```text
crates/game-cli/src/templates.rs
```

Add:

```rust
pub enum DemoTemplate {
    NoRust,
    Simple,
    DataDrivenLegacy,
}
```

Better naming:

- `NoRust` for primary.
- `SimpleRust` for secondary beginner Rust.
- `DataDrivenRustLegacy` if the old data-driven Rust wrapper remains.

CLI examples:

```bash
game-dev new my-game
game-dev new my-game --template no-rust
game-dev new my-rust-game --template rust-simple
```

The default should become no-Rust once the player/CLI package exists.

Before that point, the default may remain `simple` with a clear warning in the roadmap.

### 5.3 Generate a root `game.toml`

No-Rust template `game.toml` should be small and readable.

Do not make the first file a giant feature showcase. It should cover:

- title,
- assets,
- top-down controls,
- player,
- enemy,
- pickup,
- text map,
- common rules.

Bigger examples should live under examples.

### 5.4 Update template README

`templates/no-rust-demo/README.txt` should say:

```text
1. Open game.toml in any text editor.
2. Run ./game-dev check.
3. Run ./game-dev preview.
4. Press F5 or use preview --watch after edits.
5. No Rust or Cargo is needed for this template.
```

Do not mention `cargo run` in the primary template README except in an “engine developer” appendix outside the template.

### 5.5 Keep secondary Rust templates separate

Keep:

```text
templates/simple-demo
templates/data-driven-demo
```

but relabel:

- `simple-demo` => secondary Rust template.
- `data-driven-demo` => legacy/transitional Rust wrapper or eventually remove after migration.

Do not let old templates be named as the primary path.

## Phase 5 validation

Add architecture tests:

- `templates/no-rust-demo` contains no `.rs`.
- `templates/no-rust-demo` contains no `Cargo.toml`.
- `templates/no-rust-demo` contains no `build.rs`.
- `templates/no-rust-demo/game.toml` contains no forbidden Rust/RON tokens.
- `game-dev new --template no-rust` produces no Rust files.
- generated no-Rust package validates with `game-dev check` without running `cargo`.

---

# Phase 6 — Add a prebuilt no-Rust player executable

## Goal

A user should be able to run a no-Rust project through a prebuilt executable that loads `game.toml`.

## New binary

Add a new binary crate or binary target, for example:

```text
bin/game-player/
  Cargo.toml
  src/main.rs
```

or:

```text
bin/game/src/bin/game-player.rs
```

Prefer a separate package if release packaging is cleaner.

Suggested package name:

```text
game-player
```

Purpose:

- Load a declarative no-Rust project.
- Own runtime setup.
- Call into `game_runtime`.
- Use `game_kit::app::plugin_fn` internally.
- Load `game.toml` through the primary authoring loader.
- Never require user Rust code.

## CLI interface

Minimum:

```bash
game-player
game-player --project .
game-player --file game.toml
game-player --assets assets
game-player --smoke-frames 120
```

Environment compatibility:

```bash
GAME_PROJECT_DIR=.
GAME_FILE=game.toml
GAME_ASSET_DIR=assets
GAME_SMOKE_FRAMES=120
```

But prefer explicit flags in docs.

## Implementation shape

Pseudo-code:

```rust
fn main() -> anyhow::Result<()> {
    let options = parse_args()?;
    configure_asset_roots(&options)?;

    let config = RuntimeConfig::default()
        .title(options.title_override_or_file_title())
        .command_error_policy(CommandErrorPolicy::StoreResource);

    game_runtime::run(config, plugin_fn(|game| {
        game.load_authoring_file(&options.game_file)?;
        Ok(())
    }))
}
```

This binary is the proof that the primary surface does not require user Rust.

## Runtime config from data

Add optional metadata in `game.toml`:

```toml
[game]
title = "My Game"
window_width = 1280
window_height = 720
sim_hz = 120
```

Support:

- title,
- window width/height,
- optional sim hz if desired,
- maybe debug overlay default.

Do not expose renderer/backend config to primary files unless absolutely necessary.

## Smoke-frame support

Current runtime already supports `GAME_SMOKE_FRAMES`. Ensure `game-player` supports it.

Add CI:

```bash
GAME_SMOKE_FRAMES=60 game-player --project examples/no-rust-demo
```

or with Xvfb/lavapipe in graphical CI.

## Phase 6 validation

Add tests:

- `game-player --help` works.
- `game-player --project templates/no-rust-demo --smoke-frames 1` can be run in CI with software Vulkan.
- `game-player` does not import any content crate.
- `bin/game-player/Cargo.toml` depends on runtime and game-kit/starter, but no game-specific content crates.
- release build includes `game-player`.

---

# Phase 7 — Make `game-dev` work without Rust for the primary path

## Goal

The CLI should support a no-Rust project without calling Cargo or requiring a Rust installation.

## Current issue

`game-dev check` currently validates assets/data and then runs `cargo check`. That is correct for Rust starter projects, but wrong for the primary no-Rust path.

`game-dev package` currently builds a release binary from the current Rust project. That is also wrong for the primary no-Rust path.

## Commands to support

### Primary commands

```bash
game-dev new my-game
game-dev new my-game --template no-rust
game-dev check
game-dev preview
game-dev preview --watch
game-dev package --out dist/my-game --zip
game-dev migrate-ron assets/game.ron --out game.toml
```

### Secondary Rust commands

```bash
game-dev new my-rust-game --template rust-simple
game-dev check-rust
game-dev package-rust --release --out dist/my-rust-game --zip
```

or keep auto-detection:

- If `game.toml` and no `Cargo.toml`: no-Rust behavior.
- If `Cargo.toml`: existing Rust behavior.

But docs must make it clear which one is primary.

## Steps

### 7.1 Add project-kind detection

Use the `ProjectKind` from Phase 2.

`game-dev check` behavior:

- `NoRustPackage`: validate `game.toml`, assets, maps, animation metadata; do **not** run cargo.
- `RustStarterProject`: current behavior, including cargo check.
- `WorkspaceDemo`: current behavior for engine developers.

### 7.2 Fix asset root resolution

Current `asset-check` does:

```rust
assets::validate_assets_dir(&std::env::current_dir()?.join("assets"), false)?;
```

Replace with shared path resolution:

```rust
let project = env::current_dir()?;
let paths = resolve_project_paths(&project, cli_options)?;
validate_assets_dir(&paths.asset_dir, false)?;
```

Requirements:

- `game-dev check`
- `game-dev asset-check`
- `game-dev validate-data`
- `game-dev preview`
- `game-dev package`

must all agree on the same project root and asset root.

### 7.3 Add no-Rust validate-data behavior

Rename or supplement:

```bash
game-dev validate-data
```

Primary default:

```text
game.toml
```

Legacy support:

```bash
game-dev validate-data assets/game.ron --legacy
```

or auto-detect `.ron` as legacy with a warning.

### 7.4 Add preview command

`game-dev preview` should find and launch `game-player`.

Resolution order:

1. explicit `--player <path>`,
2. sibling executable next to `game-dev`,
3. `GAME_PLAYER` environment variable,
4. engine developer fallback: `cargo run -p game-player -- ...` only when inside the workspace and Rust exists.

For primary packaged use, docs should use only path 1 or 2.

### 7.5 Add watch mode

`game-dev preview --watch` should:

- start `game-player`,
- watch `game.toml`, maps, animation metadata, and asset directories,
- restart the child process after changes,
- debounce rapid saves,
- print friendly status:
  - “changed game.toml; restarting preview”
  - “changed assets/maps/level-1.txt; restarting preview”

This is the simplest way to support structural changes without compilation. It also avoids making the runtime fully hot-swappable too early.

### 7.6 Add no-Rust package command

For a no-Rust package, `game-dev package` should:

- validate `game.toml`,
- validate assets,
- copy `game-player`,
- copy `game-dev` if useful,
- copy `game.toml`,
- copy `assets/`,
- copy launch scripts,
- write `README.txt`,
- optionally zip.

It should not run `cargo build`.

### 7.7 Forward optional audio features

Current `game-cli/Cargo.toml` forwards only:

```toml
ci-build-sdl3 = ["game-audio/ci-build-sdl3"]
```

Add forwarding for optional audio validation features:

```toml
ogg = ["game-audio/ogg"]
mp3 = ["game-audio/mp3"]
```

If `game-player` has matching feature flags, forward them there too.

### 7.8 Improve unknown asset handling with ignore support

Current asset validation rejects unknown files, which is good. It says “add an explicit ignore rule when ignore support exists,” but that ignore support does not exist yet.

Add one of:

```text
.gameignore
assets/.gameignore
```

or a config section:

```toml
[asset_check]
ignore = ["notes.txt", "source/*.aseprite"]
```

For primary no-Rust users, a config section is probably friendlier.

Do not silently ignore unknown files.

## Phase 7 validation

Add tests:

- `game-dev check` on no-Rust package does not invoke `cargo`.
- `game-dev check` on Rust template still invokes `cargo`.
- `game-dev asset-check` respects `--project`, `--assets`, and default no-Rust layout.
- `game-dev validate-data` defaults to `game.toml`.
- unsupported `.ogg`/`.mp3` diagnostics reflect enabled/disabled codec features correctly.
- `game-dev package` for no-Rust package creates a runnable folder without compiling.

---

# Phase 8 — Reload and preview without build

## Goal

Edits to primary authoring files must be visible through load/reload/preview, not compilation.

## Current state

Current F5 reload supports partial `game.ron` reload:

- existing values can change,
- some map/file values can reload,
- structural list changes require restart.

That is acceptable for the legacy RON path but not enough as the only primary workflow.

## Target behavior

For primary no-Rust packages:

1. Small value edits should reload in-process where already supported.
2. Text-map edits should reload in-process where already supported.
3. Structural changes should be reflected by a prebuilt preview restart, not a Rust build.
4. The user-facing tool should make this obvious.

## Steps

### 8.1 Rename reload diagnostics away from `game.ron`

Current debug overlay says things like:

```text
game.ron reload:
game.ron error:
```

Change to format-neutral wording:

```text
game file reload:
game file error:
```

or:

```text
game.toml reload:
game.toml error:
```

Use the actual loaded filename.

Affected files include:

- `crates/game-kit/src/beginner/debug.rs`
- `crates/game-kit/src/context.rs`
- `crates/game-kit/src/data/tests.rs`
- docs/tutorials

### 8.2 Make `BeginnerFileRuntime` format-neutral

Rename internally if practical:

```rust
BeginnerFileRuntime -> AuthoringFileRuntime
BeginnerReloadIdentity -> AuthoringReloadIdentity
BeginnerReloadLevel -> AuthoringReloadLevel
```

This is optional if too invasive, but docs and user diagnostics should not say “beginner RON file.”

### 8.3 Add watch-preview restart path

Implement `game-dev preview --watch` before attempting full in-process structural reload.

This satisfies the requirement that edits reflect without compilation.

### 8.4 Define structural reload contract

Document:

- F5 reloads value and map edits inside the running game.
- `game-dev preview --watch` restarts the prebuilt player for structural edits.
- No Rust build is involved.
- Adding/removing prefabs/maps/rules is supported by preview restart.
- Running packaged player normally reloads on launch.

### 8.5 Optional later: full in-process authoring reload

Only after the prebuilt restart path works, consider full internal reload.

Hard part:

- rules currently install systems/schedule behavior at setup,
- changing enabled rules may require rebuilding schedule/runtime content,
- runtime currently owns schedule and active world.

A future full reload could:

- rebuild a new `GameBuilder` from `game.toml`,
- replace schedule/content registries safely at a frame boundary,
- clear world and respawn current/start map,
- preserve renderer/audio/platform backends,
- expose structured reload errors.

Do not block MVP on this if `preview --watch` provides no-build iteration.

## Phase 8 validation

- Edit `game.toml`; `preview --watch` restarts and picks up change.
- Edit text map; `preview --watch` restarts and picks up change.
- Press F5 in debug player; value/map reload diagnostics do not mention RON unless a RON file is actually loaded.
- Structural edits require no compilation.

---

# Phase 9 — Structured diagnostics and transactional map changes

## Goal

Make failures visible and prevent partial state corruption, especially because no-Rust users cannot debug through Rust internals.

## 9A — Route beginner spawn errors into structured diagnostics

### Current issue

`crates/game-kit/src/context.rs` contains `BeginnerSpawnQueue`. Its drain path currently logs failures such as failed prefab spawns. That means:

- runtime command errors are structured,
- beginner-layer spawn errors are only logs,
- tests/debug UI may miss them.

### Steps

#### 9A.1 Add an authoring diagnostics resource

Possible location:

```text
crates/game-core/src/commands.rs
```

or:

```text
crates/game-core/src/diagnostics.rs
```

Suggested type:

```rust
pub enum RuntimeDiagnosticKind {
    Command,
    AuthoringSpawn,
    MapTransition,
    DataReload,
    AssetReload,
}

pub struct RuntimeDiagnostic {
    pub kind: RuntimeDiagnosticKind,
    pub message: String,
}

pub struct RuntimeDiagnostics {
    diagnostics: Vec<RuntimeDiagnostic>,
}
```

Alternative: extend `CommandErrorKind` with:

```rust
BeginnerSpawnPrefab
MapTransition
AuthoringData
```

Use whichever keeps layering clean. Since `game-core` cannot depend on `game-kit`, the kind names must stay generic enough.

#### 9A.2 Store beginner spawn failures

In `drain_beginner_spawn_queue`, replace log-only behavior with:

- log error,
- push structured diagnostic/error resource,
- optionally respect a strict policy in tests.

#### 9A.3 Surface in debug overlay

Update beginner debug overlay to show:

```text
authoring error: failed to spawn prefab "slime": missing collider ...
```

Keep it beginner-readable. Do not show backtraces or Rust type IDs by default.

#### 9A.4 Add tests

Add tests that:

- queue an invalid beginner spawn,
- drain the queue,
- assert `RuntimeDiagnostics` or `CommandErrors` contains the failure,
- assert debug overlay can display it,
- assert strict test harness can fail on it.

## 9B — Make map switching transactional

### Current issue

`crates/game-kit/src/map.rs` has:

```rust
content.current_map = map_name;
clear_world_for_map_respawn(world);
let result = content.spawn_current(world).map(|()| map_id);
world.insert_resource(content);
result
```

If spawning fails after the clear, the world may already be partially changed.

### Steps

#### 9B.1 Add preflight validation for target map

Before clearing the real world:

- resolve map name,
- resolve map ID,
- get `GameMap`,
- validate prefab references,
- validate map-required objects,
- validate collision requirements as far as possible.

Some validation already happens at setup. This preflight is still useful because custom prefab data or runtime-loaded changes may fail.

#### 9B.2 Add staging spawn plan

Do not mutate real world until the transition is known-good.

Options:

1. **Spawn into a scratch world**
   - Create a new `World`.
   - Insert only resources needed for prefab spawn validation if any.
   - Spawn map objects into scratch world.
   - Validate components.
   - Then clear real world and spawn for real.

2. **Build a spawn plan**
   - Resolve prefab IDs and object positions/properties into a vector.
   - Validate all references before clearing.
   - Then clear and spawn.
   - This is less strong than scratch world if prefab spawn closures can fail after mutation.

Preferred: scratch world if current prefab spawns do not depend heavily on resources. If scratch world lacks required resources, use spawn plan plus improved prefab validation.

#### 9B.3 Commit only after success

Transaction order:

1. remove/borrow content runtime,
2. validate target,
3. preflight spawn,
4. clear command queue,
5. clear world,
6. spawn into real world,
7. update `current_map`,
8. insert content runtime,
9. queue runtime active-map change.

If real spawn can still fail after preflight, record structured diagnostic and keep previous map if possible. Avoid setting `current_map` before the commit point.

#### 9B.4 Add rollback tests

Tests should cover:

- unknown map leaves current map unchanged,
- spawn failure leaves current map unchanged,
- spawn failure records structured diagnostic,
- successful switch updates current map and queues active map command,
- command queue from old world is cleared on successful switch.

### Phase 9 validation

```bash
cargo test -p game-kit map_flow --locked
cargo test -p game-kit context --locked
cargo test -p game-runtime --locked
```

---

# Phase 10 — Convert examples to prove the no-Rust path

## Goal

The repository should include no-Rust examples that are not Rust crates.

## Current issue

Examples like `examples/data-driven-full-demo` still contain Rust `src/main.rs` wrappers that call `load_beginner_file("assets/game.ron")`.

That proves the data loader works, but it does not prove the primary objective.

## Steps

### 10.1 Add no-Rust example packages

Create examples such as:

```text
examples/no-rust-minimal/
  game.toml
  assets/maps/level-1.txt
  assets/textures/...
  assets/sounds/...

examples/no-rust-full/
  game.toml
  assets/maps/...
  assets/animations/player.toml
  assets/textures/...
  assets/sounds/...

examples/no-rust-tiled/
  game.toml
  assets/maps/tiled-demo.tmx
  assets/textures/...
```

These should contain no `Cargo.toml`, no `src`, and no `build.rs`.

### 10.2 Migrate current data-driven examples

For each current data-driven Rust example:

- `examples/data-driven-events-demo`
- `examples/data-driven-waves-demo`
- `examples/data-driven-projectiles-demo`
- `examples/data-driven-full-demo`
- `examples/data-driven-tiled-demo`

Create a no-Rust equivalent.

Do not necessarily delete old examples immediately. Instead:

- mark old examples as legacy Rust-wrapper examples,
- point docs and CI to the new no-Rust packages,
- remove old examples from “primary path” documentation.

### 10.3 Replace primary docs with no-Rust examples

Update docs so the first path is:

```bash
game-dev new my-game
cd my-game
game-dev check
game-dev preview
```

not:

```bash
cargo run
```

### 10.4 Keep Rust examples as secondary

Move Rust examples under a clear heading:

```text
Secondary Rust examples
```

The examples can stay in the workspace because they are useful for engine developers, but they should not be the acceptance proof for the primary path.

## Phase 10 validation

Add tests:

- every `examples/no-rust-*` package contains `game.toml`,
- no `examples/no-rust-*` package contains `.rs`, `Cargo.toml`, or `build.rs`,
- every no-Rust example validates with `game-dev check --project`,
- every no-Rust example can smoke-run with `game-player --project`.

---

# Phase 11 — Mechanical enforcement of the primary surface

## Goal

Make it impossible to accidentally reintroduce Rust-shaped primary authoring.

## New architecture test file

Add:

```text
crates/game-core/tests/architecture_no_rust_authoring.rs
```

or split into:

```text
architecture_no_rust_files.rs
architecture_no_rust_docs.rs
architecture_no_rust_templates.rs
```

## 11.1 Define primary authoring corpus

Add to `tests/support/mod.rs`:

```rust
pub(crate) const PRIMARY_NO_RUST_PATHS: &[&str] = &[
    "templates/no-rust-demo",
    "examples/no-rust-minimal",
    "examples/no-rust-full",
    "examples/no-rust-tiled",
];
```

Add primary docs list:

```rust
pub(crate) const PRIMARY_NO_RUST_DOCS: &[&str] = &[
    "README.md",
    "docs/api-boundary.md",
    "docs/no-rust-package-layout.md",
    "docs/beginner-authoring.md",
    "docs/content-authoring.md",
];
```

## 11.2 Forbid Rust project files in primary packages

Test:

- no `Cargo.toml`,
- no `Cargo.lock`,
- no `build.rs`,
- no `src/*.rs`,
- no `*.rs`.

## 11.3 Forbid Rust/RON syntax in primary data files

Forbidden tokens for primary data:

```text
Some(
None
Ok(
Err(
Result
Vec
HashMap
BTreeMap
Player((
Enemy((
Pickup((
Door((
Projectile((
Spawner((
Trigger((
Checkpoint((
TextMap((
TextMapAuto((
Tiled((
Ldtk((
TopDownControls
PlayerCollectsPickups
EnemiesDamagePlayer
CameraFollowsPlayer
ShowScore
ShowPlayerHealth
::
=> 
fn 
impl 
struct 
enum 
trait 
pub 
use 
match 
<
>
```

Be careful with `<` and `>` because docs may contain HTML or key hints. For data files, they are safe to forbid.

## 11.4 Forbid engine vocabulary in primary docs/examples

Forbidden primary vocabulary:

```text
GameCtx
StartupGameCtx
EntityId
Component
World
Transform
Velocity
Sprite::new
Collider::box_of
CommandQueue
RuntimeConfig
game_runtime
game_core
game_renderer_vulkan
game_platform_sdl
ash
sdl3
swapchain
descriptor
allocator
lifetime
generic
trait
cargo run
cargo check
rustc
```

Some docs may need to mention Rust in a clearly marked secondary section. If tests scan entire docs, use one of these strategies:

1. Split docs:
   - `docs/no-rust-authoring.md`
   - `docs/rust-authoring.md`
   - scan only no-Rust docs strictly.

2. Mark sections:
   - scan content between `<!-- primary-no-rust:start -->` and `<!-- primary-no-rust:end -->`.

Prefer separate docs for simpler enforcement.

## 11.5 Add a CLI scanner

Add:

```bash
game-dev authoring-scan --project .
```

It should run the same forbidden-token checks outside the Rust test harness.

This lets release packages and external projects validate their primary surface.

## 11.6 Add docs enforcement

Tests should assert:

- primary docs mention `game.toml`,
- primary docs mention `game-dev preview`,
- primary docs mention prebuilt executable,
- primary docs do not describe `assets/game.ron` as the current default,
- primary docs do not start with `cargo run`.

## Phase 11 validation

```bash
cargo test -p game-core architecture_no_rust --locked
cargo run -p game-cli -- authoring-scan --project templates/no-rust-demo
```

---

# Phase 12 — Migrate RON to legacy compatibility

## Goal

Keep existing RON users working temporarily while removing RON from the official primary path.

## Steps

### 12.1 Rename docs language

Every RON mention should be one of:

- legacy,
- migration,
- advanced,
- internal test fixture,
- historical roadmap.

No public “start here” path should point to RON.

### 12.2 Add `game-dev migrate-ron`

Implement:

```bash
game-dev migrate-ron assets/game.ron --out game.toml
```

This should:

1. parse existing RON through the legacy parser,
2. normalize into the internal model,
3. emit canonical TOML,
4. run validation on the TOML output,
5. print any lossy/unsupported notes.

### 12.3 Add migration docs

Create:

```text
docs/migrations/ron-to-toml.md
```

Include examples:

RON:

```ron
Player((
    name: "player",
    melee: Some((range: 30.0, damage: 25)),
))
```

TOML:

```toml
[[prefab]]
kind = "player"
name = "player"

[prefab.melee]
range = 30
damage = 25
```

### 12.4 Move RON examples out of primary docs

Keep a small legacy example if useful:

```text
examples/legacy-ron-data-demo
```

But do not include it in primary tutorial sequence.

### 12.5 Optional feature gate

Eventually consider:

```toml
[features]
legacy-ron = ["ron"]
default = ["legacy-ron"] # during migration
```

Later:

```toml
default = []
```

Do not do this until migration docs and TOML parity are strong.

## Phase 12 validation

- `game-dev migrate-ron` converts existing template/demo RON.
- converted TOML validates.
- primary docs do not point to RON.
- RON tests are labeled legacy.
- architecture tests allow RON only in legacy paths.

---

# Phase 13 — Convert animation and tuning metadata away from RON for primary use

## Goal

The primary surface must not use RON anywhere, including auxiliary metadata.

## Current issue

Current docs and code use:

```text
assets/animations/player.ron
tuning/game.ron
game-map RON maps
```

Even if `game.toml` is TOML, these files would still expose Rust-shaped syntax.

## Steps

### 13.1 Add TOML animation metadata

Current animation metadata is loaded in `crates/game-kit/src/assets.rs` through RON.

Add TOML support:

```text
assets/animations/player.toml
```

Example:

```toml
texture = "textures/player-sheet.png"
columns = 4
rows = 4

[[clip]]
name = "idle"
frames = [0, 1]
fps = 4

[[clip]]
name = "walk"
frames = [4, 5, 6, 7]
fps = 8
```

Avoid Rust-shaped frame constructors.

### 13.2 Update asset validation

`game-dev asset-check` should validate:

- `assets/animations/*.toml` as primary,
- `assets/animations/*.ron` as legacy with warning or legacy allowance,
- unknown `.ron` outside legacy should fail in primary packages.

### 13.3 Add TOML tuning metadata or fold tuning into `game.toml`

Current tuning file is RON. Choose one:

1. Add `tuning.toml`.
2. Add a `[tuning]` section in `game.toml`.

For no-Rust primary, the second is simpler unless tuning needs separate live reload.

Example:

```toml
[tuning]
slime_health = 30
player_speed = 130
```

or:

```toml
[tuning."slime.health"]
value = 30
```

Prefer readable keys over Rust-like dotted component names in beginner docs.

### 13.4 Mark RON maps as advanced/legacy

`game-map` RON map loading can stay for advanced users/tests, but it must not be part of the primary no-Rust path.

Primary maps should be:

- text maps,
- Tiled,
- LDtk,
- future editor formats,
- TOML only for small metadata, not Rust-shaped RON.

## Phase 13 validation

- primary no-Rust examples use `*.toml` for animation metadata,
- no primary examples contain `.ron`,
- asset-check validates animation TOML,
- docs do not teach animation RON in primary path.

---

# Phase 14 — Release/package the no-Rust SDK

## Goal

Ship a release artifact that proves the user can author without Rust.

## Current release model

`docs/distribution-policy.md` says current releases attach demo zips. That is good for trying the bundled demo, but not enough for no-Rust authoring.

## Target release artifacts

For each supported platform:

```text
game-sdk-linux-x86_64.zip
game-sdk-windows-x86_64.zip
```

Contents:

```text
game-player
game-dev
templates/no-rust-demo/
README.txt
LICENSE
THIRD_PARTY_NOTICES.md
```

Optional:

```text
examples/no-rust-minimal/
examples/no-rust-full/
```

## Steps

### 14.1 Update release workflow

Update:

```text
.github/workflows/release.yml
```

Add packaging steps for:

- `game-player`,
- `game-dev`,
- no-Rust template,
- launcher scripts,
- README.

### 14.2 Add verification scripts

Add or extend:

```text
scripts/verify-release-artifact.sh
scripts/verify-github-release-artifacts.sh
```

Verify:

- zip contains `game-player`,
- zip contains `game-dev`,
- zip contains no-Rust template,
- no-Rust template has `game.toml`,
- no-Rust template has no Rust project files,
- launch scripts exist,
- README says no Rust required.

### 14.3 Add local SDK dry-run

Document:

```bash
cargo run -p xtask --features ci-build-sdl3 -- package-sdk --release --out /tmp/game-sdk
/tmp/game-sdk/game-dev new /tmp/my-game --template no-rust
cd /tmp/my-game
/path/to/game-sdk/game-dev check
/path/to/game-sdk/game-dev preview --smoke-frames 60
```

### 14.4 Add CI job for SDK artifact

CI should build SDK and test a no-Rust project from it.

Important: the test should not call `cargo` after the SDK is built.

Pseudo-flow:

```bash
cargo run -p xtask -- package-sdk --release --out /tmp/sdk
/tmp/sdk/game-dev new /tmp/no-rust-smoke
cd /tmp/no-rust-smoke
/tmp/sdk/game-dev check
GAME_SMOKE_FRAMES=60 /tmp/sdk/game-player --project .
```

The last command may need Xvfb/lavapipe, same as current smoke tests.

## Phase 14 validation

- local SDK dry-run passes,
- CI SDK job passes,
- release artifact verifier passes,
- docs tell non-Rust users to download SDK zip, not install Rust.

---

# Phase 15 — Preserve and discipline the secondary Rust APIs

## Goal

Keep the existing Rust APIs useful without letting them redefine the primary objective.

## Steps

### 15.1 Keep beginner Rust API stable

Do not delete:

- `game_starter::prelude::*`
- `game_kit::beginner::prelude::*`
- `content_plugin!`

These are valuable for users who choose Rust.

### 15.2 Keep advanced API clearly labeled

Do not delete:

- `game_kit::advanced::prelude::*`
- advanced tests,
- `testbed-content`

But docs must say:

```text
Advanced Rust authoring is not the primary no-Rust surface.
```

### 15.3 Add compatibility removal plan

Current `game_kit::prelude::*` compatibility is deprecated but still present.

Add or update a policy:

```text
v0.2.x: compatibility prelude exists but deprecated.
v0.3.x: docs/examples/templates must not use it.
v0.4.x or pre-1.0: remove or feature-gate compatibility prelude.
```

### 15.4 Keep root exports narrow

Current `game-kit` and `game-core` root exports are much improved. Keep architecture tests for:

- no broad root prelude usage,
- beginner docs use correct imports,
- advanced docs are explicit,
- content crates do not import backend crates.

### 15.5 Do not put no-Rust schema types in beginner prelude

The primary no-Rust surface should not require importing Rust schema types. Avoid exporting TOML schema structs through `game_kit::beginner::prelude::*` unless tests genuinely need them.

If Rust users need data schema types, put them under:

```rust
game_kit::data
```

not the beginner prelude.

## Phase 15 validation

- current Rust examples still compile,
- no primary docs use Rust APIs,
- no no-Rust templates include Rust files,
- advanced docs/examples remain explicitly advanced.

---

# Phase 16 — Keep module size and maintainability under control

## Goal

The no-Rust work will add parser, CLI, packaging, and enforcement code. Do not let large modules become unmaintainable again.

## Current large files to watch

From static inspection of the uploaded zip, large files include:

- `docs/roadmaps/content-engine-boundary-consolidation.md`
- `crates/game-audio/src/mixer/mod.rs`
- `crates/game-kit/src/app/mod.rs`
- `crates/game-kit/src/context.rs`
- `crates/game-kit/src/data/tests.rs`
- `crates/game-kit/src/assets.rs`
- `crates/game-kit/src/map.rs`
- `crates/game-kit/src/data/validate.rs`
- `crates/game-kit/src/data/mod.rs`
- `crates/game-core/tests/architecture_beginner_surface.rs`
- `crates/game-runtime/src/runner.rs`

The current sizes are not catastrophic, but the no-Rust work should not create new monoliths.

## Steps

### 16.1 Split data tests before adding many TOML tests

Current `crates/game-kit/src/data/tests.rs` is large. Before adding lots of TOML tests, split into focused modules.

Suggested:

```text
crates/game-kit/src/data/tests/
  mod.rs
  toml_primary.rs
  ron_legacy.rs
  validation.rs
  reload.rs
  effects.rs
  diagnostics.rs
```

### 16.2 Split CLI no-Rust commands

Do not add all new CLI logic to `crates/game-cli/src/lib.rs`.

Suggested modules:

```text
crates/game-cli/src/commands/
  new.rs
  check.rs
  preview.rs
  package.rs
  validate_data.rs
  asset_check.rs
  migrate_ron.rs
  authoring_scan.rs
  package_sdk.rs

crates/game-cli/src/project.rs
crates/game-cli/src/no_rust.rs
crates/game-cli/src/starter_assets.rs
```

### 16.3 Split data parser modules

Suggested:

```text
crates/game-kit/src/data/
  mod.rs
  model.rs
  parse.rs
  toml_schema.rs
  toml_parse.rs
  ron_legacy.rs
  validate.rs
  build.rs
  reload.rs
  diagnostics.rs
  emit_toml.rs
```

### 16.4 Add size guard tests

Update architecture API surface tests:

- data modules max ~1,500 lines,
- CLI command modules max ~1,000 lines,
- architecture test files max ~1,000 lines,
- exceptions must be documented.

Do not obsess over line counts, but use them to stop another accidental monolith.

---

# Phase 17 — Final acceptance gates

## Goal

Prove the primary objective end to end.

## Required final checks

### 17.1 Source checks

```bash
cargo fmt --all -- --check
cargo test --workspace --locked --features game/ci-build-sdl3
cargo test -p game-runtime --test headless_runner --no-default-features --locked
cargo clippy --workspace --all-targets --locked --features game/ci-build-sdl3 -- -D warnings
cargo build -p game-player --release --locked --features ci-build-sdl3
cargo build -p game-cli --release --locked --features ci-build-sdl3
```

### 17.2 Primary no-Rust checks

```bash
target/release/game-dev new /tmp/no-rust-smoke --template no-rust
cd /tmp/no-rust-smoke
/path/to/target/release/game-dev check
GAME_SMOKE_FRAMES=60 /path/to/target/release/game-player --project .
/path/to/target/release/game-dev package --out /tmp/no-rust-package --zip
```

None of those commands should invoke `cargo` after the tools have been built.

### 17.3 Authoring surface scans

```bash
cargo test -p game-core architecture_no_rust --locked
target/release/game-dev authoring-scan --project /tmp/no-rust-smoke
target/release/game-dev authoring-scan --project examples/no-rust-full
```

### 17.4 Release artifact checks

```bash
cargo run -p xtask --features ci-build-sdl3 -- package-sdk --release --out /tmp/game-sdk
scripts/verify-release-artifact.sh /tmp/game-sdk-linux-x86_64.zip linux
```

Then from inside extracted SDK:

```bash
./game-dev new /tmp/sdk-smoke
cd /tmp/sdk-smoke
/path/to/sdk/game-dev check
GAME_SMOKE_FRAMES=60 /path/to/sdk/game-player --project .
```

### 17.5 Manual no-Rust edit check

Perform this by hand once before claiming done:

1. Open generated `game.toml` in a plain text editor.
2. Change player speed.
3. Run preview.
4. Change map layout.
5. Run preview.
6. Add a pickup.
7. Run preview.
8. Intentionally typo an asset filename.
9. Confirm `game-dev check` gives a beginner-readable error.
10. Confirm no command required `cargo`.

---

# Suggested commit sequence

This sequence is designed for a coding agent to follow safely.

1. `docs: add primary no-rust authoring roadmap`
2. `docs: clarify no-rust primary surface versus rust authoring tiers`
3. `cli: add project kind and no-rust path resolution`
4. `data: introduce format-neutral authoring model`
5. `data: add toml parser for primary game config`
6. `data: preserve ron as legacy authoring parser`
7. `data: add toml emitter and ron migration support`
8. `kit: add format-neutral load and validate authoring file APIs`
9. `templates: add no-rust demo template without cargo files`
10. `cli: generate no-rust template and starter assets`
11. `player: add prebuilt game-player binary`
12. `cli: add no-rust check and validate-data defaults`
13. `cli: add preview and preview-watch commands`
14. `cli: add no-rust package command`
15. `assets: support primary toml animation metadata`
16. `docs: migrate primary authoring docs from ron to toml`
17. `examples: add no-rust example packages`
18. `tests: enforce no-rust primary authoring surface`
19. `runtime: route beginner spawn failures to structured diagnostics`
20. `kit: make map transitions transactional`
21. `ci: smoke-run no-rust package through game-player`
22. `release: package no-rust sdk artifacts`
23. `docs: add ron-to-toml migration guide`
24. `cleanup: split expanded data/cli test modules`
25. `release: update checklist and distribution policy`

---

# Non-goals during this roadmap

Do not do these until the roadmap is complete:

- Build a larger actual game.
- Add new enemy types just for gameplay.
- Add more combat systems unless needed by the data schema already present.
- Rewrite the ECS.
- Rewrite the Vulkan renderer.
- Replace SDL.
- Add a visual editor.
- Add Lua/Rhai scripting as a shortcut around the declarative data goal.
- Treat ergonomic Rust builder code as the primary solution.
- Treat RON as acceptable primary no-Rust syntax.
- Require users to install Rust for the primary path.

---

# Final milestone statement

The foundation is complete when a non-programmer can receive a zip containing `game-player`, `game-dev`, and a no-Rust template; open `game.toml` in a plain text editor; edit the player, enemies, maps, scenes, sounds, and rules; run `game-dev check`; run `game-dev preview`; package the result; and never see, write, compile, or understand Rust.

At that point, the Rust engine has successfully become the hidden technical layer, and the project can justifiably move on to using the foundation to build a small game.

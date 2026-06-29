use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[test]
fn architecture_docs_name_the_current_beginner_surface() {
    for relative in [
        "README.md",
        "docs/ARCHITECTURE.md",
        "docs/beginner-authoring.md",
        "docs/content-authoring.md",
    ] {
        let source = fs::read_to_string(workspace_root().join(relative))
            .unwrap_or_else(|err| panic!("failed to read {relative}: {err}"));
        assert!(
            source.contains("game_kit::beginner::prelude::*")
                || source.contains("game_starter::prelude::*"),
            "{relative} should teach an explicit beginner prelude"
        );
    }

    let readme =
        fs::read_to_string(workspace_root().join("README.md")).expect("failed to read README");
    for required in [
        "game_starter::prelude::*",
        "game_kit::beginner::prelude::*",
        "testbed-content",
        "advanced",
        "gamepad",
    ] {
        assert!(
            readme.contains(required),
            "README should contain {required:?}"
        );
    }
}

#[test]
fn docs_do_not_describe_implemented_beginner_features_as_future_work() {
    for relative in ["docs/future-editor-import.md", "docs/dead-code-audit.md"] {
        let source = fs::read_to_string(workspace_root().join(relative))
            .unwrap_or_else(|err| panic!("failed to read {relative}: {err}"));

        for forbidden in [
            "LDtk (`.ldtk`) import are intentionally future work",
            "additional registered maps and runtime map switching are future work",
            "File-backed sound requests exist in `game-core` but are not exposed",
            "Content crates use `game_kit::prelude::*`",
            "SDL3 window, keyboard input, and a lock-free audio mixer",
        ] {
            assert!(
                !source.contains(forbidden),
                "{relative} contains stale claim: {forbidden}"
            );
        }
    }
}

#[test]
fn game_core_manifest_has_no_backend_dependencies() {
    let manifest = fs::read_to_string(workspace_root().join("crates/game-core/Cargo.toml"))
        .expect("failed to read game-core manifest");

    for forbidden in [
        "ash",
        "sdl3",
        "gpu-allocator",
        "game-renderer-vulkan",
        "game-platform-sdl",
        "game-audio",
    ] {
        assert!(
            !manifest.contains(forbidden),
            "game-core manifest must not depend on {forbidden}"
        );
    }
}

#[test]
fn game_core_source_has_no_backend_imports() {
    let src_dir = workspace_root().join("crates/game-core/src");
    let mut files = Vec::new();
    collect_rust_files(&src_dir, &mut files);

    for path in files {
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        for forbidden in [
            "ash::",
            "sdl3::",
            "gpu_allocator",
            "game_renderer_vulkan",
            "game_platform_sdl",
            "game_audio::AudioSystem",
            "crate::renderer",
            "crate::platform",
            "crate::audio::AudioSystem",
        ] {
            assert!(
                !source.contains(forbidden),
                "{} must not contain backend import/reference {forbidden:?}",
                path.display()
            );
        }
    }
}

#[test]
fn game_kit_has_no_backend_dependencies() {
    // Phase 1: the content-authoring facade wraps the engine-neutral gameplay
    // crates and must never reach a runtime/backend crate.
    let manifest = read_manifest_without_comments("crates/game-kit/Cargo.toml");
    for forbidden in [
        "game-runtime",
        "game-renderer-vulkan",
        "game-platform-sdl",
        "game-audio",
        "ash",
        "sdl3",
        "gpu-allocator",
    ] {
        assert!(
            !manifest.contains(forbidden),
            "game-kit manifest must not depend on {forbidden}"
        );
    }

    let src_dir = workspace_root().join("crates/game-kit/src");
    let mut files = Vec::new();
    collect_rust_files(&src_dir, &mut files);
    for path in files {
        // Strip comment lines: doc comments legitimately mention runtime entry
        // points (e.g. `game_runtime::run`) without creating a dependency.
        let source = read_code_without_comments(&path);
        for forbidden in [
            "game_runtime",
            "game_renderer_vulkan",
            "game_platform_sdl",
            "game_audio",
            "ash::",
            "sdl3::",
            "gpu_allocator",
        ] {
            assert!(
                !source.contains(forbidden),
                "{} must not reference backend {forbidden:?}",
                path.display()
            );
        }
    }
}

#[test]
fn game_starter_is_the_only_beginner_crate_that_depends_on_runtime() {
    let starter_manifest = read_manifest_without_comments("crates/game-starter/Cargo.toml");
    assert!(
        starter_manifest.contains("game-runtime"),
        "game-starter should own the beginner runtime dependency"
    );

    for relative in [
        "crates/game-kit/Cargo.toml",
        "examples/one-file-demo/Cargo.toml",
        "templates/simple-demo/Cargo.toml",
        "templates/data-driven-demo/Cargo.toml",
    ] {
        let manifest = read_manifest_without_comments(relative);
        assert!(
            !manifest.contains("game-runtime"),
            "{relative} must not depend directly on game-runtime"
        );
    }
}

#[test]
fn beginner_demo_and_template_hide_runtime_boot_code() {
    for relative in [
        "examples/one-file-demo/src/main.rs",
        "templates/simple-demo/src/main.rs",
        "templates/data-driven-demo/src/main.rs",
    ] {
        let source = read_code_without_comments(&workspace_root().join(relative));
        assert!(
            source.contains("use game_starter::prelude::*;"),
            "{relative} should import game_starter::prelude::*"
        );
        for forbidden in [
            "RuntimeConfig",
            "game_runtime::run",
            "game_kit::prelude::*",
            "game_kit::advanced::prelude::*",
            "game_kit::plugin_fn",
            "for<'app>",
            "struct Assets",
            "TextureHandle",
            "SoundHandle",
            "AssetAuthor",
            "game.assets(",
            "EntityId",
            "Component",
            "Transform",
            "Velocity",
            "Sprite::new",
            "Collider::box_of",
            "Health::new",
            "MeleeAttack",
            "Faction",
            "AiController",
            "ChaseTarget",
            "PathFollow",
            "Patrol",
            "GameCtx",
            "StartupGameCtx",
            "PrefabAuthor",
            "game.prefab(",
            "fixed_active",
            "fixed_systems_are_pause_guarded",
            "component::<",
            "entities_with::<",
            "for_each",
            "nearest_living_with",
            "spawn_prefab_at",
        ] {
            assert!(
                !source.contains(forbidden),
                "{relative} must not expose beginner boilerplate {forbidden:?}"
            );
        }
        assert!(
            source.contains(".asset_bag()")
                || source.contains(".assets_from_folders()")
                || source.contains("load_beginner_file("),
            "{relative} should use a beginner asset/data registration helper"
        );
    }
}

#[test]
fn every_beginner_demo_and_template_stays_on_the_high_level_surface() {
    let root = workspace_root();
    let paths = [
        "examples/one-file-demo/src",
        "examples/beginner-mini-game/src",
        "examples/coin-collector/src",
        "examples/data-driven-full-demo/src",
        "examples/data-driven-tiled-demo/src",
        "examples/projectile-demo/src",
        "examples/script-like-custom-rules/src",
        "examples/two-level-demo/src",
        "examples/waves-demo/src",
        "examples/menu-game-over/src",
        "examples/no-rust-shapes-demo/src",
        "examples/win-condition-demo/src",
        "examples/enemy-drops-demo/src",
        "examples/health-pickup-demo/src",
        "examples/checkpoint-demo/src",
        "examples/boss-demo/src",
        "examples/dialog-demo/src",
        "examples/inventory-demo/src",
        "examples/title-menu-demo/src",
        "examples/damage-zone-demo/src",
        "examples/ldtk-demo/src",
        "examples/tiled-demo/src",
        "examples/animation-demo/src",
        "examples/audio-demo/src",
        "examples/trigger-area-demo/src",
        "templates/simple-demo/src",
        "templates/data-driven-demo/src",
    ];

    for relative in paths {
        let mut files = Vec::new();
        collect_rust_files(&root.join(relative), &mut files);
        assert!(
            !files.is_empty(),
            "{relative} should contain a beginner source file"
        );

        for path in files {
            let source = read_code_without_comments(&path);
            assert!(
                source.contains("use game_starter::prelude::*;"),
                "{} should use the standalone beginner entry point",
                path.display()
            );
            assert!(
                source.contains(".asset_bag()")
                    || source.contains(".assets_from_folders()")
                    || source.contains("load_beginner_file("),
                "{} should use a beginner asset/data helper",
                path.display()
            );
            for forbidden in BEGINNER_DEMO_FORBIDDEN {
                assert!(
                    !source.contains(forbidden),
                    "{} must not expose beginner implementation detail {forbidden:?}",
                    path.display()
                );
            }
        }
    }
}

#[test]
fn tiled_demo_uses_the_beginner_tiled_import_path() {
    let source =
        read_code_without_comments(&workspace_root().join("examples/tiled-demo/src/main.rs"));
    for required in [
        "use game_starter::prelude::*;",
        "map_from_tiled(",
        ".object(",
        ".simple_theme(",
        ".use_top_down_game()",
    ] {
        assert!(
            source.contains(required),
            "tiled demo should contain {required:?}"
        );
    }
    for forbidden in BEGINNER_DEMO_FORBIDDEN {
        assert!(
            !source.contains(forbidden),
            "tiled demo must not expose beginner implementation detail {forbidden:?}"
        );
    }

    let ci = fs::read_to_string(workspace_root().join(".github/workflows/ci.yml"))
        .expect("failed to read CI workflow");
    assert!(
        ci.contains("cargo run -p tiled-demo --locked --features ci-build-sdl3"),
        "CI should smoke-run tiled-demo"
    );
}

#[test]
fn beginner_facing_sources_hide_context_lifetime_annotations() {
    let root = workspace_root();
    let paths = [
        "examples/one-file-demo",
        "examples/beginner-mini-game",
        "examples/coin-collector",
        "examples/data-driven-full-demo",
        "examples/data-driven-tiled-demo",
        "examples/projectile-demo",
        "examples/script-like-custom-rules",
        "examples/two-level-demo",
        "examples/waves-demo",
        "examples/menu-game-over",
        "examples/no-rust-shapes-demo",
        "examples/animation-demo",
        "templates/simple-demo",
        "templates/data-driven-demo",
        "crates/simple-content/src",
        "crates/arena-content/src",
        "docs/tutorials",
        "docs/cookbook",
    ];

    for relative in paths {
        let mut files = Vec::new();
        collect_beginner_surface_files(&root.join(relative), &mut files);
        for path in files {
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
            for forbidden in [
                "Game<'_",
                "GameCtx<'_",
                "StartupGameCtx<'_",
                ".commands()",
                "CommandQueue",
            ] {
                assert!(
                    !source.contains(forbidden),
                    "{} must not expose beginner context lifetimes {forbidden:?}",
                    path.display()
                );
            }
        }
    }
}

#[test]
fn simple_content_uses_the_beginner_content_plugin_macro() {
    let path = workspace_root().join("crates/simple-content/src/lib.rs");
    let source = read_code_without_comments(&path);

    assert!(
        source.contains("content_plugin!(SimplePlugin, plugin"),
        "simple-content should define its plugin with content_plugin!"
    );
    for forbidden in ["impl GamePlugin", "GameApp<'_", "pub struct SimplePlugin;"] {
        assert!(
            !source.contains(forbidden),
            "simple-content should hide plugin boilerplate {forbidden:?}"
        );
    }
}

#[test]
fn final_no_rust_shapes_demo_stays_a_complete_high_level_acceptance_example() {
    let path = workspace_root().join("examples/no-rust-shapes-demo/src/main.rs");
    let source = read_code_without_comments(&path);

    for required in [
        "use game_starter::prelude::*;",
        "assets_from_folders()",
        "map_from_text_auto",
        "player_prefab",
        "enemy_prefab",
        "pickup_prefab",
        "projectile_prefab",
        "trigger_prefab",
        "door_prefab",
        "use_simple_scene_flow",
        "play_music",
        "play_sound",
        "enemy_drops",
        "heal_player",
        "checkpoint_prefab",
        "game.rules()",
        "on_action",
    ] {
        assert!(
            source.contains(required),
            "final acceptance demo should demonstrate {required:?}"
        );
    }

    for forbidden in BEGINNER_DEMO_FORBIDDEN.iter().copied().chain([
        "GameApp<'_",
        "impl GamePlugin",
        "Health::new",
        "MeleeAttack",
        "Patrol",
    ]) {
        assert!(
            !source.contains(forbidden),
            "final acceptance demo must not expose {forbidden:?}"
        );
    }
}

#[test]
fn script_like_custom_rules_demo_stays_ecs_free() {
    let path = workspace_root().join("examples/script-like-custom-rules/src/main.rs");
    let source = read_code_without_comments(&path);

    assert!(source.contains("use game_starter::prelude::*;"));
    for required in [
        ".asset_bag()",
        "player_prefab",
        "enemy_prefab",
        "projectile_prefab",
        "spawner_prefab",
        "map(",
        "game.rules()",
        "custom_rule",
        "on_action",
    ] {
        assert!(
            source.contains(required),
            "script-like demo should demonstrate {required:?}"
        );
    }
    for forbidden in [
        "Game<'_",
        "GameCtx",
        "EntityId",
        "Component",
        "Transform",
        "Velocity",
        "Sprite",
        "Collider",
        "Health",
        "Faction",
        "MeleeAttack",
        "Patrol",
        ".commands()",
        "component::<",
        "entities_with::<",
        "game.prefab(",
        "fixed_active",
    ] {
        assert!(
            !source.contains(forbidden),
            "script-like demo must not expose {forbidden:?}"
        );
    }
}

#[test]
fn full_data_driven_demo_stays_data_only() {
    let root = workspace_root();
    let source =
        read_code_without_comments(&root.join("examples/data-driven-full-demo/src/main.rs"));

    assert!(source.contains("use game_starter::prelude::*;"));
    assert!(source.contains("load_beginner_file("));
    assert!(source.contains("assets/game.ron"));
    for forbidden in BEGINNER_DEMO_FORBIDDEN.iter().copied().chain([
        ".asset_bag()",
        ".assets_from_folders()",
        "player_prefab",
        "enemy_prefab",
        "pickup_prefab",
        "projectile_prefab",
        "spawner_prefab",
        "door_prefab",
        "trigger_prefab",
        "checkpoint_prefab",
        "game.rules()",
        "on_action",
        "on_scene_enter",
        "custom_rule",
    ]) {
        assert!(
            !source.contains(forbidden),
            "full data-driven demo main.rs must not expose {forbidden:?}"
        );
    }

    let data = fs::read_to_string(root.join("examples/data-driven-full-demo/assets/game.ron"))
        .expect("failed to read full data-driven demo game.ron");
    for required in [
        "controls: TopDown",
        "Projectile(",
        "Spawner(",
        "Door(",
        "Trigger(",
        "Checkpoint(",
        "animation_sheets:",
        "scene_flow:",
        "audio:",
        "PlayerShoots",
        "Countdown(",
        "TopDownControls",
        "EnemyDrops",
        "WinWhenAllEnemiesDead",
    ] {
        assert!(
            data.contains(required),
            "full data-driven game.ron should demonstrate {required:?}"
        );
    }
}

#[test]
fn data_driven_reload_loop_is_validated_and_honest() {
    let root = workspace_root();
    let cli = fs::read_to_string(root.join("crates/game-cli/src/lib.rs"))
        .expect("failed to read game-cli");
    assert!(
        cli.contains("validate-data") && cli.contains("validate_beginner_game_file"),
        "game-dev should expose validate-data through the same beginner data validator"
    );

    let data = fs::read_to_string(root.join("crates/game-kit/src/data.rs"))
        .expect("failed to read data loader");
    for required in [
        "BeginnerFileRuntime",
        "BeginnerReloadLevel",
        "rebuild_beginner_content_runtime",
        "changed its {kind} list",
    ] {
        assert!(
            data.contains(required),
            "data reload support should include {required:?}"
        );
    }

    let defaults = fs::read_to_string(root.join("crates/game-kit/src/beginner/defaults.rs"))
        .expect("failed to read beginner defaults");
    assert!(
        defaults.contains("reload_beginner_file_if_configured_or_log")
            && defaults.contains("reload_current_map_or_log"),
        "F5 should reload game.ron when configured and keep the text-map fallback"
    );

    let debug = fs::read_to_string(root.join("crates/game-kit/src/beginner/debug.rs"))
        .expect("failed to read debug overlay");
    for required in ["game.ron reload:", "loaded at startup", "game.ron error:"] {
        assert!(
            debug.contains(required),
            "debug overlay should explain data reload status with {required:?}"
        );
    }

    for relative in [
        "docs/tutorials/12-fast-iteration.md",
        "docs/tutorials/13-data-driven-demo.md",
        "templates/data-driven-demo/README.md",
    ] {
        let source = fs::read_to_string(root.join(relative))
            .unwrap_or_else(|err| panic!("failed to read {relative}: {err}"));
        assert!(
            source.contains("partial")
                && source.contains("Adding, removing, or")
                && source.contains("reordering")
                && source.contains("requires a restart"),
            "{relative} should describe the partial game.ron reload contract honestly"
        );
    }
}

#[test]
fn data_driven_tiled_demo_stays_data_only_and_uses_tiled_maps() {
    let root = workspace_root();
    let source =
        read_code_without_comments(&root.join("examples/data-driven-tiled-demo/src/main.rs"));

    assert!(source.contains("use game_starter::prelude::*;"));
    assert!(source.contains("load_beginner_file("));
    assert!(source.contains("assets/game.ron"));
    for forbidden in BEGINNER_DEMO_FORBIDDEN.iter().copied().chain([
        "map_from_tiled",
        ".object(",
        "player_prefab",
        "enemy_prefab",
        "game.rules()",
    ]) {
        assert!(
            !source.contains(forbidden),
            "data-driven Tiled demo main.rs must not expose {forbidden:?}"
        );
    }

    let data = fs::read_to_string(root.join("examples/data-driven-tiled-demo/assets/game.ron"))
        .expect("failed to read data-driven Tiled demo game.ron");
    for required in [
        "Tiled((",
        "path: \"maps/tiled_demo.tmx\"",
        "objects:",
        "\"Player\": \"player\"",
        "\"Slime\": \"slime\"",
        "TopDownControls",
    ] {
        assert!(
            data.contains(required),
            "data-driven Tiled game.ron should contain {required:?}"
        );
    }

    let cookbook = fs::read_to_string(root.join("docs/cookbook/tiled.md"))
        .expect("failed to read Tiled cookbook");
    assert!(cookbook.contains("cargo run -p data-driven-tiled-demo"));
    assert!(cookbook.contains("No-Rust Data File"));

    let ci = fs::read_to_string(root.join(".github/workflows/ci.yml"))
        .expect("failed to read CI workflow");
    assert!(
        ci.contains("cargo run -p data-driven-tiled-demo --locked --features ci-build-sdl3"),
        "CI should smoke-run the data-driven Tiled demo"
    );
}

#[test]
fn flag_builders_delegate_to_independent_behaviors() {
    let root = workspace_root();
    let defaults = fs::read_to_string(root.join("crates/game-kit/src/beginner/defaults.rs"))
        .expect("failed to read top-down defaults builder");
    for required in [
        "app.use_behavior(MovementBehavior",
        "app.use_behavior(MeleeCombatBehavior",
        "app.use_behavior(EnemyChaseBehavior",
        "app.use_behavior(CollisionBehavior",
        "app.use_behavior(CameraFollowBehavior",
    ] {
        assert!(
            defaults.contains(required),
            "TopDownGameAuthor::build should delegate through {required:?}"
        );
    }

    let rules = fs::read_to_string(root.join("crates/game-kit/src/beginner/rules.rs"))
        .expect("failed to read rules builder");
    for required in [
        "app.use_behavior(CollectPickupsBehavior)",
        "app.use_behavior(ProjectileMovementBehavior)",
        "app.use_behavior(SpawnerBehavior)",
        "app.use_behavior(HighLevelUiBehavior",
    ] {
        assert!(
            rules.contains(required),
            "RulesAuthor::build should delegate through {required:?}"
        );
    }
}

#[test]
fn content_tests_use_layered_testing_preludes() {
    for crate_name in ["simple-content", "arena-content"] {
        let tests_dir = workspace_root().join(format!("crates/{crate_name}/tests"));
        let mut files = Vec::new();
        collect_rust_files(&tests_dir, &mut files);

        for path in files {
            let source = read_code_without_comments(&path);
            assert!(
                source.contains("game_kit::beginner::testing::prelude"),
                "{} beginner tests should import game_kit::beginner::testing::prelude::*",
                path.display()
            );
            assert!(
                !source.contains("game_kit::testing::prelude"),
                "{} beginner tests should not import the raw compatibility testing prelude",
                path.display()
            );
            for forbidden in [
                "World",
                "EntityId",
                "Component",
                "Transform",
                "Health",
                "Faction",
            ] {
                assert!(
                    !source.contains(forbidden),
                    "{} beginner tests should avoid raw testing symbol {forbidden}",
                    path.display()
                );
            }
        }
    }

    let tests_dir = workspace_root().join("crates/testbed-content/tests");
    let mut files = Vec::new();
    collect_rust_files(&tests_dir, &mut files);
    for path in files {
        let source = read_code_without_comments(&path);
        assert!(
            source.contains("game_kit::advanced::testing::prelude"),
            "{} advanced testbed tests should import game_kit::advanced::testing::prelude::*",
            path.display()
        );
        assert!(
            !source.contains("game_kit::testing::prelude"),
            "{} advanced testbed tests should use the explicit advanced testing prelude",
            path.display()
        );
    }
}

#[test]
fn game_kit_normal_prelude_has_no_testing_or_raw_exports() {
    let source = fs::read_to_string(workspace_root().join("crates/game-kit/src/lib.rs"))
        .expect("failed to read game-kit lib");
    let prelude = extract_pub_module_body(&source, "prelude");

    for forbidden in [
        "GameTestHarness",
        "World",
        "Entity,",
        "Input,",
        "TileMap",
        "NavGrid",
        "PrefabId",
        "movement_system",
        "chase_system",
        "patrol_system",
        "apply_damage",
    ] {
        assert!(
            !prelude.contains(forbidden),
            "game_kit::prelude must not export {forbidden}"
        );
    }
}

#[test]
fn game_kit_compatibility_prelude_is_visibly_deprecated() {
    let source = fs::read_to_string(workspace_root().join("crates/game-kit/src/lib.rs"))
        .expect("failed to read game-kit lib");

    assert!(
        source.contains("#[deprecated(note = \"Use game_kit::beginner::prelude::* or game_kit::advanced::prelude::*\")]"),
        "game_kit::prelude should be marked as compatibility-only with a deprecation note"
    );
    assert!(source.contains("Compatibility prelude."));
    assert!(source.contains("game_kit::beginner::prelude::*"));
    assert!(source.contains("game_kit::advanced::prelude::*"));
}

#[test]
fn generated_templates_are_ci_checked_and_release_pinned() {
    let root = workspace_root();
    let ci = fs::read_to_string(root.join(".github/workflows/ci.yml"))
        .expect("failed to read CI workflow");

    for required in [
        "generated-templates:",
        "cargo install cargo-generate --locked --force",
        "cargo generate --path templates/simple-demo",
        "cargo generate --path templates/data-driven-demo",
        "cargo check --manifest-path /tmp/generated/smoke-simple/Cargo.toml --features ci-build-sdl3",
        "cargo check --manifest-path /tmp/generated/smoke-data/Cargo.toml --features ci-build-sdl3",
        "Build game-dev helper",
        "cargo build -p game-cli --locked --features ci-build-sdl3",
        "game-dev check --features ci-build-sdl3",
        "game-dev package --release --features ci-build-sdl3 --out /tmp/package-simple --zip",
        "game-dev package --release --features ci-build-sdl3 --out /tmp/package-data --zip",
        "cargo run --manifest-path /tmp/generated/smoke-simple/Cargo.toml --features ci-build-sdl3",
        "cargo run --manifest-path /tmp/generated/smoke-data/Cargo.toml --features ci-build-sdl3",
    ] {
        assert!(
            ci.contains(required),
            "generated-project CI must include {required:?}"
        );
    }

    for relative in [
        "templates/simple-demo/cargo-generate.toml",
        "templates/data-driven-demo/cargo-generate.toml",
    ] {
        let source = fs::read_to_string(root.join(relative))
            .unwrap_or_else(|err| panic!("failed to read {relative}: {err}"));
        assert!(
            source.contains(
                r#"default = '{ git = "https://github.com/P2949/game", rev = "b7fa6a3dc01d185312cf0e714b5efa10201578c6", package = "game-starter" }'"#
            ),
            "{relative} should pin release-generated projects to a reproducible git revision"
        );
        assert!(
            !source.contains(
                r#"default = '{ git = "https://github.com/P2949/game", package = "game-starter" }'"#
            ),
            "{relative} must not default to the moving git branch"
        );
    }

    for relative in [
        "templates/simple-demo/Cargo.toml",
        "templates/data-driven-demo/Cargo.toml",
    ] {
        let source = fs::read_to_string(root.join(relative))
            .unwrap_or_else(|err| panic!("failed to read {relative}: {err}"));
        assert!(
            source.contains(r#"ci-build-sdl3 = ["game-starter/ci-build-sdl3"]"#),
            "{relative} should let generated CI opt into source-built SDL3"
        );
        assert!(
            source.contains("[package.metadata.game]")
                && source.contains(r#"title = "{{title}}""#)
                && source.contains(r#"asset_dir = "assets""#),
            "{relative} should include beginner package metadata"
        );
    }

    let starter_manifest = fs::read_to_string(root.join("crates/game-starter/Cargo.toml"))
        .expect("failed to read game-starter manifest");
    assert!(
        starter_manifest.contains(r#"ci-build-sdl3 = ["game-runtime/ci-build-sdl3"]"#),
        "game-starter should expose the generated-project CI SDL3 feature"
    );

    let runtime_manifest = fs::read_to_string(root.join("crates/game-runtime/Cargo.toml"))
        .expect("failed to read game-runtime manifest");
    assert!(
        runtime_manifest.contains("game-platform-sdl/ci-build-sdl3")
            && runtime_manifest.contains("game-audio/ci-build-sdl3"),
        "game-runtime should forward the source-built SDL3 feature to backend crates"
    );

    for relative in [
        "crates/game-audio/Cargo.toml",
        "crates/game-platform-sdl/Cargo.toml",
    ] {
        let source = fs::read_to_string(root.join(relative))
            .unwrap_or_else(|err| panic!("failed to read {relative}: {err}"));
        assert!(
            source.contains(r#"ci-build-sdl3 = ["sdl3/build-from-source"]"#),
            "{relative} should source-build SDL3 for CI"
        );
    }
}

#[test]
fn distribution_policy_keeps_release_candidate_model_explicit() {
    let root = workspace_root();
    let policy = fs::read_to_string(root.join("docs/distribution-policy.md"))
        .expect("failed to read distribution policy");

    for required in [
        "Release-candidate templates pin",
        "specific git revision",
        "release tag",
        "cargo xtask new-demo",
        "Prebuilt demo zips",
        "Vulkan-capable GPU/driver",
        "publish crates.io packages after the beginner API stabilizes",
        "dedicated `game-template` repository",
        "version docs per release",
        "platform installer for `game-dev`",
    ] {
        assert!(
            policy.contains(required),
            "distribution policy should document {required:?}"
        );
    }

    let readme = fs::read_to_string(root.join("README.md")).expect("failed to read README");
    for required in [
        "Generated projects are pinned to release tags",
        "release-candidate templates pin a specific git revision",
        "cargo xtask new-demo",
        "distribution policy",
    ] {
        assert!(
            readme.contains(required),
            "README should link the distribution policy and explain {required:?}"
        );
    }

    let checklist = fs::read_to_string(root.join("docs/release-checklist.md"))
        .expect("failed to read release checklist");
    for required in [
        "generated-template dependency pins updated for this release tag",
        "`CHANGELOG.md` updated",
        "migration docs in `docs/migrations/` updated",
        "generated-template CI is green",
        "first-15-minutes CI is green",
        "release artifacts generated",
        "distribution policy",
        "Status: intentionally deferred",
    ] {
        assert!(
            checklist.contains(required),
            "release checklist should document distribution gate {required:?}"
        );
    }
}

#[test]
fn first_15_minutes_acceptance_path_is_scripted_and_documented() {
    let root = workspace_root();
    let script = fs::read_to_string(root.join("scripts/first-15-minutes.sh"))
        .expect("failed to read first 15 minutes script");
    for required in [
        "cargo generate --path \"$repo/templates/simple-demo\"",
        "--name first-demo",
        "cargo check",
        "GAME_SMOKE_FRAMES=\"$smoke_frames\" run_smoke cargo run",
        "assets/maps/level_1.txt",
        "$game_dev\" asset-check",
        "$game_dev\" package --release",
        "--out dist/first-demo --zip",
        "package_name=",
        "packaged executable for $package_name was not found",
        "dist/first-demo/run.sh",
    ] {
        assert!(
            script.contains(required),
            "first 15 minutes script should contain {required:?}"
        );
    }

    let ci = fs::read_to_string(root.join(".github/workflows/ci.yml"))
        .expect("failed to read CI workflow");
    for required in [
        "First 15 minutes acceptance test",
        "FIRST15_FEATURES: ci-build-sdl3",
        "FIRST15_USE_XVFB: \"1\"",
        "scripts/first-15-minutes.sh",
    ] {
        assert!(
            ci.contains(required),
            "CI should run first 15 minutes acceptance with {required:?}"
        );
    }

    let quickstart = fs::read_to_string(root.join("docs/tutorials/quickstart-zero-to-demo.md"))
        .expect("failed to read quickstart tutorial");
    for required in [
        "first 15 minutes",
        "cargo generate --path templates/simple-demo --name first-demo --destination /tmp",
        "GAME_SMOKE_FRAMES=60 cargo run",
        "assets/maps/level_1.txt",
        "game-dev asset-check",
        "game-dev package --release --out dist/first-demo --zip",
    ] {
        assert!(
            quickstart.contains(required),
            "quickstart should document first 15 minutes command {required:?}"
        );
    }
}

#[test]
fn beginner_api_stability_policy_is_documented() {
    let root = workspace_root();

    let readme = fs::read_to_string(root.join("README.md")).expect("failed to read README.md");
    for required in [
        "## API Stability",
        "Beginner API",
        "stabilized first",
        "old method for one release",
        "Data file schema",
        "versioned through `assets/game.ron`",
        "Advanced API",
        "allowed to evolve faster",
        "Engine internals",
        "unstable",
        "not tied to a moving branch",
        "docs/migrations",
    ] {
        assert!(
            readme.contains(required),
            "README should document stability policy detail {required:?}"
        );
    }

    let game_kit =
        fs::read_to_string(root.join("crates/game-kit/src/lib.rs")).expect("failed to read lib.rs");
    for required in [
        "## Stability",
        "Beginner APIs are stabilized first",
        "old method for one release",
        "Data-driven `assets/game.ron` files are versioned",
        "Advanced APIs are allowed to evolve faster",
        "Engine internals are unstable",
    ] {
        assert!(
            game_kit.contains(required),
            "game-kit rustdoc should document stability policy detail {required:?}"
        );
    }

    let changelog =
        fs::read_to_string(root.join("CHANGELOG.md")).expect("failed to read CHANGELOG.md");
    for required in [
        "### Added",
        "### Changed",
        "### Deprecated",
        "### Removed",
        "### Migration notes",
        "docs/migrations/game-ron-v1-to-v2.md",
    ] {
        assert!(
            changelog.contains(required),
            "CHANGELOG should contain release-note section {required:?}"
        );
    }

    let migrations = fs::read_to_string(root.join("docs/migrations/README.md"))
        .expect("failed to read migrations README");
    assert!(
        migrations.contains("game-ron-v1-to-v2.md"),
        "migration index should link the game.ron schema migration guide"
    );

    let game_ron_migration = fs::read_to_string(root.join("docs/migrations/game-ron-v1-to-v2.md"))
        .expect("failed to read game.ron migration guide");
    for required in [
        "version: 1",
        "version: 2",
        "game-dev validate-data assets/game.ron",
        "BeginnerGameFile.version",
    ] {
        assert!(
            game_ron_migration.contains(required),
            "game.ron migration guide should contain {required:?}"
        );
    }
}

#[test]
fn standalone_beginner_cli_is_documented_and_xtask_wrapped() {
    let root = workspace_root();
    let workspace_manifest =
        fs::read_to_string(root.join("Cargo.toml")).expect("failed to read workspace manifest");
    assert!(
        workspace_manifest.contains(r#""crates/game-cli""#),
        "workspace should include the standalone beginner CLI crate"
    );

    let cli_manifest = fs::read_to_string(root.join("crates/game-cli/Cargo.toml"))
        .expect("failed to read game-cli manifest");
    assert!(cli_manifest.contains("name = \"game-cli\""));
    assert!(cli_manifest.contains("name = \"game-dev\""));
    assert!(
        cli_manifest.contains(r#"ci-build-sdl3 = ["game-audio/ci-build-sdl3"]"#),
        "game-cli CI should be able to source-build SDL3 through game-audio"
    );
    let cli =
        fs::read_to_string(root.join("crates/game-cli/src/lib.rs")).expect("failed to read CLI");
    for required in [
        "game-dev check [--features feature-list]",
        "fn check_project",
        "validate_assets_dir(&assets, false)",
        "validate_beginner_game_file",
        "cargo check failed",
        "fn beginner_failure_advice",
        "If this looks like a setup issue:",
        "If this looks like an asset/data issue:",
        "docs/tutorials/common-errors.md",
        "game-dev doctor --explain",
        "cargo xtask release-check [--skip-smoke] [--skip-generated] [--features feature-list]",
        "fn run_release_check",
        "fn run_generated_release_checks",
        "fn run_smoke_release_checks",
        "fn run_command",
    ] {
        assert!(
            cli.contains(required),
            "game-cli should include {required:?}"
        );
    }

    let xtask_manifest =
        fs::read_to_string(root.join("xtask/Cargo.toml")).expect("failed to read xtask manifest");
    let xtask_main =
        fs::read_to_string(root.join("xtask/src/main.rs")).expect("failed to read xtask main");
    assert!(
        xtask_manifest.contains("game-cli"),
        "xtask should depend on the shared CLI implementation"
    );
    assert!(
        xtask_main.contains("game_cli::run_xtask"),
        "xtask should remain a thin wrapper around game-cli functions"
    );

    for relative in [
        "templates/simple-demo/README.md",
        "templates/data-driven-demo/README.md",
    ] {
        let source = fs::read_to_string(root.join(relative))
            .unwrap_or_else(|err| panic!("failed to read {relative}: {err}"));
        for required in [
            "cargo install --git https://github.com/P2949/game game-cli",
            "game-dev doctor",
            "game-dev check",
            "game-dev run",
            "game-dev package --release --out dist/my-game --zip",
        ] {
            assert!(
                source.contains(required),
                "{relative} should document {required:?}"
            );
        }
    }

    let readme = fs::read_to_string(root.join("README.md")).expect("failed to read README");
    assert!(
        readme.contains("cargo xtask release-check --skip-smoke"),
        "README should document the contributor release-check command"
    );
    let release_checklist = fs::read_to_string(root.join("docs/release-checklist.md"))
        .expect("failed to read release checklist");
    assert!(
        release_checklist.contains("cargo xtask release-check")
            && release_checklist.contains("--skip-smoke"),
        "release checklist should document release-check and the local smoke escape hatch"
    );
    let common_errors = fs::read_to_string(root.join("docs/tutorials/common-errors.md"))
        .expect("failed to read common errors guide");
    for required in [
        "If this looks like a setup issue:",
        "game-dev doctor --explain",
        "If this looks like an asset/data issue:",
        "game-dev asset-check",
        "game-dev validate-data assets/game.ron",
        "Rule `projectiles_damage_enemies` needs the `projectiles` rule",
    ] {
        assert!(
            common_errors.contains(required),
            "common errors guide should mirror diagnostic wording {required:?}"
        );
    }
}

#[test]
fn generated_project_package_flow_matches_beginner_contract() {
    let root = workspace_root();
    let cli =
        fs::read_to_string(root.join("crates/game-cli/src/lib.rs")).expect("failed to read CLI");
    for required in [
        "cargo build",
        "--release",
        "validate_assets_dir(&assets, false)",
        "validate_beginner_game_file",
        "run.sh",
        "run.ps1",
        "README.txt",
        "zip_package",
        "copy_runtime_libraries",
        "LD_LIBRARY_PATH",
        "libSDL3.so.0",
    ] {
        assert!(
            cli.contains(required),
            "game-dev package should include {required:?}"
        );
    }

    let tutorial = fs::read_to_string(root.join("docs/tutorials/10-package-your-demo.md"))
        .expect("failed to read package tutorial");
    for required in [
        "game-dev package --release --out dist/my-game --zip",
        "run.ps1",
        "README.txt",
        "dist/my-game.zip",
        "Send the whole dist/my-game.zip folder to a friend.",
        "runtime libraries",
    ] {
        assert!(
            tutorial.contains(required),
            "package tutorial should document {required:?}"
        );
    }
}

#[test]
fn release_workflow_publishes_prebuilt_demo_artifacts() {
    let root = workspace_root();
    let workflow = fs::read_to_string(root.join(".github/workflows/release.yml"))
        .expect("failed to read release workflow");
    for required in [
        "push:",
        "tags:",
        "\"v*\"",
        "game-demo-linux-x86_64",
        "game-demo-windows-x86_64",
        "cargo xtask package-demo --release --features ci-build-sdl3",
        "Verify Linux package archive",
        "scripts/verify-release-artifact.sh",
        "Verify Windows package archive",
        "Expand-Archive",
        "actions/upload-artifact",
        "gh release upload",
    ] {
        assert!(
            workflow.contains(required),
            "release workflow should include {required:?}"
        );
    }

    let verifier = fs::read_to_string(root.join("scripts/verify-release-artifact.sh"))
        .expect("failed to read release artifact verifier");
    for required in [
        "usage: scripts/verify-release-artifact.sh <archive.zip> <linux|windows>",
        "game",
        "game.exe",
        "libSDL3.so.0",
        "SDL3.dll",
        "run.sh",
        "run.ps1",
        "run.bat",
        "README.txt",
        "assets/fonts/DejaVuSans.ttf",
        "assets/game.ron",
        "assets/maps/tiled_demo.tmx",
        "assets/textures/test.png",
        "assets/sounds/hit.wav",
    ] {
        assert!(
            verifier.contains(required),
            "release artifact verifier should require {required:?}"
        );
    }

    let github_verifier =
        fs::read_to_string(root.join("scripts/verify-github-release-artifacts.sh"))
            .expect("failed to read GitHub release artifact verifier");
    for required in [
        "usage: scripts/verify-github-release-artifacts.sh [run-id|latest]",
        "GH_REPO",
        "RELEASE_WORKFLOW",
        "gh run list",
        "gh run download",
        "game-demo-linux-x86_64",
        "game-demo-windows-x86_64",
        "verify-release-artifact.sh",
    ] {
        assert!(
            github_verifier.contains(required),
            "GitHub release artifact verifier should include {required:?}"
        );
    }

    let cli =
        fs::read_to_string(root.join("crates/game-cli/src/lib.rs")).expect("failed to read CLI");
    assert!(
        cli.contains(
            "cargo xtask package-demo --release --out <directory> [--features feature-list]"
        ),
        "workspace demo packaging should document feature flags for release builds"
    );

    let readme = fs::read_to_string(root.join("README.md")).expect("failed to read README.md");
    for required in [
        "Want to try before building? Download the latest demo package from",
        "Releases",
        "game-demo-linux-x86_64.zip",
        "game-demo-windows-x86_64.zip",
        "Vulkan-capable GPU/driver",
        "source builds remain the main",
    ] {
        assert!(
            readme.contains(required),
            "README should document prebuilt release artifacts with {required:?}"
        );
    }

    for relative in [
        "docs/setup/linux.md",
        "docs/setup/windows.md",
        "docs/setup/macos.md",
        "docs/release-checklist.md",
    ] {
        let source = fs::read_to_string(root.join(relative))
            .unwrap_or_else(|err| panic!("failed to read {relative}: {err}"));
        assert!(
            source.contains("Prebuilt demo package")
                || source.contains("Prebuilt demo release artifacts"),
            "{relative} should document the prebuilt demo path"
        );
        assert!(
            source.contains("Releases") && source.contains("Vulkan-capable GPU/driver"),
            "{relative} should link releases and keep Vulkan expectations honest"
        );
    }

    let checklist = fs::read_to_string(root.join("docs/release-checklist.md"))
        .expect("failed to read release checklist");
    for required in [
        "gh run list --workflow release.yml --limit 5",
        "scripts/verify-github-release-artifacts.sh <run-id>",
        "latest successful `release.yml` run",
    ] {
        assert!(
            checklist.contains(required),
            "release checklist should document {required:?}"
        );
    }
}

#[test]
fn doctor_diagnostics_cover_first_run_setup() {
    let root = workspace_root();
    let cli =
        fs::read_to_string(root.join("crates/game-cli/src/lib.rs")).expect("failed to read CLI");
    for required in [
        "rustc 1.87 or newer",
        "Vulkan 1.3+ physical device",
        "SDL3 development files",
        "audio backend prerequisites",
        "assets/fonts/DejaVuSans.ttf",
        "--explain",
    ] {
        assert!(
            cli.contains(required),
            "game-dev doctor should cover {required:?}"
        );
    }

    let ci = fs::read_to_string(root.join(".github/workflows/ci.yml"))
        .expect("failed to read CI workflow");
    assert!(
        ci.contains("cargo run -p game-cli --features ci-build-sdl3 -- doctor --explain"),
        "CI should run doctor diagnostics"
    );

    for relative in [
        "docs/setup/linux.md",
        "docs/setup/macos.md",
        "docs/setup/windows.md",
        "docs/tutorials/common-errors.md",
        "README.md",
    ] {
        let source = fs::read_to_string(root.join(relative))
            .unwrap_or_else(|err| panic!("failed to read {relative}: {err}"));
        assert!(
            source.contains("game-dev doctor --explain"),
            "{relative} should mention the explanatory doctor mode"
        );
    }
}

#[test]
fn beginner_prelude_does_not_export_advanced_ecs_surface() {
    let path = workspace_root().join("crates/game-kit/src/beginner/prelude.rs");
    let source = read_code_without_comments(&path);

    for forbidden in [
        "crate::prelude",
        "EntityId",
        "Component",
        "Transform",
        "Velocity",
        "Sprite",
        "Collider",
        "Health",
        "MeleeAttack",
        "Faction",
        "AiController",
        "ChaseTarget",
        "PathFollow",
        "Patrol",
        "PrefabAuthor",
        "GameCtx",
        "StartupGameCtx",
        "Commands",
    ] {
        let found = if forbidden == "crate::prelude" {
            source.contains(forbidden)
        } else {
            contains_identifier(&source, forbidden)
        };
        assert!(!found, "beginner prelude must not export {forbidden}");
    }
}

#[test]
fn beginner_context_is_a_wrapper_not_an_advanced_alias() {
    let source = read_code_without_comments(
        &workspace_root().join("crates/game-kit/src/beginner/context.rs"),
    );

    for required in ["pub struct Game<", "pub struct StartupGame<"] {
        assert!(
            source.contains(required),
            "beginner context should define wrapper {required:?}"
        );
    }

    for forbidden in [
        "pub use crate::context",
        "pub type Game",
        "pub type StartupGame",
        "Commands",
        "EntityId",
        "Component",
        "entities_with",
        "component::<",
        "commands(",
        "resource::<",
        "resource_mut::<",
        "insert_resource",
    ] {
        assert!(
            !source.contains(forbidden),
            "beginner context wrapper must not expose advanced surface {forbidden:?}"
        );
    }

    let app = read_code_without_comments(&workspace_root().join("crates/game-kit/src/app.rs"));
    for required in [
        "FnMut(&mut BeginnerGame<'_, '_, '_>",
        "FnMut(&mut BeginnerStartupGame<'_, '_, '_>",
    ] {
        assert!(
            app.contains(required),
            "beginner callback registration should use wrapper type {required:?}"
        );
    }
}

#[test]
fn beginner_docs_examples_and_templates_hide_raw_context_methods() {
    let root = workspace_root();
    let paths = [
        "README.md",
        "docs/beginner-authoring.md",
        "docs/tutorials",
        "docs/cookbook",
        "examples/one-file-demo",
        "examples/beginner-mini-game",
        "examples/coin-collector",
        "examples/data-driven-full-demo",
        "examples/data-driven-tiled-demo",
        "examples/projectile-demo",
        "examples/script-like-custom-rules",
        "examples/two-level-demo",
        "examples/waves-demo",
        "examples/menu-game-over",
        "examples/no-rust-shapes-demo",
        "examples/win-condition-demo",
        "examples/enemy-drops-demo",
        "examples/health-pickup-demo",
        "examples/checkpoint-demo",
        "examples/boss-demo",
        "examples/dialog-demo",
        "examples/inventory-demo",
        "examples/title-menu-demo",
        "examples/damage-zone-demo",
        "examples/ldtk-demo",
        "examples/tiled-demo",
        "examples/animation-demo",
        "examples/audio-demo",
        "examples/trigger-area-demo",
        "templates/simple-demo",
        "templates/data-driven-demo",
        "crates/simple-content/src",
        "crates/arena-content/src",
    ];

    for relative in paths {
        let path = root.join(relative);
        let mut files = Vec::new();
        if path.is_dir() {
            collect_beginner_surface_files(&path, &mut files);
        } else {
            files.push(path);
        }

        for file in files {
            let source = fs::read_to_string(&file)
                .unwrap_or_else(|err| panic!("failed to read {}: {err}", file.display()));
            for forbidden in [
                "entities_with",
                "component::<",
                "component_mut::<",
                "commands()",
                "resource::<",
                "resource_mut::<",
                "insert_resource",
                "GameCtx",
                "StartupGameCtx",
                "EntityId",
            ] {
                assert!(
                    !source.contains(forbidden),
                    "{} must keep raw context method {forbidden:?} out of beginner-facing material",
                    file.display()
                );
            }
        }
    }
}

#[test]
fn readme_first_authoring_example_is_beginner_first() {
    let readme =
        fs::read_to_string(workspace_root().join("README.md")).expect("failed to read README.md");
    let section = readme
        .split("## Content Authoring Model")
        .nth(1)
        .and_then(|rest| rest.split("## Authoring levels").next())
        .expect("failed to find README content-authoring beginner section");

    assert!(section.contains("use game_starter::prelude::*"));
    assert!(section.contains("content_plugin!(MyContent, plugin"));
    assert!(section.contains(".asset_bag()"));
    assert!(section.contains("player_prefab"));

    for forbidden in [
        "game_kit::advanced::prelude",
        "GamePlugin",
        "GameApp<'_",
        "game.prefab(",
        "Transform::",
        "Velocity::",
        "Collider::box_of",
    ] {
        assert!(
            !section.contains(forbidden),
            "README first authoring example should not contain advanced API {forbidden:?}"
        );
    }
}

#[test]
fn beginner_docs_use_named_assets_before_typed_or_advanced_sections() {
    let root = workspace_root();
    let beginner_docs = [
        "README.md",
        "docs/content-authoring.md",
        "docs/beginner-authoring.md",
        "docs/tutorials/01-run-the-demo.md",
        "docs/tutorials/02-your-first-player.md",
        "docs/tutorials/03-add-a-map.md",
        "docs/tutorials/04-add-an-enemy.md",
        "docs/tutorials/optional-add-combat.md",
        "docs/tutorials/optional-add-sound-and-ui.md",
        "docs/tutorials/common-errors.md",
        "docs/cookbook/coins-and-score.md",
        "docs/cookbook/projectiles.md",
        "docs/cookbook/two-levels.md",
        "docs/cookbook/enemy-waves.md",
        "docs/cookbook/menu-and-game-over.md",
        "templates/simple-demo/README.md",
        "templates/data-driven-demo/README.md",
        "examples/one-file-demo/README.md",
    ];

    for relative in beginner_docs {
        let path = root.join(relative);
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        let beginner = beginner_doc_section(&source);

        for forbidden in [
            "assets.texture(",
            "assets.sound(",
            "TextureHandle",
            "SoundHandle",
            "GameCtx",
            "Transform::",
            "Sprite::new",
        ] {
            assert!(
                !beginner.contains(forbidden),
                "{relative} beginner section must use named beginner APIs, not {forbidden:?}"
            );
        }
    }

    let content = fs::read_to_string(root.join("docs/content-authoring.md"))
        .expect("failed to read content-authoring guide");
    for required in [
        "beginner-authoring.md",
        "tutorials/README.md",
        "cookbook/README.md",
        "advanced-content-authoring.md",
    ] {
        assert!(
            content.contains(required),
            "content-authoring index should link to {required:?}"
        );
    }
    for forbidden in BEGINNER_DOC_FORBIDDEN {
        assert!(
            !content.contains(forbidden),
            "content-authoring index should not expose advanced detail {forbidden:?}"
        );
    }

    let custom_tags = fs::read_to_string(root.join("docs/cookbook/custom-tags-and-timers.md"))
        .expect("failed to read custom tags cookbook page");
    for required in ["actors_tagged", ".tag(", ".data("] {
        assert!(
            custom_tags.contains(required),
            "custom tags cookbook page should demonstrate {required:?}"
        );
    }
}

#[test]
fn all_beginner_docs_keep_advanced_details_after_their_boundary() {
    let root = workspace_root();
    let mut paths = vec![
        root.join("README.md"),
        root.join("docs/content-authoring.md"),
        root.join("docs/beginner-authoring.md"),
        root.join("templates/simple-demo/README.md"),
        root.join("templates/data-driven-demo/README.md"),
        root.join("examples/one-file-demo/README.md"),
    ];
    collect_markdown_files(&root.join("docs/tutorials"), &mut paths);
    collect_markdown_files(&root.join("docs/cookbook"), &mut paths);

    let mut beginner_content = String::new();
    for path in paths {
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        let beginner = beginner_doc_section(&source);
        beginner_content.push_str(beginner);
        for forbidden in BEGINNER_DOC_FORBIDDEN {
            assert!(
                !beginner.contains(forbidden),
                "{} must keep {forbidden:?} out of its beginner section",
                path.display()
            );
        }
    }

    for required in [
        "game_starter::prelude",
        "asset_bag",
        ".sprite(\"",
        ".simple_theme(\"",
        "player_prefab",
        "enemy_prefab",
        "game.rules()",
    ] {
        assert!(
            beginner_content.contains(required),
            "the beginner documentation set should demonstrate {required:?}"
        );
    }
}

#[test]
fn no_rust_experience_tutorial_course_stays_complete_and_beginner_first() {
    let root = workspace_root();
    let chapters = [
        "00-start-here.md",
        "01-run-the-demo.md",
        "02-your-first-player.md",
        "03-add-a-map.md",
        "04-add-an-enemy.md",
        "05-add-pickups-and-score.md",
        "06-add-projectiles.md",
        "07-add-doors-and-levels.md",
        "08-add-sound-and-music.md",
        "09-add-ui-and-menu.md",
        "10-package-your-demo.md",
        "11-custom-behavior.md",
    ];

    for chapter in chapters {
        let source = fs::read_to_string(root.join("docs/tutorials").join(chapter))
            .unwrap_or_else(|err| panic!("failed to read tutorial {chapter}: {err}"));
        for heading in [
            "## Goal",
            "## Files to edit",
            "## Full code",
            "## What changed",
            "## Common errors",
            "## Next step",
        ] {
            assert!(
                source.contains(heading),
                "{chapter} should contain the {heading:?} section"
            );
        }
        assert!(
            source.contains("game_starter::prelude::*"),
            "{chapter} should keep the one-file game-starter path visible"
        );
    }

    let index = fs::read_to_string(root.join("docs/tutorials/README.md"))
        .expect("failed to read tutorial index");
    for chapter in chapters {
        assert!(
            index.contains(chapter),
            "tutorial index should link to {chapter}"
        );
    }
    assert!(
        index.contains("content crate"),
        "tutorial index should explain the later content-crate graduation path"
    );
}

#[test]
fn beginner_entry_docs_keep_three_tracks_and_copy_list_clear() {
    let root = workspace_root();
    for relative in ["README.md", "docs/tutorials/README.md"] {
        let source = fs::read_to_string(root.join(relative))
            .unwrap_or_else(|err| panic!("failed to read {relative}: {err}"));

        for required in [
            "Track A: No Rust",
            "templates/data-driven-demo",
            "assets/game.ron",
            "Track B: Beginner Rust",
            "templates/simple-demo",
            "tutorials 00-12",
            "Track C: Advanced",
            "advanced path",
            "beginner APIs are insufficient",
            "examples/one-file-demo",
            "examples/script-like-custom-rules",
            "examples/events-demo",
            "examples/tiled-demo",
            "crates/testbed-content",
            "do not copy first",
        ] {
            assert!(
                source.contains(required),
                "{relative} should make the beginner path include {required:?}"
            );
        }

        assert!(
            !source.contains("Track D:"),
            "{relative} should keep the entry path to three tracks"
        );
    }
}

#[test]
fn tutorial_numbered_chapters_have_unique_prefixes() {
    let tutorials = workspace_root().join("docs/tutorials");
    let mut prefixes = BTreeMap::<String, Vec<String>>::new();
    for entry in fs::read_dir(&tutorials).expect("failed to read tutorial directory") {
        let entry = entry.expect("failed to read tutorial entry");
        let file_name = entry.file_name().to_string_lossy().into_owned();
        let Some((prefix, _)) = file_name.split_once('-') else {
            continue;
        };
        if prefix.len() == 2 && prefix.chars().all(|ch| ch.is_ascii_digit()) {
            prefixes
                .entry(prefix.to_owned())
                .or_default()
                .push(file_name);
        }
    }

    let duplicates = prefixes
        .into_iter()
        .filter(|(_, files)| files.len() > 1)
        .collect::<Vec<_>>();
    assert!(
        duplicates.is_empty(),
        "tutorial chapter numbers should be unique: {duplicates:#?}"
    );
}

#[test]
fn beginner_facing_docs_examples_and_templates_do_not_use_compatibility_prelude() {
    let root = workspace_root();
    let paths = [
        "README.md",
        "docs/content-authoring.md",
        "docs/beginner-authoring.md",
        "docs/tutorials",
        "docs/cookbook",
        "templates/simple-demo",
        "templates/data-driven-demo",
        "examples/one-file-demo",
        "examples/beginner-mini-game",
        "examples/data-driven-full-demo",
        "examples/data-driven-tiled-demo",
        "examples/ldtk-demo",
        "examples/tiled-demo",
    ];

    for relative in paths {
        let path = root.join(relative);
        let mut files = Vec::new();
        if path.is_dir() {
            collect_beginner_surface_files(&path, &mut files);
        } else {
            files.push(path);
        }

        for file in files {
            let source = fs::read_to_string(&file)
                .unwrap_or_else(|err| panic!("failed to read {}: {err}", file.display()));
            assert!(
                !source.contains("game_kit::prelude::*"),
                "{} must use an explicit beginner or advanced prelude, never the compatibility prelude",
                file.display()
            );
        }
    }
}

#[test]
fn game_kit_rustdoc_is_beginner_first() {
    let source = fs::read_to_string(workspace_root().join("crates/game-kit/src/lib.rs"))
        .expect("failed to read game-kit lib");
    let beginner_section = source
        .split("## Advanced authoring")
        .next()
        .expect("game-kit rustdoc must have a beginner section");

    assert!(beginner_section.contains("game_kit::beginner::prelude"));
    for forbidden in ["Transform::", "Velocity::", "Sprite::new", "GameCtx"] {
        assert!(
            !beginner_section.contains(forbidden),
            "game-kit beginner rustdoc must not expose {forbidden:?}"
        );
    }
}

#[test]
fn game_test_harness_is_not_root_reexported() {
    let source = fs::read_to_string(workspace_root().join("crates/game-kit/src/lib.rs"))
        .expect("failed to read game-kit lib");

    assert!(
        !source.contains("pub use harness::GameTestHarness;"),
        "GameTestHarness should be exposed through game_kit::testing, not the crate root"
    );
}

#[test]
fn beginner_content_uses_only_beginner_surface() {
    for crate_name in BEGINNER_CONTENT_CRATES {
        let findings = forbidden_source_uses(crate_name, BEGINNER_CONTENT_FORBIDDEN);
        assert!(
            findings.is_empty(),
            "{crate_name} must not use advanced APIs:\n{}",
            findings.join("\n")
        );
    }
}

#[test]
fn simple_content_uses_the_asset_bag_beginner_path() {
    // `simple-content` is the pure beginner reference: keep its asset names
    // string-based instead of introducing typed engine handles.
    let findings = forbidden_source_uses("simple-content", SIMPLE_CONTENT_FORBIDDEN);
    assert!(
        findings.is_empty(),
        "simple-content must model the asset_bag beginner path:\n{}",
        findings.join("\n")
    );
}

#[test]
fn arena_content_is_the_structured_beginner_typed_asset_example() {
    let assets = fs::read_to_string(workspace_root().join("crates/arena-content/src/assets.rs"))
        .expect("failed to read arena assets");
    assert!(
        assets.contains("pub struct ArenaAssets"),
        "arena-content should retain its typed asset struct as the structured beginner example"
    );

    // Typed assets are allowed here, but `arena-content` remains constrained
    // to the high-level beginner surface by `beginner_content_uses_only_beginner_surface`.
    let findings = forbidden_source_uses("arena-content", BEGINNER_CONTENT_FORBIDDEN);
    assert!(
        findings.is_empty(),
        "arena-content may use typed assets but must not expose ECS/runtime concepts:\n{}",
        findings.join("\n")
    );
}

const BEGINNER_CONTENT_CRATES: &[&str] = &["simple-content", "arena-content"];
const ADVANCED_CONTENT_CRATES: &[&str] = &["testbed-content"];

const BEGINNER_CONTENT_FORBIDDEN: &[&str] = &[
    "game_kit::prelude::*",
    "game_kit::advanced::prelude::*",
    "EntityId",
    "Component",
    "Transform",
    "Velocity",
    "Sprite::new",
    "Collider::box_of",
    "Health::new",
    "MeleeAttack",
    "Faction",
    "AiController",
    "ChaseTarget",
    "PathFollow",
    "Patrol",
    "GameCtx<'_",
    "StartupGameCtx<'_",
    "PrefabAuthor",
    "game.prefab(",
    "component::<",
    "component_mut::<",
    "entities_with::<",
    "entities_where::<",
    "for_each",
    "nearest_by_position",
    "nearest_living_with",
    "living_entities_with",
    "fixed_active::<",
    "fixed_systems_are_pause_guarded",
    "spawn_prefab_at",
    "RuntimeConfig",
    "game_runtime::run",
    "plugin_fn",
    "for<'app>",
];

const SIMPLE_CONTENT_FORBIDDEN: &[&str] = &[
    "TextureHandle",
    "SoundHandle",
    "AssetAuthor",
    "game.assets(",
    "struct SimpleAssets",
    "register_assets(",
];

const BEGINNER_DEMO_FORBIDDEN: &[&str] = &[
    "Game<'_",
    "GameCtx<'_",
    "StartupGameCtx<'_",
    "EntityId",
    "Component",
    "Transform",
    "Velocity",
    "Sprite::new",
    "Collider::box_of",
    "Health::new",
    "MeleeAttack",
    "Faction",
    "PrefabAuthor",
    "game.prefab(",
    "game.commands()",
    "RuntimeConfig",
    "game_runtime::run",
    "plugin_fn",
    "for<'app>",
    "spawn_start_map",
    "reset_to_start_map_or_log",
    "game_kit::prelude::*",
    "game_kit::advanced::prelude::*",
];

const BEGINNER_DOC_FORBIDDEN: &[&str] = &[
    "game_kit::prelude::*",
    "game_kit::advanced::prelude::*",
    "Transform::",
    "Velocity::",
    "Sprite::new",
    "GameCtx",
    "assets.texture(\"player\")",
    "assets.sound(\"hit\")",
];

fn forbidden_source_uses(crate_name: &str, patterns: &[&str]) -> Vec<String> {
    let src_dir = workspace_root().join(format!("crates/{crate_name}/src"));
    if !src_dir.exists() {
        return Vec::new();
    }

    let mut files = Vec::new();
    collect_rust_files(&src_dir, &mut files);

    let mut findings = Vec::new();
    for path in files {
        let source = read_code_without_comments(&path);
        let production = strip_cfg_test_modules(&source);
        for pattern in patterns {
            if production.contains(pattern) {
                let relative = path.strip_prefix(workspace_root()).unwrap_or(&path);
                findings.push(format!("{} contains {:?}", relative.display(), pattern));
            }
        }
    }
    findings
}

fn contains_identifier(source: &str, name: &str) -> bool {
    source
        .split(|ch: char| ch != '_' && !ch.is_ascii_alphanumeric())
        .any(|token| token == name)
}

fn assert_content_avoids_engine_internals(path: &Path, production: &str) {
    for forbidden in [
        "game_core::",
        "game_map::",
        "game_ai::",
        "game_combat::",
        "game_physics::",
        "game_runtime",
        "game_renderer_vulkan",
        "game_platform_sdl",
        "game_audio",
        "GameBuilder",
        "Schedule",
        "PrefabRegistry",
        "MapRegistry",
        "PrefabValidator",
        "MapValidator",
        "StartCtx",
        "CommandQueue",
    ] {
        assert!(
            !production.contains(forbidden),
            "{} must not reach around game-kit with {forbidden:?}",
            path.display()
        );
    }
}

/// Reads Rust source with whole-line comments removed, so documentation that
/// mentions a forbidden crate name does not trip a `contains` import check.
fn read_code_without_comments(path: &Path) -> String {
    let raw = fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    raw.lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn content_crates_depend_only_on_game_kit_and_common_deps() {
    // Phase 13: content authors use the facade. Direct gameplay-crate dependencies
    // mean the facade has a gap; backend dependencies break the runtime boundary.
    for crate_name in ["simple-content", "arena-content", "testbed-content"] {
        let manifest = read_manifest_without_comments(&format!("crates/{crate_name}/Cargo.toml"));
        for forbidden in [
            "game-core",
            "game-map",
            "game-ai",
            "game-combat",
            "game-physics",
            "game-runtime",
            "game-renderer-vulkan",
            "game-platform-sdl",
            "game-audio",
            "ash",
            "sdl3",
            "gpu-allocator",
        ] {
            assert!(
                !manifest.contains(forbidden),
                "{crate_name} manifest must not depend on {forbidden}"
            );
        }

        for required in ["anyhow.workspace", "glam.workspace", "game-kit"] {
            assert!(
                manifest.contains(required),
                "{crate_name} manifest should depend on {required}"
            );
        }
    }
}

#[test]
fn content_source_uses_authoring_facade_not_engine_internals() {
    for crate_name in BEGINNER_CONTENT_CRATES {
        let src_dir = workspace_root().join(format!("crates/{crate_name}/src"));
        let mut files = Vec::new();
        collect_rust_files(&src_dir, &mut files);

        for path in files {
            let source = read_code_without_comments(&path);
            let production = strip_cfg_test_modules(&source);
            assert!(
                production.contains("use game_kit::beginner::prelude::*;"),
                "{} should import the beginner authoring prelude",
                path.display()
            );
            assert_content_avoids_engine_internals(&path, &production);
        }
    }

    for crate_name in ADVANCED_CONTENT_CRATES {
        let src_dir = workspace_root().join(format!("crates/{crate_name}/src"));
        let mut files = Vec::new();
        collect_rust_files(&src_dir, &mut files);

        for path in files {
            let source = read_code_without_comments(&path);
            let production = strip_cfg_test_modules(&source);
            assert!(
                production.contains("use game_kit::advanced::prelude::*;"),
                "{} should import the advanced authoring prelude",
                path.display()
            );
            assert_content_avoids_engine_internals(&path, &production);
        }
    }
}

#[test]
fn testbed_content_is_documented_as_advanced() {
    let readme = fs::read_to_string(workspace_root().join("crates/testbed-content/README.md"))
        .expect("failed to read testbed README");
    let lib = fs::read_to_string(workspace_root().join("crates/testbed-content/src/lib.rs"))
        .expect("failed to read testbed lib");

    assert!(readme.to_lowercase().contains("advanced"));
    assert!(lib.to_lowercase().contains("advanced"));
    assert!(lib.contains("game_kit::advanced::prelude"));
}

#[test]
fn testbed_content_remains_an_explicit_advanced_lab() {
    let source_dir = workspace_root().join("crates/testbed-content/src");
    let mut files = Vec::new();
    collect_rust_files(&source_dir, &mut files);
    assert!(
        !files.is_empty(),
        "testbed-content should contain source files"
    );
    for path in files {
        let source = read_code_without_comments(&path);
        assert!(
            source.contains("use game_kit::advanced::prelude::*;"),
            "{} must keep its explicit advanced prelude",
            path.display()
        );
        for forbidden in [
            "game_core::",
            "game_runtime::",
            "game_renderer_vulkan::",
            "sdl3::",
        ] {
            assert!(
                !source.contains(forbidden),
                "{} advanced content still must not bypass game-kit with {forbidden:?}",
                path.display()
            );
        }
    }

    for relative in [
        "README.md",
        "docs/content-authoring.md",
        "docs/when-to-use-advanced-api.md",
        "docs/tutorials/README.md",
    ] {
        let source = fs::read_to_string(workspace_root().join(relative))
            .unwrap_or_else(|err| panic!("failed to read {relative}: {err}"));
        assert!(
            source.contains("testbed-content") && source.contains("advanced"),
            "{relative} should direct beginners away from the advanced testbed"
        );
    }
}

#[test]
fn advanced_transition_guide_names_the_boundary() {
    let root = workspace_root();
    let guide = fs::read_to_string(root.join("docs/when-to-use-advanced-api.md"))
        .expect("failed to read advanced transition guide");
    for required in [
        "Most demos should stay with the beginner API",
        "Stay beginner for normal demos",
        "Use `game_kit::advanced::prelude::*`",
        "custom ECS systems",
        "GameCtx",
        "Keep those concepts out of beginner templates",
        "Do not copy `testbed-content` for a first game",
        "advanced lab",
    ] {
        assert!(
            guide.contains(required),
            "advanced transition guide should contain {required:?}"
        );
    }

    for relative in [
        "README.md",
        "docs/content-authoring.md",
        "docs/advanced-content-authoring.md",
        "docs/tutorials/advanced-when-needed.md",
    ] {
        let source = fs::read_to_string(root.join(relative))
            .unwrap_or_else(|err| panic!("failed to read {relative}: {err}"));
        assert!(
            source.contains("when-to-use-advanced-api.md"),
            "{relative} should link the advanced transition guide"
        );
    }
}

#[test]
fn content_does_not_import_game_core_internal_prelude() {
    for crate_name in ["arena-content", "testbed-content"] {
        let src_dir = workspace_root().join(format!("crates/{crate_name}/src"));
        let mut files = Vec::new();
        collect_rust_files(&src_dir, &mut files);

        for path in files {
            let source = read_code_without_comments(&path);
            let production = strip_cfg_test_modules(&source);

            assert!(
                !production.contains("game_core::internal_prelude"),
                "{} production content must use game_kit::prelude, not game_core::internal_prelude",
                path.display()
            );
        }
    }
}

#[test]
fn production_content_does_not_use_raw_world_escape_hatches() {
    for crate_name in ["arena-content", "testbed-content"] {
        let src_dir = workspace_root().join(format!("crates/{crate_name}/src"));
        let mut files = Vec::new();
        collect_rust_files(&src_dir, &mut files);

        for path in files {
            let source = read_code_without_comments(&path);
            let production = strip_cfg_test_modules(&source);

            for forbidden in [
                "World",
                "Entity::new",
                ".ids_with::<",
                ".get::<",
                ".get_mut::<",
                ".world()",
                ".world_mut()",
                ".world_and_input()",
                ".world_and_map()",
                ".world_and_nav()",
                "TileMap",
                "NavGrid",
                "movement_system(",
                "chase_system(",
                "patrol_system(",
                "apply_damage(",
                "game_kit::testing::prelude",
            ] {
                assert!(
                    !production.contains(forbidden),
                    "{} production content must not use raw ECS/helper {:?}",
                    path.display(),
                    forbidden
                );
            }
        }
    }
}

#[test]
fn runtime_and_backends_do_not_depend_on_content_crates() {
    for crate_name in [
        "game-runtime",
        "game-renderer-vulkan",
        "game-platform-sdl",
        "game-audio",
    ] {
        let manifest = read_manifest_without_comments(&format!("crates/{crate_name}/Cargo.toml"));
        for forbidden in ["arena-content", "testbed-content"] {
            assert!(
                !manifest.contains(forbidden),
                "{crate_name} manifest must not depend on {forbidden}"
            );
        }

        let src_dir = workspace_root().join(format!("crates/{crate_name}/src"));
        let mut files = Vec::new();
        collect_rust_files(&src_dir, &mut files);
        for path in files {
            let source = read_code_without_comments(&path);
            for forbidden in ["arena_content", "testbed_content"] {
                assert!(
                    !source.contains(forbidden),
                    "{} must not import {forbidden}; bin/game selects content plugins",
                    path.display()
                );
            }
        }
    }
}

/// Reads a manifest with comment lines stripped, so documentation that mentions a
/// forbidden crate name does not trip a `contains` dependency check.
fn read_manifest_without_comments(relative: &str) -> String {
    let raw = fs::read_to_string(workspace_root().join(relative))
        .unwrap_or_else(|err| panic!("failed to read {relative}: {err}"));
    raw.lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n")
}

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("game-core lives under crates/")
}

fn beginner_doc_section(source: &str) -> &str {
    let boundary = [
        "## Typed assets for larger content crates",
        "## Advanced Path",
        "### Advanced API",
    ]
    .into_iter()
    .filter_map(|marker| source.find(marker))
    .min()
    .unwrap_or(source.len());
    &source[..boundary]
}

fn collect_rust_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    for entry in fs::read_dir(dir).unwrap_or_else(|err| {
        panic!("failed to read directory {}: {err}", dir.display());
    }) {
        let entry = entry.expect("failed to read directory entry");
        let path = entry.path();
        if path.is_dir() {
            collect_rust_files(&path, out);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

fn collect_beginner_surface_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    for entry in fs::read_dir(dir).unwrap_or_else(|err| {
        panic!("failed to read directory {}: {err}", dir.display());
    }) {
        let entry = entry.expect("failed to read directory entry");
        let path = entry.path();
        if path.is_dir() {
            collect_beginner_surface_files(&path, out);
        } else if matches!(
            path.extension().and_then(|extension| extension.to_str()),
            Some("rs" | "md")
        ) {
            out.push(path);
        }
    }
}

fn collect_markdown_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    for entry in fs::read_dir(dir).unwrap_or_else(|err| {
        panic!("failed to read directory {}: {err}", dir.display());
    }) {
        let entry = entry.expect("failed to read directory entry");
        let path = entry.path();
        if path.is_dir() {
            collect_markdown_files(&path, out);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("md") {
            out.push(path);
        }
    }
}

fn extract_pub_module_body<'a>(source: &'a str, module_name: &str) -> &'a str {
    let marker = format!("pub mod {module_name}");
    let module_start = source
        .find(&marker)
        .unwrap_or_else(|| panic!("failed to find {marker}"));
    let open_brace = module_start
        + source[module_start..]
            .find('{')
            .unwrap_or_else(|| panic!("failed to find opening brace for {marker}"));
    let mut depth = 0usize;

    for (offset, ch) in source[open_brace..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    let close_brace = open_brace + offset;
                    return &source[open_brace + 1..close_brace];
                }
            }
            _ => {}
        }
    }

    panic!("failed to find closing brace for {marker}");
}

fn strip_cfg_test_modules(source: &str) -> String {
    let mut output = Vec::new();
    let mut pending_cfg_test = false;
    let mut skipping_test_module = false;
    let mut brace_depth = 0usize;

    for line in source.lines() {
        let trimmed = line.trim_start();

        if skipping_test_module {
            brace_depth = apply_brace_delta(brace_depth, line);
            if brace_depth == 0 {
                skipping_test_module = false;
            }
            continue;
        }

        if pending_cfg_test {
            pending_cfg_test = false;
            if trimmed.starts_with("mod tests") {
                brace_depth = apply_brace_delta(0, line);
                if brace_depth > 0 {
                    skipping_test_module = true;
                }
                continue;
            }
        }

        if trimmed.starts_with("#[cfg(test)]") {
            pending_cfg_test = true;
            continue;
        }

        output.push(line);
    }

    output.join("\n")
}

fn apply_brace_delta(depth: usize, line: &str) -> usize {
    let opens = line.chars().filter(|ch| *ch == '{').count();
    let closes = line.chars().filter(|ch| *ch == '}').count();
    depth.saturating_add(opens).saturating_sub(closes)
}

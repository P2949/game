mod support;

use std::fs;

use support::*;

fn read_module_tree_without_comments(relative: &str) -> String {
    let mut files = Vec::new();
    collect_rust_files(&workspace_root().join(relative), &mut files);
    files.sort();
    files
        .into_iter()
        .map(|path| read_code_without_comments(&path))
        .collect::<Vec<_>>()
        .join("\n")
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
    let root = workspace_root();
    for relative in [
        "examples/tiled-demo/build.rs",
        "examples/tiled-demo/README.md",
        "examples/tiled-demo/assets/maps/tiled_demo.tmx",
    ] {
        assert!(
            root.join(relative).is_file(),
            "Rust Tiled demo should include copyable file {relative}"
        );
    }

    let source = read_code_without_comments(&root.join("examples/tiled-demo/src/main.rs"));
    for required in [
        "use game_starter::prelude::*;",
        "assets_from_folders()",
        "required_textures([\"player\", \"slime\", \"floor\", \"wall\"])",
        "required_sounds([\"hit\"])",
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
    assert!(
        !source.contains("textures/test.png"),
        "Rust Tiled demo should not depend on the workspace test texture"
    );
    for forbidden in BEGINNER_DEMO_FORBIDDEN {
        assert!(
            !source.contains(forbidden),
            "tiled demo must not expose beginner implementation detail {forbidden:?}"
        );
    }

    let ci = fs::read_to_string(root.join(".github/workflows/ci.yml"))
        .expect("failed to read CI workflow");
    assert!(
        ci.contains("cargo run -p tiled-demo --locked --features ci-build-sdl3"),
        "CI should smoke-run tiled-demo"
    );
    assert!(
        ci.contains("examples/tiled-demo/assets"),
        "CI should smoke-run tiled-demo against its own assets"
    );
    assert!(
        ci.contains("examples/data-driven-tiled-demo/assets"),
        "CI should smoke-run data-driven-tiled-demo against its own assets"
    );

    let release_check = read_game_cli_sources();
    for required in [
        "GAME_ASSET_DIR=examples/tiled-demo/assets",
        "GAME_ASSET_DIR=examples/data-driven-tiled-demo/assets",
    ] {
        assert!(
            release_check.contains(required),
            "release-check smoke command should include {required:?}"
        );
    }
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
fn architecture_primary_authoring_toml_fixtures_do_not_use_rust_shaped_syntax() {
    let root = workspace_root();
    for relative in ["examples/data-driven-full-demo/game.toml"] {
        let source = fs::read_to_string(root.join(relative))
            .unwrap_or_else(|error| panic!("failed to read {relative}: {error}"));

        for required in [
            "kind = \"player\"",
            "kind = \"enemy\"",
            "kind = \"pickup\"",
            "preset = \"top-down\"",
            "\"top-down-controls\"",
            "\"win-when-all-enemies-dead\"",
        ] {
            assert!(
                source.contains(required),
                "{relative} should demonstrate primary TOML spelling {required:?}"
            );
        }

        for forbidden in [
            "Some(",
            "Player((",
            "Enemy((",
            "Pickup((",
            "Projectile((",
            "Spawner((",
            "Door((",
            "Trigger((",
            "Checkpoint((",
            "TopDownControls",
            "WinWhenAllEnemiesDead",
            "TowardsMouse",
            "::",
            "fn ",
            "impl ",
            "struct ",
            "enum ",
            "pub ",
            "Result",
        ] {
            assert!(
                !source.contains(forbidden),
                "{relative} must not expose Rust/RON-shaped primary authoring syntax {forbidden:?}"
            );
        }
    }
}

#[test]
fn data_driven_reload_loop_is_validated_and_honest() {
    let root = workspace_root();
    let cli = fs::read_to_string(root.join("crates/game-cli/src/lib.rs"))
        .expect("failed to read game-cli");
    assert!(
        cli.contains("Some(\"validate-data\") => validate_data_command(args)"),
        "game-dev should route validate-data through the focused command module"
    );
    let validate_data =
        fs::read_to_string(root.join("crates/game-cli/src/commands/validate_data.rs"))
            .expect("failed to read validate-data command");
    assert!(
        validate_data.contains("validate_authoring_file_with_asset_root")
            && validate_data.contains("validate_beginner_game_file")
            && validate_data.contains("RON data files are legacy"),
        "game-dev validate-data should use the primary TOML validator and retain a labeled legacy RON validator"
    );

    let data = fs::read_to_string(root.join("crates/game-kit/src/data/mod.rs"))
        .expect("failed to read data loader");
    for required in [
        "BeginnerFileRuntime",
        "BeginnerReloadLevel",
        "rebuild_authoring_content_runtime",
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
    for required in [
        "last reload:",
        "{name}: loaded at startup",
        "{name} reload:",
        "{name} error:",
    ] {
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
    assert!(cookbook.contains("Tiled no-Rust"));
    assert!(cookbook.contains("game-dev preview --project examples/no-rust-tiled"));
    assert!(cookbook.contains("legacy Rust-wrapper"));
    assert!(cookbook.contains("examples/data-driven-tiled-demo"));

    let ci = fs::read_to_string(root.join(".github/workflows/ci.yml"))
        .expect("failed to read CI workflow");
    assert!(
        ci.contains("cargo run -p data-driven-tiled-demo --locked --features ci-build-sdl3"),
        "CI should smoke-run the data-driven Tiled demo"
    );
    assert!(
        ci.contains("examples/data-driven-tiled-demo/assets"),
        "CI should smoke-run the data-driven Tiled demo against its own assets"
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

    let rules = read_module_tree_without_comments("crates/game-kit/src/beginner/rules");
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
fn game_kit_commands_do_not_expose_raw_map_change() {
    let source = fs::read_to_string(workspace_root().join("crates/game-kit/src/context.rs"))
        .expect("failed to read game-kit context");

    assert!(
        !source.contains("pub fn change_map(&mut self, map: game_core::builder::MapId)"),
        "content-facing Commands must not expose raw MapId switching"
    );
    assert!(
        source.contains("pub(crate) fn queue_active_map_change_unchecked"),
        "game-kit should keep raw active-map command queueing crate-private"
    );
    assert!(
        source.contains("let map_id = change_to_map_world(self.inner.world, map)?;"),
        "GameCtx::change_map should keep updating ContentRuntime before queueing the runtime map switch"
    );
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

    let app = read_module_tree_without_comments("crates/game-kit/src/app");
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

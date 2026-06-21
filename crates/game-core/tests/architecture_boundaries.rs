use std::fs;
use std::path::Path;

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
            source.contains(".asset_bag()") || source.contains(".assets_from_folders()"),
            "{relative} should use a beginner asset registration helper"
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
        "examples/animation-demo/src",
        "examples/audio-demo/src",
        "examples/trigger-area-demo/src",
        "templates/simple-demo/src",
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
                source.contains(".asset_bag()") || source.contains(".assets_from_folders()"),
                "{} should use a beginner asset helper",
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
fn beginner_facing_sources_hide_context_lifetime_annotations() {
    let root = workspace_root();
    let paths = [
        "examples/one-file-demo",
        "examples/beginner-mini-game",
        "examples/coin-collector",
        "examples/projectile-demo",
        "examples/script-like-custom-rules",
        "examples/two-level-demo",
        "examples/waves-demo",
        "examples/menu-game-over",
        "examples/no-rust-shapes-demo",
        "examples/animation-demo",
        "templates/simple-demo",
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
        "game.rules()",
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
        "docs/tutorials/05-add-combat.md",
        "docs/tutorials/06-add-sound-and-ui.md",
        "docs/tutorials/common-errors.md",
        "docs/cookbook/coins-and-score.md",
        "docs/cookbook/projectiles.md",
        "docs/cookbook/two-levels.md",
        "docs/cookbook/enemy-waves.md",
        "docs/cookbook/menu-and-game-over.md",
        "templates/simple-demo/README.md",
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
}

#[test]
fn all_beginner_docs_keep_advanced_details_after_their_boundary() {
    let root = workspace_root();
    let mut paths = vec![
        root.join("README.md"),
        root.join("docs/content-authoring.md"),
        root.join("docs/beginner-authoring.md"),
        root.join("templates/simple-demo/README.md"),
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
fn beginner_facing_docs_examples_and_templates_do_not_use_compatibility_prelude() {
    let root = workspace_root();
    let paths = [
        "README.md",
        "docs/content-authoring.md",
        "docs/beginner-authoring.md",
        "docs/tutorials",
        "docs/cookbook",
        "templates/simple-demo",
        "examples/one-file-demo",
        "examples/beginner-mini-game",
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
    let findings = forbidden_source_uses("simple-content", SIMPLE_CONTENT_FORBIDDEN);
    assert!(
        findings.is_empty(),
        "simple-content must model the asset_bag beginner path:\n{}",
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

mod support;

use std::collections::BTreeMap;
use std::fs;

use support::*;

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
            "Tiled no-Rust",
            "examples/data-driven-tiled-demo",
            "Tiled Rust",
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

    let tiled_cookbook = fs::read_to_string(root.join("docs/cookbook/tiled.md"))
        .expect("failed to read Tiled cookbook");
    for required in ["Tiled Rust", "Tiled no-Rust"] {
        assert!(
            tiled_cookbook.contains(required),
            "Tiled cookbook should preserve the {required:?} path label"
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

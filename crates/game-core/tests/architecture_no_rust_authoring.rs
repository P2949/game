mod support;

use std::fs;
use std::path::Path;

use support::*;

const FORBIDDEN_PROJECT_FILES: &[&str] = &["Cargo.toml", "Cargo.lock", "build.rs", "src/main.rs"];

const FORBIDDEN_DATA_TOKENS: &[&str] = &[
    "Some(",
    "None",
    "Ok(",
    "Err(",
    "Result",
    "Vec",
    "HashMap",
    "BTreeMap",
    "Player((",
    "Enemy((",
    "Pickup((",
    "Door((",
    "Projectile((",
    "Spawner((",
    "Trigger((",
    "Checkpoint((",
    "TextMap((",
    "TextMapAuto((",
    "Tiled((",
    "Ldtk((",
    "TopDownControls",
    "PlayerCollectsPickups",
    "EnemiesDamagePlayer",
    "CameraFollowsPlayer",
    "ShowScore",
    "ShowPlayerHealth",
    "::",
    "=>",
    "fn ",
    "impl ",
    "struct ",
    "enum ",
    "trait ",
    "pub ",
    "use ",
    "match ",
    "<",
    ">",
];

const FORBIDDEN_ENGINE_VOCAB: &[&str] = &[
    "GameCtx",
    "StartupGameCtx",
    "EntityId",
    "Component",
    "World",
    "Transform",
    "Velocity",
    "Sprite::new",
    "Collider::box_of",
    "CommandQueue",
    "RuntimeConfig",
    "game_runtime",
    "game_core",
    "game_renderer_vulkan",
    "game_platform_sdl",
    "ash",
    "sdl3",
    "swapchain",
    "descriptor",
    "allocator",
    "lifetime",
    "generic",
    "trait",
    "cargo run",
    "cargo check",
    "rustc",
];

#[test]
fn primary_no_rust_corpus_is_explicit() {
    for required in [
        "templates/no-rust-demo",
        "examples/no-rust-minimal",
        "examples/no-rust-events",
        "examples/no-rust-waves",
        "examples/no-rust-projectiles",
        "examples/no-rust-full",
        "examples/no-rust-tiled",
    ] {
        assert!(
            PRIMARY_NO_RUST_PATHS.contains(&required),
            "primary no-Rust corpus should include {required}"
        );
    }

    for required in [
        "README.md",
        "docs/api-boundary.md",
        "docs/no-rust-package-layout.md",
        "docs/beginner-authoring.md",
        "docs/content-authoring.md",
    ] {
        assert!(
            PRIMARY_NO_RUST_DOCS.contains(&required),
            "primary no-Rust docs corpus should include {required}"
        );
    }
}

#[test]
fn primary_no_rust_packages_are_plain_authoring_folders() {
    let root = workspace_root();
    for relative in PRIMARY_NO_RUST_PATHS {
        let package = root.join(relative);
        assert!(
            package.join("game.toml").is_file(),
            "{relative} should contain root game.toml"
        );
        assert!(
            package.join("assets").is_dir(),
            "{relative} should contain assets/"
        );
        for forbidden in FORBIDDEN_PROJECT_FILES {
            assert!(
                !package.join(forbidden).exists(),
                "{relative} must not contain {forbidden}"
            );
        }
        assert!(
            !package.join("assets/game.ron").exists(),
            "{relative} must not contain legacy assets/game.ron"
        );

        let mut files = Vec::new();
        collect_all_files(&package, &mut files);
        for file in files {
            assert_ne!(
                file.extension().and_then(|extension| extension.to_str()),
                Some("rs"),
                "{relative} must not contain Rust source: {}",
                file.display()
            );
            assert_ne!(
                file.extension().and_then(|extension| extension.to_str()),
                Some("ron"),
                "{relative} must not contain RON metadata or data files: {}",
                file.display()
            );
        }
    }
}

#[test]
fn primary_no_rust_toml_is_not_rust_or_ron_shaped() {
    let root = workspace_root();
    for relative in PRIMARY_NO_RUST_PATHS {
        let package = root.join(relative);
        let mut files = Vec::new();
        collect_all_files(&package, &mut files);
        for file in files {
            if file.extension().and_then(|extension| extension.to_str()) != Some("toml") {
                continue;
            }
            let source = fs::read_to_string(&file)
                .unwrap_or_else(|err| panic!("failed to read {}: {err}", file.display()));
            for forbidden in FORBIDDEN_DATA_TOKENS {
                assert!(
                    !source_contains_forbidden_token(&source, forbidden),
                    "{} must not contain Rust/RON-shaped token {forbidden:?}",
                    file.display()
                );
            }
        }
    }
}

#[test]
fn primary_no_rust_text_surfaces_hide_engine_vocabulary() {
    let root = workspace_root();
    for relative in PRIMARY_NO_RUST_PATHS {
        let package = root.join(relative);
        let mut files = Vec::new();
        collect_all_files(&package, &mut files);
        for file in files {
            if !is_primary_text_surface(&file) {
                continue;
            }
            let source = fs::read_to_string(&file)
                .unwrap_or_else(|err| panic!("failed to read {}: {err}", file.display()));
            for forbidden in FORBIDDEN_ENGINE_VOCAB {
                assert!(
                    !source_contains_forbidden_token(&source, forbidden),
                    "{} must not expose engine/Rust vocabulary {forbidden:?}",
                    file.display()
                );
            }
        }
    }
}

#[test]
fn primary_docs_have_scannable_no_rust_sections() {
    let root = workspace_root();
    for relative in PRIMARY_NO_RUST_DOCS {
        let source = fs::read_to_string(root.join(relative))
            .unwrap_or_else(|err| panic!("failed to read {relative}: {err}"));
        let sections = primary_no_rust_sections(relative, &source);
        let combined = sections.join("\n");
        for required in ["game.toml", "game-dev preview", "prebuilt executable"] {
            assert!(
                combined.contains(required),
                "{relative} primary no-Rust section should mention {required:?}"
            );
        }
        assert!(
            !combined.contains("assets/game.ron"),
            "{relative} primary no-Rust section must not present assets/game.ron as current"
        );
        assert!(
            !combined.trim_start().starts_with("cargo run"),
            "{relative} primary no-Rust section must not start with cargo run"
        );
        for forbidden in FORBIDDEN_ENGINE_VOCAB {
            assert!(
                !source_contains_forbidden_token(&combined, forbidden),
                "{relative} primary no-Rust section must not expose {forbidden:?}"
            );
        }
    }
}

#[test]
fn legacy_ron_migration_guide_is_documented() {
    let root = workspace_root();
    let index = fs::read_to_string(root.join("docs/migrations/README.md"))
        .expect("failed to read migrations index");
    assert!(
        index.contains("ron-to-toml.md"),
        "migration index should link the RON-to-TOML guide"
    );

    let guide = fs::read_to_string(root.join("docs/migrations/ron-to-toml.md"))
        .expect("failed to read RON-to-TOML migration guide");
    for required in [
        "game-dev migrate-ron assets/game.ron --out game.toml",
        "game-dev check",
        "game-dev preview",
        "Legacy RON",
        "Player((",
        "melee: Some",
        "[[prefab]]",
        "kind = \"player\"",
        "[prefab.melee]",
        "duration",
    ] {
        assert!(
            guide.contains(required),
            "RON-to-TOML guide should contain {required:?}"
        );
    }
}

#[test]
fn legacy_ron_tutorial_is_outside_primary_sequence() {
    let root = workspace_root();
    let start_here = fs::read_to_string(root.join("docs/tutorials/00-start-here.md"))
        .expect("failed to read start-here tutorial");
    assert!(
        !start_here.contains("templates/data-driven-demo --name my-game"),
        "start-here tutorial should not generate the legacy RON template as a default path"
    );
    assert!(
        start_here.contains("existing projects and migration"),
        "start-here tutorial should frame the RON wrapper as legacy/migration"
    );

    let tutorial_index = fs::read_to_string(root.join("docs/tutorials/README.md"))
        .expect("failed to read tutorial index");
    let primary_course = tutorial_index
        .split("## Beginner Rust Course")
        .nth(1)
        .and_then(|rest| rest.split("## Optional Follow-Ups").next())
        .expect("failed to find numbered beginner course section");
    assert!(
        !primary_course.contains("13-data-driven-demo"),
        "legacy RON tutorial should not be in the numbered primary tutorial sequence"
    );

    let legacy_tutorial = fs::read_to_string(root.join("docs/tutorials/13-data-driven-demo.md"))
        .expect("failed to read legacy data-driven tutorial");
    for required in [
        "# Legacy Data-Driven First Game",
        "transitional",
        "RON path",
        "primary no-Rust target is `game.toml`",
    ] {
        assert!(
            legacy_tutorial.contains(required),
            "legacy RON tutorial should keep legacy framing {required:?}"
        );
    }
}

#[test]
fn legacy_ron_tests_are_labeled_legacy() {
    let root = workspace_root();
    let data_tests_mod = fs::read_to_string(root.join("crates/game-kit/src/data/tests/mod.rs"))
        .expect("failed to read data test module");
    assert!(
        data_tests_mod.contains("mod ron_legacy;"),
        "RON data tests should be explicitly labeled as legacy"
    );
    assert!(
        data_tests_mod.contains("mod toml_primary;"),
        "primary TOML tests should stay separate from legacy RON tests"
    );
    assert!(
        root.join("crates/game-kit/src/data/tests/ron_legacy.rs")
            .is_file(),
        "legacy RON tests should live in ron_legacy.rs"
    );
    assert!(
        !root.join("crates/game-kit/src/data/tests.rs").exists(),
        "old unlabeled monolithic data tests file should not come back"
    );
}

#[test]
fn cli_exposes_authoring_scan_command() {
    let cli = read_game_cli_sources();
    for required in [
        "game-dev authoring-scan [--project dir]",
        "fn authoring_scan_command",
        "scan_no_rust_project",
        "FORBIDDEN_DATA_TOKENS",
        "FORBIDDEN_ENGINE_VOCAB",
    ] {
        assert!(
            cli.contains(required),
            "game-cli source should expose primary authoring scan piece {required:?}"
        );
    }
}

fn is_primary_text_surface(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("toml" | "txt" | "md")
    )
}

fn primary_no_rust_sections<'a>(relative: &str, source: &'a str) -> Vec<&'a str> {
    const START: &str = "<!-- primary-no-rust:start -->";
    const END: &str = "<!-- primary-no-rust:end -->";

    let mut sections = Vec::new();
    let mut rest = source;
    while let Some(start) = rest.find(START) {
        let after_start = &rest[start + START.len()..];
        let end = after_start
            .find(END)
            .unwrap_or_else(|| panic!("{relative} missing {END} marker"));
        sections.push(&after_start[..end]);
        rest = &after_start[end + END.len()..];
    }

    assert!(
        !sections.is_empty(),
        "{relative} should mark primary no-Rust docs with {START} / {END}"
    );
    sections
}

fn source_contains_forbidden_token(source: &str, token: &str) -> bool {
    match token {
        "fn " | "impl " | "struct " | "enum " | "trait " | "pub " | "use " | "match " => {
            contains_whole_word(source, token.trim())
        }
        _ if is_identifier_token(token) => contains_whole_word(source, token),
        _ => source.contains(token),
    }
}

fn is_identifier_token(token: &str) -> bool {
    token
        .chars()
        .all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

fn contains_whole_word(source: &str, word: &str) -> bool {
    source.match_indices(word).any(|(start, _)| {
        let before = source[..start].chars().next_back();
        let after = source[start + word.len()..].chars().next();
        before.is_none_or(|ch| !is_word_char(ch)) && after.is_none_or(|ch| !is_word_char(ch))
    })
}

fn is_word_char(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphanumeric()
}

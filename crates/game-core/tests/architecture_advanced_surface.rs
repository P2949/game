mod support;

use std::fs;

use support::*;

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
        "Most demos should stay with the primary `game.toml` package",
        "Advanced Rust authoring is not the primary no-Rust surface.",
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

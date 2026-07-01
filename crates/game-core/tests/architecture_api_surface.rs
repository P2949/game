mod support;

use std::fs;

use support::*;

#[test]
fn game_kit_data_module_files_stay_split() {
    let root = workspace_root();
    let old_monolith = root.join("crates/game-kit/src/data.rs");
    assert!(
        !old_monolith.exists(),
        "game-kit data loader should stay split under crates/game-kit/src/data/"
    );

    let data_dir = root.join("crates/game-kit/src/data");
    let mut files = fs::read_dir(&data_dir)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", data_dir.display()))
        .map(|entry| entry.expect("failed to read data module entry").path())
        .filter(|path| path.extension().and_then(|extension| extension.to_str()) == Some("rs"))
        .collect::<Vec<_>>();
    files.sort();
    assert!(
        files.len() >= 6,
        "game-kit data loader should remain split into focused modules"
    );

    const MAX_DATA_MODULE_LINES: usize = 1_500;
    for path in files {
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        let line_count = source.lines().count();
        assert!(
            line_count <= MAX_DATA_MODULE_LINES,
            "{} has {line_count} lines; keep data modules below {MAX_DATA_MODULE_LINES} lines to avoid rebuilding the old monolith",
            path.display()
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
fn game_kit_root_does_not_reexport_authoring_surface() {
    let source = fs::read_to_string(workspace_root().join("crates/game-kit/src/lib.rs"))
        .expect("failed to read game-kit lib");

    assert!(
        source.contains("pub mod compat"),
        "temporary root compatibility exports should live under game_kit::compat"
    );

    for forbidden in [
        "\npub use app::",
        "\npub use assets::",
        "\npub use beginner::",
        "\npub use bundle::",
        "\npub use context::{Commands",
        "\npub use data::{Beginner",
        "\npub use helpers::",
        "\npub use input::",
        "\npub use map::",
        "\npub use prefab::",
        "\npub use system::",
        "\npub mod helpers;",
    ] {
        assert!(
            !source.contains(forbidden),
            "game-kit root must not expose broad authoring surface through {forbidden:?}"
        );
    }
}

#[test]
fn game_core_root_does_not_reexport_internal_surface() {
    let source = fs::read_to_string(workspace_root().join("crates/game-core/src/lib.rs"))
        .expect("failed to read game-core lib");

    assert!(source.contains("pub mod prelude"));
    assert!(source.contains("pub mod internal_prelude"));

    for forbidden in [
        "\npub use app::",
        "\npub use assets::",
        "\npub use audio::",
        "\npub use backend::",
        "\npub use builder::",
        "\npub use camera::",
        "\npub use commands::",
        "\npub use gfx::",
        "\npub use input::",
        "\npub use nav::",
        "\npub use plugin::",
        "\npub use query::",
        "\npub use schedule::",
        "\npub use tilemap::",
        "\npub use world::",
    ] {
        assert!(
            !source.contains(forbidden),
            "game-core root must not expose raw internals through {forbidden:?}"
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

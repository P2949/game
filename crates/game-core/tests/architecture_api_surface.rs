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
fn data_tests_stay_split_into_focused_modules() {
    let root = workspace_root();
    let old_monolith = root.join("crates/game-kit/src/data/tests.rs");
    assert!(
        !old_monolith.exists(),
        "game-kit data tests should stay split under crates/game-kit/src/data/tests/"
    );

    let tests_dir = root.join("crates/game-kit/src/data/tests");
    for required in ["mod.rs", "toml_primary.rs", "ron_legacy.rs"] {
        assert!(
            tests_dir.join(required).is_file(),
            "data tests should keep focused module {required}"
        );
    }

    const MAX_DATA_TEST_MODULE_LINES: usize = 1_500;
    for entry in fs::read_dir(&tests_dir)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", tests_dir.display()))
    {
        let path = entry.expect("failed to read data test entry").path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
            continue;
        }
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        let line_count = source.lines().count();
        assert!(
            line_count <= MAX_DATA_TEST_MODULE_LINES,
            "{} has {line_count} lines; split data tests before exceeding {MAX_DATA_TEST_MODULE_LINES} lines",
            path.display()
        );
    }
}

#[test]
fn data_parser_modules_stay_split_by_format_and_role() {
    let root = workspace_root();
    let data_dir = root.join("crates/game-kit/src/data");
    let mod_rs = fs::read_to_string(data_dir.join("mod.rs")).expect("failed to read data mod");
    for required in [
        "model.rs",
        "toml_schema.rs",
        "toml_parse.rs",
        "toml_emit.rs",
        "legacy_ron.rs",
        "validate.rs",
        "build.rs",
    ] {
        assert!(
            data_dir.join(required).is_file(),
            "data parser/build/validation code should keep focused module {required}"
        );
        let module_name = required.trim_end_matches(".rs");
        assert!(
            mod_rs.contains(&format!("mod {module_name};")),
            "data mod.rs should wire focused module {module_name}"
        );
    }
}

#[test]
fn cli_no_rust_commands_stay_split() {
    let root = workspace_root();
    let commands_dir = root.join("crates/game-cli/src/commands");
    let lib = fs::read_to_string(root.join("crates/game-cli/src/lib.rs"))
        .expect("failed to read game-cli lib");
    let commands_mod =
        fs::read_to_string(commands_dir.join("mod.rs")).expect("failed to read commands mod");

    for required in [
        "asset_check.rs",
        "authoring_scan.rs",
        "check.rs",
        "migrate_ron.rs",
        "package.rs",
        "package_sdk.rs",
        "preview.rs",
        "validate_data.rs",
    ] {
        assert!(
            commands_dir.join(required).is_file(),
            "game-cli no-Rust command logic should keep focused module {required}"
        );
        let module_name = required.trim_end_matches(".rs");
        assert!(
            commands_mod.contains(&format!("mod {module_name};")),
            "commands/mod.rs should wire focused module {module_name}"
        );
    }
    assert!(
        !lib.contains("fn validate_data_file("),
        "validate-data command implementation should live under commands/validate_data.rs"
    );
    assert!(
        !lib.contains("fn asset_check_at("),
        "asset-check command implementation should live under commands/asset_check.rs"
    );
}

#[test]
fn size_guarded_modules_stay_below_documented_limits() {
    let root = workspace_root();
    assert_rust_files_under_limit("crates/game-cli/src/commands", 1_000);
    assert_rust_files_under_limit("crates/game-core/tests", 1_000);

    let lib = root.join("crates/game-cli/src/lib.rs");
    let line_count = fs::read_to_string(&lib)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", lib.display()))
        .lines()
        .count();
    assert!(
        line_count <= 400,
        "{} has {line_count} lines; keep game-cli lib.rs focused on routing and shared glue",
        lib.display()
    );
}

fn assert_rust_files_under_limit(relative_dir: &str, max_lines: usize) {
    let root = workspace_root();
    let mut files = Vec::new();
    collect_rust_files(&root.join(relative_dir), &mut files);
    assert!(
        !files.is_empty(),
        "{relative_dir} should contain Rust files for size guard checks"
    );
    for path in files {
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        let line_count = source.lines().count();
        assert!(
            line_count <= max_lines,
            "{} has {line_count} lines; split or document an explicit architecture exception before exceeding {max_lines} lines",
            path.display()
        );
    }
}

#[test]
fn game_kit_compatibility_prelude_is_visibly_deprecated() {
    let source = fs::read_to_string(workspace_root().join("crates/game-kit/src/lib.rs"))
        .expect("failed to read game-kit lib");
    let boundary = fs::read_to_string(workspace_root().join("docs/api-boundary.md"))
        .expect("failed to read api boundary");

    assert!(
        source.contains("#[deprecated(note = \"Use game_kit::beginner::prelude::* or game_kit::advanced::prelude::*\")]"),
        "game_kit::prelude should be marked as compatibility-only with a deprecation note"
    );
    assert!(source.contains("Compatibility prelude."));
    assert!(source.contains("game_kit::beginner::prelude::*"));
    assert!(source.contains("game_kit::advanced::prelude::*"));
    for required in [
        "v0.2.x: compatibility prelude exists but deprecated.",
        "v0.3.x: docs/examples/templates must not use it.",
        "v0.4.x or pre-1.0: remove or feature-gate compatibility prelude.",
    ] {
        assert!(
            boundary.contains(required),
            "API boundary should document compatibility policy line {required:?}"
        );
    }
}

#[test]
fn secondary_rust_authoring_surfaces_remain_available() {
    let root = workspace_root();
    let starter = fs::read_to_string(root.join("crates/game-starter/src/lib.rs"))
        .expect("failed to read game-starter lib");
    let beginner = fs::read_to_string(root.join("crates/game-kit/src/beginner/prelude.rs"))
        .expect("failed to read beginner prelude");
    let advanced = fs::read_to_string(root.join("crates/game-kit/src/advanced/prelude.rs"))
        .expect("failed to read advanced prelude");
    let kit = fs::read_to_string(root.join("crates/game-kit/src/lib.rs"))
        .expect("failed to read game-kit lib");

    assert!(
        starter.contains("pub use game_kit::beginner::prelude::*;"),
        "game_starter::prelude::* should keep reexporting the beginner Rust API"
    );
    assert!(
        beginner.contains("pub use crate::content_plugin;"),
        "game_kit::beginner::prelude::* should keep content_plugin! available"
    );
    assert!(
        kit.contains("macro_rules! content_plugin"),
        "content_plugin! should remain defined by game-kit"
    );
    assert!(
        advanced.contains("pub use game_core::query::{"),
        "game_kit::advanced::prelude::* should remain the explicit lower-level Rust surface"
    );
    assert!(
        advanced.contains("pub use crate::context::{Commands, GameCtx, StartupGameCtx};"),
        "advanced prelude should keep explicit advanced context access"
    );
}

#[test]
fn beginner_prelude_does_not_export_no_rust_schema_types() {
    let beginner =
        fs::read_to_string(workspace_root().join("crates/game-kit/src/beginner/prelude.rs"))
            .expect("failed to read beginner prelude");
    let data = fs::read_to_string(workspace_root().join("crates/game-kit/src/data/mod.rs"))
        .expect("failed to read data module");

    for forbidden in [
        "BeginnerAssetsFile",
        "BeginnerControlsFile",
        "BeginnerGameFile",
        "BeginnerMapFile",
        "BeginnerPrefabFile",
        "BeginnerRuleFile",
        "BeginnerScriptRuleFile",
        "RuleEffectFile",
    ] {
        assert!(
            !beginner.contains(forbidden),
            "no-Rust schema type {forbidden} belongs under game_kit::data, not game_kit::beginner::prelude::*"
        );
    }
    assert!(
        data.contains("pub use legacy_ron::*;"),
        "Rust users who need data schema compatibility types should reach them through game_kit::data"
    );
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

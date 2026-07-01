mod support;

use support::*;

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

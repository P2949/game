mod support;

use std::fs;

use support::*;

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

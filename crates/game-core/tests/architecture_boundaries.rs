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
fn game_test_harness_is_not_root_reexported() {
    let source = fs::read_to_string(workspace_root().join("crates/game-kit/src/lib.rs"))
        .expect("failed to read game-kit lib");

    assert!(
        !source.contains("pub use harness::GameTestHarness;"),
        "GameTestHarness should be exposed through game_kit::testing, not the crate root"
    );
}

#[test]
fn simple_content_uses_only_beginner_surface() {
    let findings = forbidden_source_uses("simple-content", BEGINNER_CONTENT_FORBIDDEN);

    assert!(
        findings.is_empty(),
        "simple-content is the pure beginner example and must not use advanced APIs:\n{}",
        findings.join("\n")
    );
}

#[test]
fn migrated_content_reports_remaining_advanced_surface() {
    for crate_name in ["arena-content", "testbed-content"] {
        let findings = forbidden_source_uses(crate_name, BEGINNER_CONTENT_FORBIDDEN);
        if !findings.is_empty() {
            eprintln!(
                "{crate_name} still uses advanced authoring APIs:\n{}",
                findings.join("\n")
            );
        }
    }
}

const BEGINNER_CONTENT_FORBIDDEN: &[&str] = &[
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
    for crate_name in ["arena-content", "testbed-content"] {
        let src_dir = workspace_root().join(format!("crates/{crate_name}/src"));
        let mut files = Vec::new();
        collect_rust_files(&src_dir, &mut files);

        for path in files {
            let source = read_code_without_comments(&path);
            let production = strip_cfg_test_modules(&source);
            assert!(
                production.contains("use game_kit::prelude::*;"),
                "{} should import the authoring facade prelude",
                path.display()
            );

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

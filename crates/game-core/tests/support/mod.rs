#![allow(dead_code)]

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub(crate) const BEGINNER_CONTENT_CRATES: &[&str] = &["simple-content", "arena-content"];
pub(crate) const ADVANCED_CONTENT_CRATES: &[&str] = &["testbed-content"];

pub(crate) const PRIMARY_NO_RUST_PATHS: &[&str] = &[
    "templates/no-rust-demo",
    "examples/no-rust-minimal",
    "examples/no-rust-events",
    "examples/no-rust-waves",
    "examples/no-rust-projectiles",
    "examples/no-rust-full",
    "examples/no-rust-tiled",
];

pub(crate) const PRIMARY_NO_RUST_DOCS: &[&str] = &[
    "README.md",
    "docs/api-boundary.md",
    "docs/no-rust-package-layout.md",
    "docs/beginner-authoring.md",
    "docs/content-authoring.md",
];

pub(crate) const BEGINNER_CONTENT_FORBIDDEN: &[&str] = &[
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

pub(crate) const SIMPLE_CONTENT_FORBIDDEN: &[&str] = &[
    "TextureHandle",
    "SoundHandle",
    "AssetAuthor",
    "game.assets(",
    "struct SimpleAssets",
    "register_assets(",
];

pub(crate) const BEGINNER_DEMO_FORBIDDEN: &[&str] = &[
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

pub(crate) const BEGINNER_DOC_FORBIDDEN: &[&str] = &[
    "game_kit::prelude::*",
    "game_kit::advanced::prelude::*",
    "Transform::",
    "Velocity::",
    "Sprite::new",
    "GameCtx",
    "assets.texture(\"player\")",
    "assets.sound(\"hit\")",
];

pub(crate) fn forbidden_source_uses(crate_name: &str, patterns: &[&str]) -> Vec<String> {
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

pub(crate) fn contains_identifier(source: &str, name: &str) -> bool {
    source
        .split(|ch: char| ch != '_' && !ch.is_ascii_alphanumeric())
        .any(|token| token == name)
}

pub(crate) fn assert_content_avoids_engine_internals(path: &Path, production: &str) {
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
pub(crate) fn read_code_without_comments(path: &Path) -> String {
    let raw = fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    raw.lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Reads a manifest with comment lines stripped, so documentation that mentions a
/// forbidden crate name does not trip a `contains` dependency check.
pub(crate) fn read_manifest_without_comments(relative: &str) -> String {
    let raw = fs::read_to_string(workspace_root().join(relative))
        .unwrap_or_else(|err| panic!("failed to read {relative}: {err}"));
    raw.lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n")
}

pub(crate) fn release_metadata() -> BTreeMap<String, String> {
    let source = fs::read_to_string(workspace_root().join("release.toml"))
        .expect("failed to read release metadata");
    source
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                return None;
            }
            let (key, value) = trimmed.split_once('=')?;
            Some((
                key.trim().to_string(),
                value
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string(),
            ))
        })
        .collect()
}

pub(crate) fn required_release_value(metadata: &BTreeMap<String, String>, key: &str) -> String {
    metadata
        .get(key)
        .unwrap_or_else(|| panic!("release.toml missing {key}"))
        .to_string()
}

pub(crate) fn read_game_cli_sources() -> String {
    [
        "crates/game-cli/src/lib.rs",
        "crates/game-cli/src/assets.rs",
        "crates/game-cli/src/manifest.rs",
        "crates/game-cli/src/paths.rs",
        "crates/game-cli/src/process.rs",
        "crates/game-cli/src/templates.rs",
        "crates/game-cli/src/commands/authoring_scan.rs",
        "crates/game-cli/src/commands/check.rs",
        "crates/game-cli/src/commands/doctor.rs",
        "crates/game-cli/src/commands/migrate_ron.rs",
        "crates/game-cli/src/commands/package.rs",
        "crates/game-cli/src/commands/release_check.rs",
    ]
    .into_iter()
    .map(|relative| {
        fs::read_to_string(workspace_root().join(relative))
            .unwrap_or_else(|err| panic!("failed to read {relative}: {err}"))
    })
    .collect::<Vec<_>>()
    .join("\n")
}

pub(crate) fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("game-core lives under crates/")
}

pub(crate) fn beginner_doc_section(source: &str) -> &str {
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

pub(crate) fn collect_rust_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
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

pub(crate) fn collect_beginner_surface_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
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

pub(crate) fn collect_markdown_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
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

pub(crate) fn collect_all_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    for entry in fs::read_dir(dir).unwrap_or_else(|err| {
        panic!("failed to read directory {}: {err}", dir.display());
    }) {
        let entry = entry.expect("failed to read directory entry");
        let path = entry.path();
        if path.is_dir() {
            collect_all_files(&path, out);
        } else {
            out.push(path);
        }
    }
}

pub(crate) fn extract_pub_module_body<'a>(source: &'a str, module_name: &str) -> &'a str {
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

pub(crate) fn strip_cfg_test_modules(source: &str) -> String {
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

pub(crate) fn apply_brace_delta(depth: usize, line: &str) -> usize {
    let opens = line.chars().filter(|ch| *ch == '{').count();
    let closes = line.chars().filter(|ch| *ch == '}').count();
    depth.saturating_add(opens).saturating_sub(closes)
}

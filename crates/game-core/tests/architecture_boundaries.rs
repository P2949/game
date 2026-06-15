use std::fs;
use std::path::Path;

struct KnownLeak {
    label: &'static str,
    file: &'static str,
    patterns: &'static [&'static str],
}

const KNOWN_LEAKS: &[KnownLeak] = &[
    KnownLeak {
        label: "engine::app -> platform, renderer, audio",
        file: "crates/game-core/src/app.rs",
        patterns: &[
            "crate::platform",
            "crate::renderer",
            "crate::audio::AudioSystem",
        ],
    },
    KnownLeak {
        label: "engine::gfx -> renderer::TextureId",
        file: "crates/game-core/src/gfx.rs",
        patterns: &["crate::renderer", "TextureId"],
    },
    KnownLeak {
        label: "engine::audio -> audio::AudioSystem",
        file: "crates/game-core/src/audio.rs",
        patterns: &["game_audio::AudioSystem"],
    },
    KnownLeak {
        label: "renderer::context -> engine::camera::Camera2D (resolved in Phase 2)",
        file: "crates/game-renderer-vulkan/src/context.rs",
        patterns: &["crate::engine::camera::Camera2D"],
    },
    KnownLeak {
        label: "game::ai tests -> platform::input",
        file: "crates/arena-content/src/ai.rs",
        patterns: &["crate::platform::input"],
    },
    KnownLeak {
        label: "game::combat tests -> platform::input",
        file: "crates/arena-content/src/combat.rs",
        patterns: &["crate::platform::input"],
    },
];

const FUTURE_HARD_GATES: &[&str] = &[
    // TODO(architecture): game-renderer-vulkan must not import arena-content.
    // TODO(architecture): game-audio must not import arena-content.
    // TODO(architecture): game-platform-sdl must not import arena-content.
    // TODO(architecture): game-runtime must not import arena-content except through
    // a plugin trait object passed by the binary.
    "game-renderer-vulkan must not import arena-content",
    "game-audio must not import arena-content",
    "game-platform-sdl must not import arena-content",
    "game-runtime must not import arena-content except through plugin trait object passed by bin",
];

#[test]
fn current_boundary_leaks_are_advisory() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .and_then(Path::parent)
        .expect("game-core lives under crates/");

    for leak in KNOWN_LEAKS {
        let path = workspace_root.join(leak.file);
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        let present = leak
            .patterns
            .iter()
            .filter(|pattern| source.contains(**pattern))
            .count();

        eprintln!(
            "advisory architecture leak: {} ({present}/{}) patterns still present",
            leak.label,
            leak.patterns.len()
        );
    }
}

#[test]
fn future_architecture_gates_are_recorded() {
    assert_eq!(FUTURE_HARD_GATES.len(), 4);
}

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
fn second_demo_only_uses_gameplay_crates() {
    // Phase 12: the testbed demo must prove the split by depending solely on the
    // engine-neutral gameplay crates — never on the runtime/backends or the other
    // demo's content crate.
    let raw = fs::read_to_string(workspace_root().join("crates/testbed-content/Cargo.toml"))
        .expect("failed to read testbed-content manifest");
    // Ignore comment lines so documentation mentioning forbidden crates does not
    // trip the dependency check.
    let manifest: String = raw
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");

    for forbidden in [
        "game-runtime",
        "game-renderer-vulkan",
        "game-audio",
        "game-platform-sdl",
        "arena-content",
        "ash",
        "sdl3",
        "gpu-allocator",
    ] {
        assert!(
            !manifest.contains(forbidden),
            "testbed-content manifest must not depend on {forbidden}"
        );
    }

    for required in [
        "game-core",
        "game-map",
        "game-ai",
        "game-combat",
        "game-physics",
    ] {
        assert!(
            manifest.contains(required),
            "testbed-content manifest should use {required}"
        );
    }
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

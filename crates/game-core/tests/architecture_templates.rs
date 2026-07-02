mod support;

use std::fs;

use support::*;

#[test]
fn generated_templates_are_ci_checked_and_release_pinned() {
    let root = workspace_root();
    let release = release_metadata();
    let current_tag = required_release_value(&release, "current_tag");
    let game_starter_dependency = required_release_value(&release, "game_starter_dependency");
    let ci = fs::read_to_string(root.join(".github/workflows/ci.yml"))
        .expect("failed to read CI workflow");

    for required in [
        "generated-templates:",
        "cargo install cargo-generate --locked --force",
        "cargo generate --path templates/simple-demo",
        "cargo generate --path templates/data-driven-demo",
        "cargo check --manifest-path /tmp/generated/smoke-simple/Cargo.toml --features ci-build-sdl3",
        "cargo check --manifest-path /tmp/generated/smoke-data/Cargo.toml --features ci-build-sdl3",
        "Build game-dev helper",
        "cargo build -p game-cli --locked --features ci-build-sdl3",
        "game-dev check --features ci-build-sdl3",
        "game-dev package --release --features ci-build-sdl3 --out /tmp/package-simple --zip",
        "game-dev package --release --features ci-build-sdl3 --out /tmp/package-data --zip",
        "working-directory: /tmp/generated/smoke-simple",
        "working-directory: /tmp/generated/smoke-data",
        "cargo run --features ci-build-sdl3",
    ] {
        assert!(
            ci.contains(required),
            "generated-project CI must include {required:?}"
        );
    }

    for required in [
        "Check and smoke no-Rust example packages",
        "examples/no-rust-minimal",
        "examples/no-rust-events",
        "examples/no-rust-waves",
        "examples/no-rust-projectiles",
        "examples/no-rust-full",
        "examples/no-rust-tiled",
        "cargo run -p game-cli --features ci-build-sdl3 -- check --project \"$project\"",
        "cargo run -p game-player --locked --features ci-build-sdl3 -- --project \"$project\" --smoke-frames 0",
    ] {
        assert!(
            ci.contains(required),
            "CI should validate no-Rust example package flow with {required:?}"
        );
    }

    for relative in [
        "templates/simple-demo/cargo-generate.toml",
        "templates/data-driven-demo/cargo-generate.toml",
    ] {
        let source = fs::read_to_string(root.join(relative))
            .unwrap_or_else(|err| panic!("failed to read {relative}: {err}"));
        assert!(
            source.contains(&format!("default = '{game_starter_dependency}'")),
            "{relative} should pin release-generated projects to the release tag"
        );
        assert!(
            source.contains(&format!(r#"tag = "{current_tag}""#)),
            "{relative} should use the current release tag from release.toml"
        );
        assert!(
            !source.contains(
                r#"default = '{ git = "https://github.com/P2949/game", package = "game-starter" }'"#
            ),
            "{relative} must not default to the moving git branch"
        );
        assert!(
            !source.contains("rev ="),
            "{relative} should not use the release-candidate revision after tag selection"
        );
    }

    for relative in [
        "templates/simple-demo/Cargo.toml",
        "templates/data-driven-demo/Cargo.toml",
    ] {
        let source = fs::read_to_string(root.join(relative))
            .unwrap_or_else(|err| panic!("failed to read {relative}: {err}"));
        assert!(
            source.contains(r#"ci-build-sdl3 = ["game-starter/ci-build-sdl3"]"#),
            "{relative} should let generated CI opt into source-built SDL3"
        );
        assert!(
            source.contains("[package.metadata.game]")
                && source.contains(r#"title = "{{title}}""#)
                && source.contains(r#"asset_dir = "assets""#),
            "{relative} should include beginner package metadata"
        );
    }

    let starter_manifest = fs::read_to_string(root.join("crates/game-starter/Cargo.toml"))
        .expect("failed to read game-starter manifest");
    assert!(
        starter_manifest.contains(r#"ci-build-sdl3 = ["game-runtime/ci-build-sdl3"]"#),
        "game-starter should expose the generated-project CI SDL3 feature"
    );

    let runtime_manifest = fs::read_to_string(root.join("crates/game-runtime/Cargo.toml"))
        .expect("failed to read game-runtime manifest");
    assert!(
        runtime_manifest.contains("game-platform-sdl/ci-build-sdl3")
            && runtime_manifest.contains("game-audio/ci-build-sdl3"),
        "game-runtime should forward the source-built SDL3 feature to backend crates"
    );

    for relative in [
        "crates/game-audio/Cargo.toml",
        "crates/game-platform-sdl/Cargo.toml",
    ] {
        let source = fs::read_to_string(root.join(relative))
            .unwrap_or_else(|err| panic!("failed to read {relative}: {err}"));
        assert!(
            source.contains(r#"ci-build-sdl3 = ["sdl3/build-from-source"]"#),
            "{relative} should source-build SDL3 for CI"
        );
    }
}

#[test]
fn architecture_no_rust_template_contains_no_rust_project_files() {
    let root = workspace_root();
    let template = root.join("templates/no-rust-demo");
    assert!(template.join("game.toml").is_file());
    assert!(template.join("README.txt").is_file());
    assert!(template.join("assets/maps/level-1.txt").is_file());

    for forbidden in [
        "Cargo.toml",
        "build.rs",
        "src/main.rs",
        "cargo-generate.toml",
    ] {
        assert!(
            !template.join(forbidden).exists(),
            "templates/no-rust-demo must not contain {forbidden}"
        );
    }

    let mut files = Vec::new();
    collect_files(&template, &mut files);
    for path in files {
        assert_ne!(
            path.extension().and_then(|extension| extension.to_str()),
            Some("rs"),
            "templates/no-rust-demo must not contain Rust source: {}",
            path.display()
        );
    }

    let game_toml =
        fs::read_to_string(template.join("game.toml")).expect("failed to read no-rust game.toml");
    for required in [
        "kind = \"player\"",
        "kind = \"enemy\"",
        "kind = \"pickup\"",
        "preset = \"top-down\"",
        "\"top-down-controls\"",
        "\"player-collects-pickups\"",
    ] {
        assert!(
            game_toml.contains(required),
            "templates/no-rust-demo/game.toml should include {required:?}"
        );
    }
    for forbidden in [
        "Some(",
        "Player((",
        "Enemy((",
        "Pickup((",
        "TopDownControls",
        "PlayerCollectsPickups",
        "::",
        "fn ",
        "impl ",
        "struct ",
        "enum ",
        "pub ",
        "Result",
    ] {
        assert!(
            !game_toml.contains(forbidden),
            "templates/no-rust-demo/game.toml must not contain {forbidden:?}"
        );
    }
}

#[test]
fn architecture_no_rust_examples_are_plain_packages() {
    let root = workspace_root();
    for relative in [
        "examples/no-rust-minimal",
        "examples/no-rust-events",
        "examples/no-rust-waves",
        "examples/no-rust-projectiles",
        "examples/no-rust-full",
        "examples/no-rust-tiled",
    ] {
        let package = root.join(relative);
        assert!(
            package.join("game.toml").is_file(),
            "{relative} should contain primary game.toml"
        );
        assert!(
            package.join("assets").is_dir(),
            "{relative} should contain assets/"
        );
        for forbidden in [
            "Cargo.toml",
            "Cargo.lock",
            "build.rs",
            "src/main.rs",
            "assets/game.ron",
        ] {
            assert!(
                !package.join(forbidden).exists(),
                "{relative} must not contain {forbidden}"
            );
        }

        let mut files = Vec::new();
        collect_files(&package, &mut files);
        for path in files {
            assert_ne!(
                path.extension().and_then(|extension| extension.to_str()),
                Some("rs"),
                "{relative} must not contain Rust source: {}",
                path.display()
            );
        }
    }
}

#[test]
fn architecture_game_player_is_prebuilt_no_rust_runner_only() {
    let root = workspace_root();
    let workspace_manifest =
        fs::read_to_string(root.join("Cargo.toml")).expect("failed to read workspace manifest");
    assert!(
        workspace_manifest.contains(r#"default-members = ["bin/game", "bin/game-player"]"#),
        "release/default builds should include game-player"
    );
    assert!(
        workspace_manifest.contains(r#""bin/game-player""#),
        "workspace members should include game-player"
    );

    let manifest = fs::read_to_string(root.join("bin/game-player/Cargo.toml"))
        .expect("failed to read game-player manifest");
    for required in ["game-runtime", "game-kit"] {
        assert!(
            manifest.contains(required),
            "game-player manifest should depend on {required}"
        );
    }
    for required in [
        r#"ogg = ["game-runtime/ogg"]"#,
        r#"mp3 = ["game-runtime/mp3"]"#,
    ] {
        assert!(
            manifest.contains(required),
            "game-player should forward optional audio feature {required:?}"
        );
    }
    for forbidden in [
        "arena-content",
        "testbed-content",
        "simple-content",
        "data-driven",
    ] {
        assert!(
            !manifest.contains(forbidden),
            "game-player manifest must not depend on content crate {forbidden:?}"
        );
    }

    let source = read_code_without_comments(&root.join("bin/game-player/src/main.rs"));
    assert!(source.contains("plugin_fn"));
    assert!(source.contains("load_authoring_file_with_asset_root"));
    for forbidden in [
        "arena_content",
        "testbed_content",
        "simple_content",
        "GAME_DEMO",
    ] {
        assert!(
            !source.contains(forbidden),
            "game-player source must not import/select content crate {forbidden:?}"
        );
    }
}

fn collect_files(root: &std::path::Path, files: &mut Vec<std::path::PathBuf>) {
    for entry in fs::read_dir(root).unwrap_or_else(|error| {
        panic!(
            "failed to read template directory '{}': {error}",
            root.display()
        )
    }) {
        let path = entry.expect("failed to read template entry").path();
        if path.is_dir() {
            collect_files(&path, files);
        } else {
            files.push(path);
        }
    }
}

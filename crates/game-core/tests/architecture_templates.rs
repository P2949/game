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

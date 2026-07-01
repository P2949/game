mod support;

use std::fs;

use support::*;

#[test]
fn release_dependency_strings_match_release_metadata() {
    let root = workspace_root();
    let release = release_metadata();
    let current_tag = required_release_value(&release, "current_tag");
    let game_starter_dependency = required_release_value(&release, "game_starter_dependency");
    assert_eq!(
        game_starter_dependency,
        format!(
            r#"{{ git = "https://github.com/P2949/game", tag = "{current_tag}", package = "game-starter" }}"#
        ),
        "release.toml should keep the tag and game-starter dependency in sync"
    );

    let cli_templates = fs::read_to_string(root.join("crates/game-cli/src/templates.rs"))
        .expect("failed to read CLI templates module");
    assert!(
        cli_templates.contains(&game_starter_dependency),
        "game-cli RELEASE_GAME_STARTER_DEPENDENCY should match release.toml"
    );

    for relative in [
        "templates/simple-demo/cargo-generate.toml",
        "templates/data-driven-demo/cargo-generate.toml",
    ] {
        let source = fs::read_to_string(root.join(relative))
            .unwrap_or_else(|err| panic!("failed to read {relative}: {err}"));
        assert!(
            source.contains(&format!("default = '{game_starter_dependency}'")),
            "{relative} should match release.toml"
        );
    }

    for relative in [
        "README.md",
        "docs/distribution-policy.md",
        "docs/release-checklist.md",
    ] {
        let source = fs::read_to_string(root.join(relative))
            .unwrap_or_else(|err| panic!("failed to read {relative}: {err}"));
        assert!(
            source.contains(&current_tag),
            "{relative} should mention the current release tag from release.toml"
        );
    }

    for relative in [
        "docs/distribution-policy.md",
        "docs/migrations/v0.1-to-v0.2.md",
    ] {
        let source = fs::read_to_string(root.join(relative))
            .unwrap_or_else(|err| panic!("failed to read {relative}: {err}"));
        assert!(
            source.contains(&format!(r#"tag = "{current_tag}""#))
                && source.contains(r#"package = "game-starter""#),
            "{relative} should keep the documented game-starter tag in sync"
        );
    }
}

#[test]
fn ci_permissions_and_release_checklist_name_boundary_gates() {
    let root = workspace_root();
    let ci = fs::read_to_string(root.join(".github/workflows/ci.yml"))
        .expect("failed to read CI workflow");
    assert!(
        ci.contains("permissions:\n  contents: read"),
        "CI should declare least-privilege read permissions"
    );
    assert!(
        ci.contains("cargo test --workspace --locked --features game/ci-build-sdl3"),
        "workspace tests should keep architecture, CLI asset, command policy, data split, and map boundary tests in the normal CI gate"
    );

    let checklist = fs::read_to_string(root.join("docs/release-checklist.md"))
        .expect("failed to read release checklist");
    for required in [
        "`docs/api-boundary.md` reviewed",
        "`game-kit` root export guard passed",
        "`game-core` root export guard passed",
        "unknown asset extension check verified",
        "command error policy tests passed",
        "map transition boundary tests passed",
        "audio/docs consistency checked",
    ] {
        assert!(
            checklist.contains(required),
            "release checklist should name boundary gate {required:?}"
        );
    }
}

#[test]
fn distribution_policy_keeps_tagged_git_model_explicit() {
    let root = workspace_root();
    let release = release_metadata();
    let current_tag = required_release_value(&release, "current_tag");
    let policy = fs::read_to_string(root.join("docs/distribution-policy.md"))
        .expect("failed to read distribution policy");

    for required in [
        "tagged Git dependency model",
        "published release tag",
        current_tag.as_str(),
        "cargo xtask new-demo",
        "Prebuilt demo zips",
        "verified Linux and Windows demo zips",
        "Vulkan-capable GPU/driver",
        "publish crates.io packages after the beginner API stabilizes",
        "dedicated `game-template` repository",
        "version docs per release",
        "platform installer for `game-dev`",
    ] {
        assert!(
            policy.contains(required),
            "distribution policy should document {required:?}"
        );
    }

    let readme = fs::read_to_string(root.join("README.md")).expect("failed to read README");
    for required in [
        format!("current templates pin the published `{current_tag}` release tag"),
        "external generated projects resolve the same checked release".to_string(),
        "cargo xtask new-demo".to_string(),
        "distribution policy".to_string(),
    ] {
        assert!(
            readme.contains(&required),
            "README should link the distribution policy and explain {required:?}"
        );
    }

    let checklist = fs::read_to_string(root.join("docs/release-checklist.md"))
        .expect("failed to read release checklist");
    for required in [
        "generated-template dependency pins target the intended release tag",
        "`CHANGELOG.md` updated",
        "migration docs in `docs/migrations/` updated",
        "generated-template CI is green",
        "first-15-minutes CI is green",
        "release artifacts generated",
        "distribution policy",
        "Status: intentionally deferred",
    ] {
        assert!(
            checklist.contains(required),
            "release checklist should document distribution gate {required:?}"
        );
    }
}

#[test]
fn first_15_minutes_acceptance_path_is_scripted_and_documented() {
    let root = workspace_root();
    let script = fs::read_to_string(root.join("scripts/first-15-minutes.sh"))
        .expect("failed to read first 15 minutes script");
    for required in [
        "cargo generate --path \"$repo/templates/simple-demo\"",
        "--name first-demo",
        "cargo check",
        "GAME_SMOKE_FRAMES=\"$smoke_frames\" run_smoke cargo run",
        "assets/maps/level_1.txt",
        "$game_dev\" asset-check",
        "$game_dev\" package --release",
        "--out dist/first-demo --zip",
        "package_name=",
        "packaged executable for $package_name was not found",
        "dist/first-demo/run.sh",
    ] {
        assert!(
            script.contains(required),
            "first 15 minutes script should contain {required:?}"
        );
    }

    let ci = fs::read_to_string(root.join(".github/workflows/ci.yml"))
        .expect("failed to read CI workflow");
    for required in [
        "First 15 minutes acceptance test",
        "FIRST15_FEATURES: ci-build-sdl3",
        "FIRST15_USE_XVFB: \"1\"",
        "scripts/first-15-minutes.sh",
    ] {
        assert!(
            ci.contains(required),
            "CI should run first 15 minutes acceptance with {required:?}"
        );
    }

    let quickstart = fs::read_to_string(root.join("docs/tutorials/quickstart-zero-to-demo.md"))
        .expect("failed to read quickstart tutorial");
    for required in [
        "first 15 minutes",
        "cargo generate --path templates/simple-demo --name first-demo --destination /tmp",
        "GAME_SMOKE_FRAMES=60 cargo run",
        "assets/maps/level_1.txt",
        "game-dev asset-check",
        "game-dev package --release --out dist/first-demo --zip",
    ] {
        assert!(
            quickstart.contains(required),
            "quickstart should document first 15 minutes command {required:?}"
        );
    }
}

#[test]
fn standalone_beginner_cli_is_documented_and_xtask_wrapped() {
    let root = workspace_root();
    let workspace_manifest =
        fs::read_to_string(root.join("Cargo.toml")).expect("failed to read workspace manifest");
    assert!(
        workspace_manifest.contains(r#""crates/game-cli""#),
        "workspace should include the standalone beginner CLI crate"
    );

    let cli_manifest = fs::read_to_string(root.join("crates/game-cli/Cargo.toml"))
        .expect("failed to read game-cli manifest");
    assert!(cli_manifest.contains("name = \"game-cli\""));
    assert!(cli_manifest.contains("name = \"game-dev\""));
    assert!(
        cli_manifest.contains(r#"ci-build-sdl3 = ["game-audio/ci-build-sdl3"]"#),
        "game-cli CI should be able to source-build SDL3 through game-audio"
    );
    let cli = read_game_cli_sources();
    for required in [
        "game-dev check [--features feature-list]",
        "fn check_project",
        "validate_assets_dir(&assets, false)",
        "validate_beginner_game_file",
        "cargo check failed",
        "fn beginner_failure_advice",
        "If this looks like a setup issue:",
        "If this looks like an asset/data issue:",
        "docs/tutorials/common-errors.md",
        "game-dev doctor --explain",
        "cargo xtask release-check [--skip-smoke] [--skip-generated] [--features feature-list]",
        "fn run_release_check",
        "fn run_generated_release_checks",
        "fn run_smoke_release_checks",
        "fn run_command",
    ] {
        assert!(
            cli.contains(required),
            "game-cli should include {required:?}"
        );
    }

    let xtask_manifest =
        fs::read_to_string(root.join("xtask/Cargo.toml")).expect("failed to read xtask manifest");
    let xtask_main =
        fs::read_to_string(root.join("xtask/src/main.rs")).expect("failed to read xtask main");
    assert!(
        xtask_manifest.contains("game-cli"),
        "xtask should depend on the shared CLI implementation"
    );
    assert!(
        xtask_main.contains("game_cli::run_xtask"),
        "xtask should remain a thin wrapper around game-cli functions"
    );

    for relative in [
        "templates/simple-demo/README.md",
        "templates/data-driven-demo/README.md",
    ] {
        let source = fs::read_to_string(root.join(relative))
            .unwrap_or_else(|err| panic!("failed to read {relative}: {err}"));
        for required in [
            "cargo install --git https://github.com/P2949/game game-cli",
            "game-dev doctor",
            "game-dev check",
            "game-dev run",
            "game-dev package --release --out dist/my-game --zip",
        ] {
            assert!(
                source.contains(required),
                "{relative} should document {required:?}"
            );
        }
    }

    let readme = fs::read_to_string(root.join("README.md")).expect("failed to read README");
    assert!(
        readme.contains("cargo xtask release-check --skip-smoke"),
        "README should document the contributor release-check command"
    );
    let release_checklist = fs::read_to_string(root.join("docs/release-checklist.md"))
        .expect("failed to read release checklist");
    assert!(
        release_checklist.contains("cargo xtask release-check")
            && release_checklist.contains("--skip-smoke"),
        "release checklist should document release-check and the local smoke escape hatch"
    );
    let common_errors = fs::read_to_string(root.join("docs/tutorials/common-errors.md"))
        .expect("failed to read common errors guide");
    for required in [
        "If this looks like a setup issue:",
        "game-dev doctor --explain",
        "If this looks like an asset/data issue:",
        "game-dev asset-check",
        "game-dev validate-data assets/game.ron",
        "Rule `projectiles_damage_enemies` needs the `projectiles` rule",
        "restart required",
        ".object(\"Slime\", \"slime\")",
        "objects: {\"Slime\": \"slime\"}",
        "game-starter",
        "git tag",
        "git rev",
        "ci-build-sdl3",
        "GAME_ASSET_DIR=examples/tiled-demo/assets",
    ] {
        assert!(
            common_errors.contains(required),
            "common errors guide should mirror diagnostic wording {required:?}"
        );
    }
}

#[test]
fn generated_project_package_flow_matches_beginner_contract() {
    let root = workspace_root();
    let cli = read_game_cli_sources();
    for required in [
        "cargo build",
        "--release",
        "validate_assets_dir(&assets, false)",
        "validate_beginner_game_file",
        "run.sh",
        "run.ps1",
        "README.txt",
        "zip_package",
        "copy_runtime_libraries",
        "LD_LIBRARY_PATH",
        "libSDL3.so.0",
    ] {
        assert!(
            cli.contains(required),
            "game-dev package should include {required:?}"
        );
    }

    let tutorial = fs::read_to_string(root.join("docs/tutorials/10-package-your-demo.md"))
        .expect("failed to read package tutorial");
    for required in [
        "game-dev package --release --out dist/my-game --zip",
        "run.ps1",
        "README.txt",
        "dist/my-game.zip",
        "Send the whole dist/my-game.zip folder to a friend.",
        "runtime libraries",
    ] {
        assert!(
            tutorial.contains(required),
            "package tutorial should document {required:?}"
        );
    }
}

#[test]
fn release_workflow_publishes_prebuilt_demo_artifacts() {
    let root = workspace_root();
    let workflow = fs::read_to_string(root.join(".github/workflows/release.yml"))
        .expect("failed to read release workflow");
    for required in [
        "push:",
        "tags:",
        "\"v*\"",
        "game-demo-linux-x86_64",
        "game-demo-windows-x86_64",
        "cargo run -p xtask --features ci-build-sdl3 -- package-demo --release --features ci-build-sdl3",
        "Verify Linux package archive",
        "scripts/verify-release-artifact.sh",
        "Verify Windows package archive",
        "Expand-Archive",
        "actions/upload-artifact",
        "gh release upload",
    ] {
        assert!(
            workflow.contains(required),
            "release workflow should include {required:?}"
        );
    }

    let verifier = fs::read_to_string(root.join("scripts/verify-release-artifact.sh"))
        .expect("failed to read release artifact verifier");
    for required in [
        "usage: scripts/verify-release-artifact.sh <archive.zip> <linux|windows>",
        "game",
        "game.exe",
        "libSDL3.so.0",
        "SDL3.dll",
        "run.sh",
        "run.ps1",
        "run.bat",
        "README.txt",
        "assets/fonts/DejaVuSans.ttf",
        "assets/game.ron",
        "assets/maps/tiled_demo.tmx",
        "assets/textures/test.png",
        "assets/sounds/hit.wav",
    ] {
        assert!(
            verifier.contains(required),
            "release artifact verifier should require {required:?}"
        );
    }

    let github_verifier =
        fs::read_to_string(root.join("scripts/verify-github-release-artifacts.sh"))
            .expect("failed to read GitHub release artifact verifier");
    for required in [
        "usage: scripts/verify-github-release-artifacts.sh [run-id|latest]",
        "GH_REPO",
        "RELEASE_WORKFLOW",
        "gh run list",
        "gh run download",
        "game-demo-linux-x86_64",
        "game-demo-windows-x86_64",
        "verify-release-artifact.sh",
    ] {
        assert!(
            github_verifier.contains(required),
            "GitHub release artifact verifier should include {required:?}"
        );
    }

    let cli = read_game_cli_sources();
    assert!(
        cli.contains(
            "cargo xtask package-demo --release --out <directory> [--features feature-list]"
        ),
        "workspace demo packaging should document feature flags for release builds"
    );

    let xtask_manifest =
        fs::read_to_string(root.join("xtask/Cargo.toml")).expect("failed to read xtask manifest");
    for required in ["[features]", "ci-build-sdl3 = [\"game-cli/ci-build-sdl3\"]"] {
        assert!(
            xtask_manifest.contains(required),
            "xtask should forward release workflow features with {required:?}"
        );
    }

    let readme = fs::read_to_string(root.join("README.md")).expect("failed to read README.md");
    for required in [
        "Want to try before building? Download the latest demo package from",
        "Releases",
        "game-demo-linux-x86_64.zip",
        "game-demo-windows-x86_64.zip",
        "Vulkan-capable GPU/driver",
        "source builds remain the main",
    ] {
        assert!(
            readme.contains(required),
            "README should document prebuilt release artifacts with {required:?}"
        );
    }

    for relative in [
        "docs/setup/linux.md",
        "docs/setup/windows.md",
        "docs/setup/macos.md",
        "docs/release-checklist.md",
    ] {
        let source = fs::read_to_string(root.join(relative))
            .unwrap_or_else(|err| panic!("failed to read {relative}: {err}"));
        assert!(
            source.contains("Prebuilt demo package")
                || source.contains("Prebuilt demo release artifacts"),
            "{relative} should document the prebuilt demo path"
        );
        assert!(
            source.contains("Releases") && source.contains("Vulkan-capable GPU/driver"),
            "{relative} should link releases and keep Vulkan expectations honest"
        );
    }

    let checklist = fs::read_to_string(root.join("docs/release-checklist.md"))
        .expect("failed to read release checklist");
    for required in [
        "gh run list --workflow release.yml --limit 5",
        "scripts/verify-github-release-artifacts.sh <run-id>",
        "latest successful `release.yml` run",
        "cargo run -p xtask --features ci-build-sdl3 -- package-demo --release --features ci-build-sdl3",
    ] {
        assert!(
            checklist.contains(required),
            "release checklist should document {required:?}"
        );
    }
}

#[test]
fn doctor_diagnostics_cover_first_run_setup() {
    let root = workspace_root();
    let cli = read_game_cli_sources();
    for required in [
        "rustc 1.87 or newer",
        "Vulkan 1.3+ physical device",
        "SDL3 development files",
        "audio backend prerequisites",
        "assets/fonts/DejaVuSans.ttf",
        "--explain",
    ] {
        assert!(
            cli.contains(required),
            "game-dev doctor should cover {required:?}"
        );
    }

    let ci = fs::read_to_string(root.join(".github/workflows/ci.yml"))
        .expect("failed to read CI workflow");
    assert!(
        ci.contains("cargo run -p game-cli --features ci-build-sdl3 -- doctor --explain"),
        "CI should run doctor diagnostics"
    );

    for relative in [
        "docs/setup/linux.md",
        "docs/setup/macos.md",
        "docs/setup/windows.md",
        "docs/tutorials/common-errors.md",
        "README.md",
    ] {
        let source = fs::read_to_string(root.join(relative))
            .unwrap_or_else(|err| panic!("failed to read {relative}: {err}"));
        assert!(
            source.contains("game-dev doctor --explain"),
            "{relative} should mention the explanatory doctor mode"
        );
    }
}

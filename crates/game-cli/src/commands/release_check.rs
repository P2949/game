use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, anyhow, bail};

use crate::commands::package::package_project_at;
use crate::process::{add_features, cargo_run_game_cli, run_command, workspace_feature_names};
use crate::templates::{DemoTemplate, new_project};

struct ReleaseCheckOptions {
    skip_smoke: bool,
    skip_generated: bool,
    features: Vec<String>,
}

pub(crate) fn release_check_command(
    args: impl Iterator<Item = String>,
    workspace: &Path,
) -> Result<()> {
    let options = parse_release_check_options(args)?;
    run_release_check(workspace, &options)
}

fn parse_release_check_options(
    mut args: impl Iterator<Item = String>,
) -> Result<ReleaseCheckOptions> {
    let mut skip_smoke = false;
    let mut skip_generated = false;
    let mut features = Vec::new();
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--skip-smoke" => skip_smoke = true,
            "--skip-generated" => skip_generated = true,
            "--features" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow!("--features needs a comma-separated feature list"))?;
                features.push(value);
            }
            other => bail!(
                "unknown release-check argument '{other}'; expected --skip-smoke, --skip-generated, or --features <list>"
            ),
        }
    }
    Ok(ReleaseCheckOptions {
        skip_smoke,
        skip_generated,
        features,
    })
}

fn run_release_check(workspace: &Path, options: &ReleaseCheckOptions) -> Result<()> {
    let workspace_features = workspace_feature_names(&options.features);

    let mut fmt = Command::new("cargo");
    fmt.args(["fmt", "--all", "--", "--check"])
        .current_dir(workspace);
    run_command(&mut fmt, "cargo fmt --all -- --check")?;

    let mut test = Command::new("cargo");
    test.args(["test", "--workspace", "--locked"])
        .current_dir(workspace);
    add_features(&mut test, &workspace_features);
    run_command(&mut test, "cargo test --workspace --locked")?;

    let mut headless = Command::new("cargo");
    headless
        .args([
            "test",
            "-p",
            "game-runtime",
            "--test",
            "headless_runner",
            "--no-default-features",
            "--locked",
        ])
        .current_dir(workspace);
    run_command(
        &mut headless,
        "cargo test -p game-runtime --test headless_runner --no-default-features --locked",
    )?;

    let mut clippy = Command::new("cargo");
    clippy
        .args(["clippy", "--workspace", "--all-targets", "--locked"])
        .current_dir(workspace);
    add_features(&mut clippy, &workspace_features);
    clippy.args(["--", "-D", "warnings"]);
    run_command(
        &mut clippy,
        "cargo clippy --workspace --all-targets --locked -- -D warnings",
    )?;

    let mut build = Command::new("cargo");
    build
        .args(["build", "-p", "game", "--release", "--locked"])
        .current_dir(workspace);
    add_features(&mut build, &options.features);
    run_command(&mut build, "cargo build -p game --release --locked")?;

    let mut doctor = cargo_run_game_cli(workspace, &options.features);
    doctor.args(["doctor", "--explain"]);
    run_command(&mut doctor, "cargo run -p game-cli -- doctor --explain")?;

    let mut asset_check = cargo_run_game_cli(workspace, &options.features);
    asset_check.arg("asset-check");
    run_command(&mut asset_check, "cargo run -p game-cli -- asset-check")?;

    let mut validate_data = cargo_run_game_cli(workspace, &options.features);
    validate_data.args(["validate-data", "assets/game.ron"]);
    run_command(
        &mut validate_data,
        "cargo run -p game-cli -- validate-data assets/game.ron",
    )?;

    if options.skip_generated {
        println!("==> skipping generated-project checks");
    } else {
        run_generated_release_checks(workspace, options)?;
    }

    if options.skip_smoke {
        println!("==> skipping graphical smoke checks");
    } else {
        run_smoke_release_checks(workspace, &options.features)?;
    }

    println!("release check passed");
    Ok(())
}

fn run_generated_release_checks(workspace: &Path, options: &ReleaseCheckOptions) -> Result<()> {
    let root = env::temp_dir().join("game-release-check/generated");
    if root.exists() {
        fs::remove_dir_all(&root)
            .with_context(|| format!("failed to remove '{}'", root.display()))?;
    }
    fs::create_dir_all(&root).with_context(|| format!("failed to create '{}'", root.display()))?;

    let dependency = format!(
        r#"{{ path = "{}" }}"#,
        workspace.join("crates/game-starter").display()
    );
    let simple = root.join("simple");
    let data = root.join("data");
    new_project(&simple, DemoTemplate::Simple, &dependency)?;
    new_project(&data, DemoTemplate::DataDriven, &dependency)?;

    run_generated_project_release_checks(workspace, &simple, "simple", &options.features)?;
    run_generated_project_release_checks(workspace, &data, "data-driven", &options.features)
}

fn run_generated_project_release_checks(
    workspace: &Path,
    project: &Path,
    label: &str,
    features: &[String],
) -> Result<()> {
    let mut check = Command::new("cargo");
    check.arg("check").current_dir(project);
    add_features(&mut check, features);
    run_command(
        &mut check,
        &format!("cargo check ({label} generated project)"),
    )?;

    let mut game_dev_check = Command::new("cargo");
    game_dev_check
        .args(["run", "--manifest-path"])
        .arg(workspace.join("Cargo.toml"))
        .args(["-p", "game-cli"])
        .current_dir(project);
    add_features(&mut game_dev_check, features);
    game_dev_check.args(["--", "check"]);
    add_features(&mut game_dev_check, features);
    run_command(
        &mut game_dev_check,
        &format!("game-dev check ({label} generated project)"),
    )?;

    package_project_at(
        project,
        &PathBuf::from(format!("dist/{label}-release-check")),
        true,
        features,
    )
    .with_context(|| format!("failed to package {label} generated project"))
}

fn run_smoke_release_checks(workspace: &Path, features: &[String]) -> Result<()> {
    let mut default_game = Command::new("cargo");
    default_game
        .args(["run", "-p", "game", "--locked"])
        .env("GAME_SMOKE_FRAMES", "120")
        .current_dir(workspace);
    add_features(&mut default_game, features);
    run_command(
        &mut default_game,
        "GAME_SMOKE_FRAMES=120 cargo run -p game --locked",
    )?;

    let mut simple_game = Command::new("cargo");
    simple_game
        .args(["run", "-p", "game", "--locked"])
        .env("GAME_DEMO", "simple")
        .env("GAME_SMOKE_FRAMES", "120")
        .current_dir(workspace);
    add_features(&mut simple_game, features);
    run_command(
        &mut simple_game,
        "GAME_DEMO=simple GAME_SMOKE_FRAMES=120 cargo run -p game --locked",
    )?;

    let mut testbed_game = Command::new("cargo");
    testbed_game
        .args(["run", "-p", "game", "--locked"])
        .env("GAME_DEMO", "testbed")
        .env("GAME_SMOKE_FRAMES", "120")
        .current_dir(workspace);
    add_features(&mut testbed_game, features);
    run_command(
        &mut testbed_game,
        "GAME_DEMO=testbed GAME_SMOKE_FRAMES=120 cargo run -p game --locked",
    )?;

    let mut release_game = Command::new("cargo");
    release_game
        .args(["run", "-p", "game", "--release", "--locked"])
        .env("GAME_ASSET_DIR", "assets")
        .env("GAME_SMOKE_FRAMES", "120")
        .current_dir(workspace);
    add_features(&mut release_game, features);
    run_command(
        &mut release_game,
        "GAME_ASSET_DIR=assets GAME_SMOKE_FRAMES=120 cargo run -p game --release --locked",
    )?;

    let mut tiled = Command::new("cargo");
    tiled
        .args(["run", "-p", "tiled-demo", "--locked"])
        .env("GAME_ASSET_DIR", "examples/tiled-demo/assets")
        .env("GAME_SMOKE_FRAMES", "60")
        .current_dir(workspace);
    add_features(&mut tiled, features);
    run_command(
        &mut tiled,
        "GAME_ASSET_DIR=examples/tiled-demo/assets GAME_SMOKE_FRAMES=60 cargo run -p tiled-demo --locked",
    )?;

    let mut data_driven_tiled = Command::new("cargo");
    data_driven_tiled
        .args(["run", "-p", "data-driven-tiled-demo", "--locked"])
        .env("GAME_ASSET_DIR", "examples/data-driven-tiled-demo/assets")
        .env("GAME_SMOKE_FRAMES", "60")
        .current_dir(workspace);
    add_features(&mut data_driven_tiled, features);
    run_command(
        &mut data_driven_tiled,
        "GAME_ASSET_DIR=examples/data-driven-tiled-demo/assets GAME_SMOKE_FRAMES=60 cargo run -p data-driven-tiled-demo --locked",
    )
}

use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};

pub(crate) fn run_project() -> Result<()> {
    let status = Command::new("cargo")
        .arg("run")
        .status()
        .context("could not run `cargo run`")?;
    if !status.success() {
        bail!("cargo run failed.\n\n{}", beginner_failure_advice());
    }
    Ok(())
}

pub(crate) fn beginner_failure_advice() -> &'static str {
    "If this looks like a setup issue:\n    game-dev doctor --explain\n\nIf this looks like an asset/data issue:\n    game-dev asset-check\n    game-dev validate-data assets/game.ron\n\nSee:\n    docs/tutorials/common-errors.md"
}

pub(crate) fn cargo_run_game_cli(workspace: &Path, features: &[String]) -> Command {
    let mut command = Command::new("cargo");
    command
        .args(["run", "-p", "game-cli"])
        .current_dir(workspace);
    add_features(&mut command, features);
    command.arg("--");
    command
}

pub(crate) fn workspace_feature_names(features: &[String]) -> Vec<String> {
    features
        .iter()
        .map(|feature| {
            if feature.contains('/') {
                feature.clone()
            } else {
                format!("game/{feature}")
            }
        })
        .collect()
}

pub(crate) fn add_features(command: &mut Command, features: &[String]) {
    for feature in features {
        command.arg("--features").arg(feature);
    }
}

pub(crate) fn run_command(command: &mut Command, label: &str) -> Result<()> {
    println!("==> {label}");
    let status = command
        .status()
        .with_context(|| format!("could not run `{label}`"))?;
    if !status.success() {
        bail!("`{label}` failed with {status}");
    }
    Ok(())
}

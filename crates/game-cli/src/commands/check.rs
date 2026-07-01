use std::env;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, anyhow, bail};

use crate::assets::validate_assets_dir;
use crate::commands::doctor::{DoctorOptions, doctor};
use crate::paths::{absolutize_from, configured_asset_root};
use crate::process::beginner_failure_advice;

pub(crate) struct CheckOptions {
    features: Vec<String>,
}

pub(crate) fn parse_check_options(mut args: impl Iterator<Item = String>) -> Result<CheckOptions> {
    let mut features = Vec::new();
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--features" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow!("--features needs a comma-separated feature list"))?;
                features.push(value);
            }
            extra => bail!("unexpected check argument '{extra}'; expected --features <list>"),
        }
    }
    Ok(CheckOptions { features })
}

pub(crate) fn check_project(options: &CheckOptions) -> Result<()> {
    let project = env::current_dir().context("failed to resolve current project directory")?;
    check_project_at(&project, options)
}

fn check_project_at(project: &Path, options: &CheckOptions) -> Result<()> {
    let asset_root = configured_asset_root();
    let assets = absolutize_from(project, &asset_root);

    println!("checking project setup...");
    println!("doctor output is advisory here; hard failures are assets, data, and cargo check.");
    doctor(DoctorOptions { explain: false });

    println!("\nchecking hard project gates...");
    println!("checking assets...");
    validate_assets_dir(&assets, false)?;

    let data_file = assets.join("game.ron");
    if data_file.is_file() {
        println!("checking data file...");
        game_kit::data::validate_beginner_game_file(&data_file)?;
    }

    println!("running cargo check...");
    let mut command = Command::new("cargo");
    command.arg("check").current_dir(project);
    for feature in &options.features {
        command.arg("--features").arg(feature);
    }
    let status = command
        .status()
        .context("could not run `cargo check`; is Rust installed and available on PATH?")?;
    if !status.success() {
        bail!("cargo check failed.\n\n{}", beginner_failure_advice());
    }

    println!("project check passed");
    Ok(())
}

mod assets;
mod commands;
mod manifest;
mod paths;
mod process;
mod templates;

use std::path::Path;

use anyhow::{Result, anyhow, bail};

use commands::check::{check_project, parse_check_options};
use commands::doctor::{doctor, parse_doctor_options};
use commands::package::{package_project_command, package_workspace_demo_command};
use commands::release_check::release_check_command;
use paths::{
    absolutize_from_current, configured_asset_root, game_path_from_destination,
    normalize_validate_data_path, workspace_root, xtask_demo_destination,
};
use templates::{RELEASE_GAME_STARTER_DEPENDENCY, new_project, parse_template_args};

pub use templates::DemoTemplate;

pub fn run(args: impl IntoIterator<Item = String>) -> Result<()> {
    let mut args = args.into_iter();
    match args.next().as_deref() {
        Some("new") => {
            let path = args.next().ok_or_else(|| {
                anyhow!("usage: game-dev new <path> [--template simple|data-driven]")
            })?;
            let template = parse_template_args(args)?;
            let destination = absolutize_from_current(Path::new(&path))?;
            new_project(&destination, template, RELEASE_GAME_STARTER_DEPENDENCY)
        }
        Some("doctor") => {
            let options = parse_doctor_options(args)?;
            doctor(options);
            Ok(())
        }
        Some("run") => {
            reject_extra(args, "run")?;
            process::run_project()
        }
        Some("check") => {
            let options = parse_check_options(args)?;
            check_project(&options)
        }
        Some("package") => package_project_command(args),
        Some("asset-check") => {
            reject_extra(args, "asset-check")?;
            assets::validate_assets_dir(&std::env::current_dir()?.join("assets"), false)?;
            println!("assets look valid");
            Ok(())
        }
        Some("validate-data") => {
            let path = args.next().unwrap_or_else(|| "game.ron".to_string());
            reject_extra(args, "validate-data")?;
            let asset_root = configured_asset_root();
            game_kit::data::validate_beginner_game_file(normalize_validate_data_path(
                &path,
                &asset_root,
            ))?;
            println!("beginner data file is valid");
            Ok(())
        }
        _ => bail!(
            "usage:\n    game-dev new <path> [--template simple|data-driven]\n    game-dev doctor\n    game-dev check [--features feature-list]\n    game-dev run\n    game-dev package --release --out <directory> [--features feature-list] [--zip]\n    game-dev asset-check\n    game-dev validate-data [game.ron]"
        ),
    }
}

pub fn run_xtask(args: impl IntoIterator<Item = String>) -> Result<()> {
    let workspace = workspace_root()?;
    let mut args = args.into_iter();
    match args.next().as_deref() {
        Some("new-demo") => {
            let name = args.next().ok_or_else(|| {
                anyhow!(
                    "usage: cargo xtask new-demo <name-or-path> [--template simple|data-driven]"
                )
            })?;
            let template = parse_template_args(args)?;
            let destination = xtask_demo_destination(&workspace, &name)?;
            let game_path = game_path_from_destination(&workspace, &destination)?;
            let dependency = format!(r#"{{ path = "{game_path}/crates/game-starter" }}"#);
            new_project(&destination, template, &dependency)
        }
        Some("doctor") => {
            let options = parse_doctor_options(args)?;
            doctor(options);
            Ok(())
        }
        Some("release-check") => release_check_command(args, &workspace),
        Some("package-demo") => package_workspace_demo_command(args, &workspace),
        _ => bail!(
            "usage:\n    cargo xtask new-demo <name-or-path> [--template simple|data-driven]\n    cargo xtask new-demo <name-or-path> --data-driven\n    cargo xtask release-check [--skip-smoke] [--skip-generated] [--features feature-list]\n    cargo xtask package-demo --release --out <directory> [--features feature-list]\n    cargo xtask doctor\n\nCreates an outside-workspace beginner demo, runs release-candidate checks, packages the bundled playable demo, or checks local graphics prerequisites."
        ),
    }
}

fn reject_extra(mut args: impl Iterator<Item = String>, command: &str) -> Result<()> {
    if let Some(extra) = args.next() {
        bail!("unexpected argument for {command}: '{extra}'");
    }
    Ok(())
}

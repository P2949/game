mod assets;
mod commands;
mod manifest;
mod paths;
mod process;
pub mod project;
mod starter_assets;
mod templates;

use std::path::Path;

use anyhow::{Result, anyhow, bail};

use commands::asset_check::asset_check_command;
use commands::authoring_scan::authoring_scan_command;
use commands::check::{check_project, parse_check_options};
use commands::doctor::{doctor, parse_doctor_options};
use commands::migrate_ron::migrate_ron_command;
use commands::package::{package_project_command, package_workspace_demo_command};
use commands::package_sdk::package_sdk_command;
use commands::preview::preview_command;
use commands::release_check::release_check_command;
use commands::validate_data::validate_data_command;
use paths::{
    absolutize_from_current, game_path_from_destination, workspace_root, xtask_demo_destination,
};
use templates::{RELEASE_GAME_STARTER_DEPENDENCY, new_project, parse_template_args};

pub use templates::DemoTemplate;

pub fn run(args: impl IntoIterator<Item = String>) -> Result<()> {
    let mut args = args.into_iter();
    match args.next().as_deref() {
        Some("new") => {
            let path = args.next().ok_or_else(|| {
                anyhow!(
                    "usage: game-dev new <path> [--template no-rust|simple|rust-simple|data-driven|data-driven-legacy]"
                )
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
        Some("preview") => preview_command(args),
        Some("authoring-scan") => authoring_scan_command(args),
        Some("migrate-ron") => migrate_ron_command(args),
        Some("asset-check") => asset_check_command(args),
        Some("validate-data") => validate_data_command(args),
        _ => bail!(
            "usage:\n    game-dev new <path> [--template no-rust|simple|rust-simple|data-driven|data-driven-legacy]\n    game-dev doctor\n    game-dev check [--project dir] [--features feature-list]\n    game-dev preview [--watch]\n    game-dev authoring-scan [--project dir]\n    game-dev migrate-ron assets/game.ron --out game.toml\n    game-dev run\n    game-dev package [--release] --out <directory> [--features feature-list] [--zip]\n    game-dev asset-check [--project dir] [--assets dir]\n    game-dev validate-data [game.toml]"
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
                    "usage: cargo xtask new-demo <name-or-path> [--template no-rust|simple|rust-simple|data-driven|data-driven-legacy]"
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
        Some("package-sdk") => package_sdk_command(args, &workspace),
        _ => bail!(
            "usage:\n    cargo xtask new-demo <name-or-path> [--template no-rust|simple|rust-simple|data-driven|data-driven-legacy]\n    cargo xtask new-demo <name-or-path> --data-driven\n    cargo xtask release-check [--skip-smoke] [--skip-generated] [--features feature-list]\n    cargo xtask package-demo --release --out <directory> [--features feature-list]\n    cargo xtask package-sdk --release --out <directory> [--features feature-list]\n    cargo xtask doctor\n\nCreates an outside-workspace beginner demo, runs release-candidate checks, packages the bundled playable demo, packages the no-Rust SDK, or checks local graphics prerequisites."
        ),
    }
}

fn reject_extra(mut args: impl Iterator<Item = String>, command: &str) -> Result<()> {
    if let Some(extra) = args.next() {
        bail!("unexpected argument for {command}: '{extra}'");
    }
    Ok(())
}

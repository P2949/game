use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, anyhow, bail};

use crate::assets::{
    asset_ignore_patterns_from_game_file, validate_assets_dir, validate_assets_dir_with_ignores,
};
use crate::commands::doctor::{DoctorOptions, doctor};
use crate::paths::{absolutize_from, configured_asset_root};
use crate::process::beginner_failure_advice;
use crate::project::{
    NoRustPathOverrides, ProjectKind, detect_project_kind, resolve_no_rust_project_paths_with_env,
};

pub(crate) struct CheckOptions {
    features: Vec<String>,
    project: Option<PathBuf>,
}

pub(crate) fn parse_check_options(mut args: impl Iterator<Item = String>) -> Result<CheckOptions> {
    let mut features = Vec::new();
    let mut project = None;
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--features" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow!("--features needs a comma-separated feature list"))?;
                features.push(value);
            }
            "--project" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow!("--project needs a project directory"))?;
                project = Some(PathBuf::from(value));
            }
            extra => bail!(
                "unexpected check argument '{extra}'; expected --project <dir> or --features <list>"
            ),
        }
    }
    Ok(CheckOptions { features, project })
}

pub(crate) fn check_project(options: &CheckOptions) -> Result<()> {
    let current = env::current_dir().context("failed to resolve current project directory")?;
    let project = options
        .project
        .as_deref()
        .map(|project| absolutize_from(&current, project))
        .unwrap_or(current);
    check_project_at(&project, options)
}

fn check_project_at(project: &Path, options: &CheckOptions) -> Result<()> {
    check_project_at_with_cargo(project, options, Path::new("cargo"))
}

fn check_project_at_with_cargo(
    project: &Path,
    options: &CheckOptions,
    cargo_executable: &Path,
) -> Result<()> {
    let kind = detect_project_kind(project)?;
    if kind == ProjectKind::NoRustPackage {
        return check_no_rust_project_at(project);
    }

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
    let mut command = Command::new(cargo_executable);
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

fn check_no_rust_project_at(project: &Path) -> Result<()> {
    let paths = resolve_no_rust_project_paths_with_env(project, &NoRustPathOverrides::default());

    println!("checking no-Rust project setup...");
    println!("doctor output is advisory here; hard failures are assets and game.toml.");
    doctor(DoctorOptions { explain: false });

    println!("\nchecking hard project gates...");
    println!("checking assets...");
    let ignore = asset_ignore_patterns_from_game_file(&paths.game_file)?;
    validate_assets_dir_with_ignores(&paths.asset_dir, false, ignore)?;

    println!("checking game config...");
    game_kit::data::validate_authoring_file_with_asset_root(&paths.game_file, &paths.asset_dir)?;

    println!("no-Rust project check passed");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{CheckOptions, check_project_at_with_cargo};
    use crate::templates::{DemoTemplate, new_project};
    use std::fs;
    use std::path::{Path, PathBuf};

    #[test]
    fn no_rust_check_validates_game_toml_without_cargo_manifest() {
        let project = temp_project("no-rust-check");
        new_project(&project, DemoTemplate::NoRust, "{ path = \"unused\" }").unwrap();

        check_project_at_with_cargo(
            &project,
            &CheckOptions {
                features: Vec::new(),
                project: None,
            },
            &project.join("missing-cargo"),
        )
        .unwrap();
        assert!(!project.join("Cargo.toml").exists());
    }

    #[test]
    fn rust_check_invokes_cargo_for_rust_template() {
        let project = temp_project("rust-check");
        let tools = temp_project("rust-check-tools");
        fs::create_dir_all(&tools).unwrap();
        let marker = tools.join("cargo-called");
        let cargo = fake_cargo(&tools, &marker);
        new_project(&project, DemoTemplate::Simple, "{ path = \"unused\" }").unwrap();

        check_project_at_with_cargo(
            &project,
            &CheckOptions {
                features: Vec::new(),
                project: None,
            },
            &cargo,
        )
        .unwrap();

        let invocation = fs::read_to_string(marker).unwrap();
        assert!(invocation.contains("check"));
    }

    #[cfg(unix)]
    fn fake_cargo(directory: &Path, marker: &Path) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;

        let cargo = directory.join("cargo");
        fs::write(
            &cargo,
            format!(
                "#!/usr/bin/env sh\nprintf '%s\\n' \"$*\" > \"{}\"\nexit 0\n",
                marker.display()
            ),
        )
        .unwrap();
        let mut permissions = fs::metadata(&cargo).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&cargo, permissions).unwrap();
        cargo
    }

    #[cfg(windows)]
    fn fake_cargo(directory: &Path, marker: &Path) -> PathBuf {
        let cargo = directory.join("cargo.bat");
        fs::write(
            &cargo,
            format!(
                "@echo off\r\necho %* > \"{}\"\r\nexit /b 0\r\n",
                marker.display()
            ),
        )
        .unwrap();
        cargo
    }

    fn temp_project(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "game-cli-{name}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        if dir.exists() {
            fs::remove_dir_all(&dir).unwrap();
        }
        dir
    }
}

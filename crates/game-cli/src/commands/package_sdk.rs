use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, anyhow, bail};

use crate::assets::{asset_ignore_patterns_from_game_file, validate_assets_dir_with_ignores};
use crate::paths::{absolutize_from, executable_name};
use crate::starter_assets;

use super::package::{
    PackageOptions, copy_directory, copy_runtime_libraries, ensure_empty_or_missing,
    parse_package_options, write_launchers,
};

pub(crate) fn package_sdk_command(
    args: impl Iterator<Item = String>,
    workspace: &Path,
) -> Result<()> {
    let PackageOptions {
        release,
        output,
        zip,
        features,
    } = parse_package_options(args, "package-sdk")?;
    if zip {
        bail!(
            "cargo xtask package-sdk does not support --zip; archive the output directory after verification"
        );
    }
    if !release {
        bail!("package-sdk currently requires --release");
    }
    let output = output.ok_or_else(|| anyhow!("package-sdk requires --out <directory>"))?;
    package_sdk(workspace, &output, &features)
}

fn package_sdk(workspace: &Path, requested_output: &Path, features: &[String]) -> Result<()> {
    let output = absolutize_from(workspace, requested_output);
    ensure_empty_or_missing(&output)?;

    let mut build = Command::new("cargo");
    build.args([
        "build",
        "-p",
        "game-player",
        "-p",
        "game-cli",
        "--release",
        "--locked",
    ]);
    for feature in features {
        build.arg("--features").arg(feature);
    }
    let status = build
        .current_dir(workspace)
        .status()
        .context("could not run cargo build for package-sdk")?;
    if !status.success() {
        bail!("release build failed; no SDK package was created");
    }

    let target = env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| workspace.join("target"));
    let build_dir = target.join("release");
    let player_name = executable_name("game-player");
    let game_dev_name = executable_name("game-dev");
    let player = build_dir.join(&player_name);
    let game_dev = build_dir.join(&game_dev_name);
    if !player.is_file() {
        bail!(
            "release build completed but '{}' was not produced",
            player.display()
        );
    }
    if !game_dev.is_file() {
        bail!(
            "release build completed but '{}' was not produced",
            game_dev.display()
        );
    }

    fs::create_dir_all(&output)
        .with_context(|| format!("failed to create SDK output '{}'", output.display()))?;
    fs::copy(&player, output.join(&player_name)).with_context(|| {
        format!(
            "failed to copy game-player '{}' to '{}'",
            player.display(),
            output.display()
        )
    })?;
    fs::copy(&game_dev, output.join(&game_dev_name)).with_context(|| {
        format!(
            "failed to copy game-dev '{}' to '{}'",
            game_dev.display(),
            output.display()
        )
    })?;
    copy_runtime_libraries(&build_dir, &output)?;

    let template_source = workspace.join("templates/no-rust-demo");
    let template_output = output.join("templates/no-rust-demo");
    copy_directory(&template_source, &template_output)?;
    starter_assets::write_builtin_font(&template_output.join("assets"))?;
    validate_sdk_no_rust_template(&template_output)?;

    for example in ["no-rust-minimal", "no-rust-full"] {
        let example_output = output.join("examples").join(example);
        copy_directory(&workspace.join("examples").join(example), &example_output)?;
        starter_assets::write_builtin_font(&example_output.join("assets"))?;
    }

    copy_required_file(&workspace.join("LICENSE"), &output.join("LICENSE"))?;
    copy_required_file(
        &workspace.join("THIRD_PARTY_NOTICES.md"),
        &output.join("THIRD_PARTY_NOTICES.md"),
    )?;
    write_launchers(&output, &player_name)?;
    write_sdk_readme(&output, &player_name, &game_dev_name)?;

    println!("packaged no-Rust SDK at {}", output.display());
    Ok(())
}

fn copy_required_file(source: &Path, destination: &Path) -> Result<()> {
    if !source.is_file() {
        bail!(
            "required package file '{}' does not exist",
            source.display()
        );
    }
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create '{}'", parent.display()))?;
    }
    fs::copy(source, destination).with_context(|| {
        format!(
            "failed to copy '{}' to '{}'",
            source.display(),
            destination.display()
        )
    })?;
    Ok(())
}

fn validate_sdk_no_rust_template(template: &Path) -> Result<()> {
    for forbidden in ["Cargo.toml", "Cargo.lock", "build.rs", "src/main.rs"] {
        let path = template.join(forbidden);
        if path.exists() {
            bail!(
                "SDK no-Rust template must not contain Rust project file '{}'",
                path.display()
            );
        }
    }
    if !template.join("game.toml").is_file() {
        bail!(
            "SDK no-Rust template '{}' is missing game.toml",
            template.display()
        );
    }
    let assets = template.join("assets");
    let ignore = asset_ignore_patterns_from_game_file(&template.join("game.toml"))?;
    validate_assets_dir_with_ignores(&assets, false, ignore)?;
    game_kit::data::validate_authoring_file_with_asset_root(template.join("game.toml"), &assets)?;
    Ok(())
}

fn write_sdk_readme(output: &Path, player_name: &str, game_dev_name: &str) -> Result<()> {
    let readme = output.join("README.txt");
    fs::write(
        &readme,
        format!(
            "No-Rust game SDK\n\nNo Rust or Cargo is required to create, check, preview, edit, or package a no-Rust game with this SDK.\n\nStart a new game:\n\n  ./{game_dev_name} new my-game --template no-rust\n  cd my-game\n  ../{game_dev_name} check\n  ../{game_dev_name} preview\n\nTry the bundled template directly:\n\n  ./run.sh --project templates/no-rust-demo\n\nEdit game.toml and assets/ in any text editor. The prebuilt player is {player_name}; the helper CLI is {game_dev_name}. Keep LICENSE and THIRD_PARTY_NOTICES.md with redistributed SDK copies.\n"
        ),
    )
    .with_context(|| format!("failed to write '{}'", readme.display()))
}

#[cfg(test)]
mod tests {
    use super::validate_sdk_no_rust_template;
    use crate::templates::{DemoTemplate, new_project};
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn sdk_no_rust_template_validation_rejects_rust_project_files() {
        let template = temp_path("sdk-template");
        new_project(&template, DemoTemplate::NoRust, "{ path = \"unused\" }").unwrap();
        validate_sdk_no_rust_template(&template).unwrap();

        fs::write(template.join("Cargo.toml"), "[package]\nname = \"bad\"\n").unwrap();
        let error = validate_sdk_no_rust_template(&template)
            .unwrap_err()
            .to_string();
        assert!(error.contains("must not contain Rust project file"));

        fs::remove_dir_all(template).unwrap();
    }

    fn temp_path(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "game-cli-{name}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        if path.exists() {
            if path.is_dir() {
                fs::remove_dir_all(&path).unwrap();
            } else {
                fs::remove_file(&path).unwrap();
            }
        }
        path
    }
}

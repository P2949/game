use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, anyhow, bail};
use walkdir::WalkDir;

use crate::assets::{
    asset_ignore_patterns_from_game_file, validate_assets_dir, validate_assets_dir_with_ignores,
};
use crate::manifest::package_info_from_manifest;
use crate::paths::{absolutize_from, executable_name, source_assets_dir};
use crate::process::beginner_failure_advice;
use crate::project::{
    NoRustPathOverrides, ProjectKind, detect_project_kind, resolve_no_rust_project_paths_with_env,
};

pub(super) struct PackageOptions {
    pub(super) release: bool,
    pub(super) output: Option<PathBuf>,
    pub(super) zip: bool,
    pub(super) features: Vec<String>,
}

pub(crate) fn package_project_command(args: impl Iterator<Item = String>) -> Result<()> {
    let PackageOptions {
        release,
        output,
        zip,
        features,
    } = parse_package_options(args, "package")?;
    let output = output.ok_or_else(|| anyhow!("game-dev package requires --out <directory>"))?;
    package_current_project(&output, zip, release, &features)
}

pub(crate) fn package_workspace_demo_command(
    args: impl Iterator<Item = String>,
    workspace: &Path,
) -> Result<()> {
    let PackageOptions {
        release,
        output,
        zip,
        features,
    } = parse_package_options(args, "package-demo")?;
    if zip {
        bail!(
            "cargo xtask package-demo does not support --zip; use game-dev package for project zips"
        );
    }
    if !release {
        bail!("package-demo currently requires --release");
    }
    let output = output.ok_or_else(|| anyhow!("package-demo requires --out <directory>"))?;
    package_workspace_demo(workspace, &output, &features)
}

pub(super) fn parse_package_options(
    mut args: impl Iterator<Item = String>,
    command: &str,
) -> Result<PackageOptions> {
    let mut release = false;
    let mut output = None;
    let mut zip = false;
    let mut features = Vec::new();
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--release" => release = true,
            "--zip" => zip = true,
            "--features" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow!("--features needs a comma-separated feature list"))?;
                features.push(value);
            }
            "--out" => {
                let path = args
                    .next()
                    .ok_or_else(|| anyhow!("--out needs a destination directory"))?;
                output = Some(PathBuf::from(path));
            }
            other => bail!("unknown {command} argument '{other}'"),
        }
    }
    Ok(PackageOptions {
        release,
        output,
        zip,
        features,
    })
}

fn package_current_project(
    requested_output: &Path,
    zip: bool,
    release: bool,
    features: &[String],
) -> Result<()> {
    let project = env::current_dir().context("failed to resolve current project directory")?;
    package_project_at(&project, requested_output, zip, release, features)
}

pub(crate) fn package_project_at(
    project: &Path,
    requested_output: &Path,
    zip: bool,
    release: bool,
    features: &[String],
) -> Result<()> {
    if detect_project_kind(project)? == ProjectKind::NoRustPackage {
        return package_no_rust_project_at(project, requested_output, zip, None);
    }
    if !release {
        bail!("game-dev package currently requires --release for Rust projects");
    }

    let output = absolutize_from(project, requested_output);
    ensure_empty_or_missing(&output)?;

    let package_info = package_info_from_manifest(&project.join("Cargo.toml"))?;
    let assets = absolutize_from(project, &package_info.asset_dir);
    if !assets.is_dir() {
        bail!("assets directory '{}' does not exist", assets.display());
    }
    validate_assets_dir(&assets, false)?;

    let mut build = Command::new("cargo");
    build.args(["build", "--release"]).current_dir(project);
    for feature in features {
        build.arg("--features").arg(feature);
    }
    let status = build
        .status()
        .context("could not run release build for generated project")?;
    if !status.success() {
        bail!(
            "release build failed; no package was created.\n\n{}",
            beginner_failure_advice()
        );
    }

    let executable_name = executable_name(&package_info.package_name);
    let target = env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| project.join("target"));
    let executable = target.join("release").join(&executable_name);
    if !executable.is_file() {
        bail!(
            "release build completed but '{}' was not produced",
            executable.display()
        );
    }

    fs::create_dir_all(&output)
        .with_context(|| format!("failed to create package output '{}'", output.display()))?;
    fs::copy(&executable, output.join(&executable_name)).with_context(|| {
        format!(
            "failed to copy packaged executable '{}' to '{}'",
            executable.display(),
            output.display()
        )
    })?;
    copy_runtime_libraries(&target.join("release"), &output)?;
    copy_directory(&assets, &output.join("assets"))?;
    ensure_builtin_font(&output.join("assets"))?;
    validate_assets_dir(&output.join("assets"), true)?;
    write_launchers(&output, &executable_name)?;
    write_project_package_readme(&output, &executable_name)?;
    if zip {
        zip_package(&output)?;
    }

    println!("packaged project at {}", output.display());
    Ok(())
}

fn package_no_rust_project_at(
    project: &Path,
    requested_output: &Path,
    zip: bool,
    player_override: Option<&Path>,
) -> Result<()> {
    let output = absolutize_from(project, requested_output);
    ensure_empty_or_missing(&output)?;
    let paths = resolve_no_rust_project_paths_with_env(project, &NoRustPathOverrides::default());

    let ignore = asset_ignore_patterns_from_game_file(&paths.game_file)?;
    validate_assets_dir_with_ignores(&paths.asset_dir, false, ignore.clone())?;
    game_kit::data::validate_authoring_file_with_asset_root(&paths.game_file, &paths.asset_dir)?;

    let player = resolve_player_executable(player_override)?;
    let player_name = executable_name("game-player");

    fs::create_dir_all(&output)
        .with_context(|| format!("failed to create package output '{}'", output.display()))?;
    fs::copy(&player, output.join(&player_name)).with_context(|| {
        format!(
            "failed to copy game-player '{}' to '{}'",
            player.display(),
            output.display()
        )
    })?;
    if let Ok(game_dev) = env::current_exe()
        && game_dev.is_file()
    {
        let game_dev_name = game_dev
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("game-dev");
        let _ = fs::copy(&game_dev, output.join(game_dev_name));
    }
    fs::copy(&paths.game_file, output.join("game.toml")).with_context(|| {
        format!(
            "failed to copy game config '{}' to package",
            paths.game_file.display()
        )
    })?;
    copy_directory(&paths.asset_dir, &output.join("assets"))?;
    ensure_builtin_font(&output.join("assets"))?;
    validate_assets_dir_with_ignores(&output.join("assets"), true, ignore)?;
    write_launchers(&output, &player_name)?;
    write_no_rust_package_readme(&output, &player_name)?;
    if zip {
        zip_package(&output)?;
    }

    println!("packaged no-Rust project at {}", output.display());
    Ok(())
}

fn resolve_player_executable(explicit: Option<&Path>) -> Result<PathBuf> {
    if let Some(path) = explicit {
        if path.is_file() {
            return Ok(path.to_path_buf());
        }
        bail!("game-player executable '{}' does not exist", path.display());
    }
    if let Some(path) = env::var_os("GAME_PLAYER").map(PathBuf::from)
        && path.is_file()
    {
        return Ok(path);
    }
    if let Ok(current_exe) = env::current_exe()
        && let Some(parent) = current_exe.parent()
    {
        let sibling = parent.join(executable_name("game-player"));
        if sibling.is_file() {
            return Ok(sibling);
        }
    }
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for profile in ["release", "debug"] {
        let candidate = workspace
            .join("target")
            .join(profile)
            .join(executable_name("game-player"));
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    bail!("could not find game-player; build it first or set GAME_PLAYER")
}

fn package_workspace_demo(
    workspace: &Path,
    requested_output: &Path,
    features: &[String],
) -> Result<()> {
    let output = absolutize_from(workspace, requested_output);
    ensure_empty_or_missing(&output)?;

    let assets = workspace.join("assets");
    validate_assets_dir(&assets, true)?;

    let mut build = Command::new("cargo");
    build.args(["build", "-p", "game", "--release", "--locked"]);
    for feature in features {
        build.arg("--features").arg(feature);
    }
    let status = build
        .current_dir(workspace)
        .status()
        .context("could not run cargo build for package-demo")?;
    if !status.success() {
        bail!("release build failed; shaders are not confirmed and no package was created");
    }

    let target = env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| workspace.join("target"));
    let executable_name = executable_name("game");
    let executable = target.join("release").join(&executable_name);
    if !executable.is_file() {
        bail!(
            "release build completed but '{}' was not produced",
            executable.display()
        );
    }

    fs::create_dir_all(&output)
        .with_context(|| format!("failed to create package output '{}'", output.display()))?;
    fs::copy(&executable, output.join(&executable_name)).with_context(|| {
        format!(
            "failed to copy packaged executable '{}'",
            executable.display()
        )
    })?;
    copy_runtime_libraries(&target.join("release"), &output)?;
    copy_directory(&assets, &output.join("assets"))?;
    write_launchers(&output, &executable_name)?;
    write_workspace_package_readme(&output, &executable_name)?;

    println!("packaged release demo at {}", output.display());
    println!("send the entire directory, including assets/, to a player");
    Ok(())
}

pub(super) fn ensure_empty_or_missing(output: &Path) -> Result<()> {
    if output.exists()
        && fs::read_dir(output)
            .with_context(|| format!("failed to read package destination '{}'", output.display()))?
            .next()
            .is_some()
    {
        bail!(
            "package destination '{}' already exists and is not empty; choose a new --out directory",
            output.display()
        );
    }
    Ok(())
}

pub(super) fn copy_directory(source: &Path, destination: &Path) -> Result<()> {
    for entry in WalkDir::new(source) {
        let entry = entry.with_context(|| format!("could not walk '{}'", source.display()))?;
        let relative = entry
            .path()
            .strip_prefix(source)
            .expect("walk entry is under its source directory");
        let target = destination.join(relative);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target)
                .with_context(|| format!("failed to create '{}'", target.display()))?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("failed to create '{}'", parent.display()))?;
            }
            fs::copy(entry.path(), &target).with_context(|| {
                format!(
                    "failed to copy asset '{}' to '{}'",
                    entry.path().display(),
                    target.display()
                )
            })?;
        }
    }
    Ok(())
}

pub(super) fn copy_runtime_libraries(build_dir: &Path, output: &Path) -> Result<()> {
    for name in [
        "libSDL3.so.0",
        "libSDL3.0.dylib",
        "libSDL3.dylib",
        "SDL3.dll",
    ] {
        let source = build_dir.join(name);
        if source.is_file() {
            fs::copy(&source, output.join(name)).with_context(|| {
                format!(
                    "failed to copy runtime library '{}' to '{}'",
                    source.display(),
                    output.display()
                )
            })?;
        }
    }
    Ok(())
}

fn ensure_builtin_font(assets: &Path) -> Result<()> {
    let target = assets.join("fonts/DejaVuSans.ttf");
    if target.is_file() {
        return Ok(());
    }
    let source = source_assets_dir().join("fonts/DejaVuSans.ttf");
    if !source.is_file() {
        bail!(
            "release packages need assets/fonts/DejaVuSans.ttf, but '{}' was not found; add that font to your project assets",
            source.display()
        );
    }
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create '{}'", parent.display()))?;
    }
    fs::copy(&source, &target).with_context(|| {
        format!(
            "failed to copy bundled font '{}' to '{}'",
            source.display(),
            target.display()
        )
    })?;
    Ok(())
}

pub(super) fn write_launchers(output: &Path, executable_name: &str) -> Result<()> {
    let shell = output.join("run.sh");
    fs::write(
        &shell,
        format!(
            "#!/usr/bin/env sh\ncd \"$(dirname \"$0\")\"\npackage_dir=$(pwd)\nif [ -n \"${{LD_LIBRARY_PATH:-}}\" ]; then\n  export LD_LIBRARY_PATH=\"$package_dir:$LD_LIBRARY_PATH\"\nelse\n  export LD_LIBRARY_PATH=\"$package_dir\"\nfi\nif [ -n \"${{DYLD_LIBRARY_PATH:-}}\" ]; then\n  export DYLD_LIBRARY_PATH=\"$package_dir:$DYLD_LIBRARY_PATH\"\nelse\n  export DYLD_LIBRARY_PATH=\"$package_dir\"\nfi\nexec ./{executable_name} \"$@\"\n"
        ),
    )
    .with_context(|| format!("failed to write '{}'", shell.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&shell, fs::Permissions::from_mode(0o755))
            .with_context(|| format!("failed to mark '{}' executable", shell.display()))?;
    }

    let powershell = output.join("run.ps1");
    fs::write(
        &powershell,
        format!(
            "Set-Location -LiteralPath $PSScriptRoot\r\n& .\\{executable_name} @args\r\nexit $LASTEXITCODE\r\n"
        ),
    )
    .with_context(|| format!("failed to write '{}'", powershell.display()))?;

    let batch = output.join("run.bat");
    fs::write(
        &batch,
        "@echo off\r\ncd /d \"%~dp0\"\r\npowershell -ExecutionPolicy Bypass -File .\\run.ps1 %*\r\n",
    )
    .with_context(|| format!("failed to write '{}'", batch.display()))?;
    Ok(())
}

fn write_project_package_readme(output: &Path, executable_name: &str) -> Result<()> {
    let readme = output.join("README.txt");
    fs::write(
        &readme,
        format!(
            "Playable game package\n\nKeep this directory together: `{executable_name}` needs the adjacent `assets` folder. If runtime library files such as SDL3 are included, keep them beside the executable too.\n\nLinux/macOS: run ./run.sh from a terminal.\nWindows: right-click run.ps1 and choose Run with PowerShell, or double-click run.bat.\n\nRuntime requirements\n\nThis build requires a Vulkan-capable GPU and driver. If it fails to start, install or update your Vulkan runtime/driver.\n\nLinux: install the Vulkan loader/tools package and your GPU vendor driver. Mesa/lavapipe can run smoke tests but is not ideal for players.\nWindows: update your graphics driver; the Vulkan Runtime is usually included with current NVIDIA, AMD, and Intel drivers.\nmacOS: run through MoltenVK/Vulkan SDK support; this command does not create a .app bundle.\n"
        ),
    )
    .with_context(|| format!("failed to write '{}'", readme.display()))
}

fn write_no_rust_package_readme(output: &Path, executable_name: &str) -> Result<()> {
    let readme = output.join("README.txt");
    fs::write(
        &readme,
        format!(
            "No-Rust game package\n\nOpen game.toml in any text editor to change the game. Keep this directory together: `{executable_name}` needs the adjacent `game.toml` and `assets` folder.\n\nLinux/macOS: run ./run.sh from a terminal.\nWindows: right-click run.ps1 and choose Run with PowerShell, or double-click run.bat.\n\nNo Rust or Cargo is needed to play or edit this package.\n"
        ),
    )
    .with_context(|| format!("failed to write '{}'", readme.display()))
}

fn write_workspace_package_readme(output: &Path, executable_name: &str) -> Result<()> {
    let readme = output.join("README.txt");
    fs::write(
        &readme,
        format!(
            "Playable game package\n\nKeep this directory together: `{executable_name}` needs the adjacent `assets` folder. If runtime library files such as SDL3 are included, keep them beside the executable too.\n\nLinux: run ./run.sh from a terminal.\nWindows: right-click run.ps1 and choose Run with PowerShell, or double-click run.bat.\nmacOS: open Terminal in this folder and run ./run.sh; an app bundle is not created by this command.\n\nRuntime requirements\n\nThis build requires a Vulkan-capable GPU and driver. If it fails to start, install or update your Vulkan runtime/driver.\n\nLinux: install the Vulkan loader/tools package and your GPU vendor driver. Mesa/lavapipe can run smoke tests but is not ideal for players.\nWindows: update your graphics driver; the Vulkan Runtime is usually included with current NVIDIA, AMD, and Intel drivers.\nmacOS: run through MoltenVK/Vulkan SDK support; this command does not create a .app bundle.\n\nThe bundled binary defaults to the Arena demo. Set GAME_DEMO=simple or GAME_DEMO=testbed before launching to select those bundled demos.\n"
        ),
    )
    .with_context(|| format!("failed to write '{}'", readme.display()))
}

fn zip_package(output: &Path) -> Result<()> {
    let zip_path = output.with_extension("zip");
    if zip_path.exists() {
        bail!(
            "zip destination '{}' already exists; remove it or choose another --out path",
            zip_path.display()
        );
    }
    let status = Command::new("zip")
        .args(["-r"])
        .arg(&zip_path)
        .arg(".")
        .current_dir(output)
        .status()
        .context("could not run `zip`; install zip or omit --zip")?;
    if !status.success() {
        bail!("zip command failed while packaging '{}'", output.display());
    }
    println!("wrote {}", zip_path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::package_no_rust_project_at;
    use crate::templates::{DemoTemplate, new_project};
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn no_rust_package_copies_player_config_and_assets_without_cargo() {
        let project = temp_path("package-project");
        let output = temp_path("package-output");
        let player = temp_path("fake-player");
        new_project(&project, DemoTemplate::NoRust, "{ path = \"unused\" }").unwrap();
        fs::write(&player, "fake player").unwrap();

        package_no_rust_project_at(&project, &output, false, Some(&player)).unwrap();

        assert!(output.join("game.toml").is_file());
        assert!(output.join("assets/maps/level-1.txt").is_file());
        assert!(output.join("README.txt").is_file());
        assert!(output.join(super::executable_name("game-player")).is_file());
        assert!(!output.join("Cargo.toml").exists());
        assert!(!output.join("src/main.rs").exists());
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

use std::collections::HashMap;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, anyhow, bail};
use fontdue::{Font, FontSettings};
use image::ImageReader;
use walkdir::WalkDir;

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("new-demo") => {
            let name = args.next().ok_or_else(|| {
                anyhow!(
                    "usage: cargo xtask new-demo <name-or-path> [--template simple|data-driven]"
                )
            })?;
            let mut template = DemoTemplate::Simple;
            while let Some(argument) = args.next() {
                match argument.as_str() {
                    "--data-driven" => template = DemoTemplate::DataDriven,
                    "--template" => {
                        let value = args
                            .next()
                            .ok_or_else(|| anyhow!("--template needs simple or data-driven"))?;
                        template = DemoTemplate::parse(&value)?;
                    }
                    extra => {
                        bail!(
                            "unexpected new-demo argument '{extra}'; expected --template simple|data-driven"
                        )
                    }
                }
            }
            new_demo(&name, template)
        }
        Some("doctor") => {
            if let Some(extra) = args.next() {
                bail!("unexpected argument for doctor: '{extra}'");
            }
            doctor();
            Ok(())
        }
        Some("package-demo") => package_demo_command(args),
        _ => {
            bail!(
                "usage:\n    cargo xtask new-demo <name-or-path> [--template simple|data-driven]\n    cargo xtask new-demo <name-or-path> --data-driven\n    cargo xtask package-demo --release --out <directory>\n    cargo xtask doctor\n\nCreates an outside-workspace beginner demo, packages the bundled playable demo, or checks local graphics prerequisites."
            );
        }
    }
}

fn package_demo_command(mut args: impl Iterator<Item = String>) -> Result<()> {
    let mut release = false;
    let mut output = None;
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--release" => release = true,
            "--out" => {
                let path = args
                    .next()
                    .ok_or_else(|| anyhow!("--out needs a destination directory"))?;
                output = Some(PathBuf::from(path));
            }
            other => bail!("unknown package-demo argument '{other}'"),
        }
    }
    if !release {
        bail!("package-demo currently requires --release");
    }
    let output = output.ok_or_else(|| anyhow!("package-demo requires --out <directory>"))?;
    package_demo(&output)
}

fn package_demo(requested_output: &Path) -> Result<()> {
    let workspace = workspace_root()?;
    let output = if requested_output.is_absolute() {
        requested_output.to_path_buf()
    } else {
        workspace.join(requested_output)
    };
    if output.exists()
        && fs::read_dir(&output)
            .with_context(|| format!("failed to read package destination '{}'", output.display()))?
            .next()
            .is_some()
    {
        bail!(
            "package destination '{}' already exists and is not empty; choose a new --out directory",
            output.display()
        );
    }

    let assets = workspace.join("assets");
    validate_package_assets(&assets)?;

    let status = Command::new("cargo")
        .args(["build", "-p", "game", "--release", "--locked"])
        .current_dir(&workspace)
        .status()
        .context("could not run cargo build for package-demo")?;
    if !status.success() {
        bail!("release build failed; shaders are not confirmed and no package was created");
    }

    let target = env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| workspace.join("target"));
    let executable_name = if cfg!(windows) { "game.exe" } else { "game" };
    let executable = target.join("release").join(executable_name);
    if !executable.is_file() {
        bail!(
            "release build completed but '{}' was not produced",
            executable.display()
        );
    }

    fs::create_dir_all(&output)
        .with_context(|| format!("failed to create package output '{}'", output.display()))?;
    fs::copy(&executable, output.join(executable_name)).with_context(|| {
        format!(
            "failed to copy packaged executable '{}'",
            executable.display()
        )
    })?;
    copy_directory(&assets, &output.join("assets"))?;
    write_launchers(&output, executable_name)?;
    write_package_readme(&output, executable_name)?;

    println!("packaged release demo at {}", output.display());
    println!("send the entire directory, including assets/, to a player");
    Ok(())
}

fn validate_package_assets(assets: &Path) -> Result<()> {
    if !assets.is_dir() {
        bail!(
            "package assets directory '{}' does not exist",
            assets.display()
        );
    }
    for required in ["fonts/DejaVuSans.ttf", "textures/test.png"] {
        let path = assets.join(required);
        if !path.is_file() {
            bail!(
                "required packaged asset '{}' does not exist",
                path.display()
            );
        }
    }

    let mut checked = 0usize;
    for entry in WalkDir::new(assets) {
        let entry =
            entry.with_context(|| format!("could not walk assets '{}'", assets.display()))?;
        if !entry.file_type().is_file() {
            continue;
        }
        checked += 1;
        let path = entry.path();
        match path.extension().and_then(|extension| extension.to_str()) {
            Some(extension) if extension.eq_ignore_ascii_case("png") => {
                ImageReader::open(path)
                    .with_context(|| format!("could not open PNG '{}'", path.display()))?
                    .with_guessed_format()
                    .with_context(|| format!("could not identify PNG '{}'", path.display()))?
                    .decode()
                    .with_context(|| format!("could not decode PNG '{}'", path.display()))?;
            }
            Some(extension) if extension.eq_ignore_ascii_case("ttf") => {
                let bytes = fs::read(path)
                    .with_context(|| format!("could not read font '{}'", path.display()))?;
                Font::from_bytes(bytes, FontSettings::default()).map_err(|error| {
                    anyhow!("could not parse font '{}': {error}", path.display())
                })?;
            }
            Some(extension)
                if matches!(
                    extension.to_ascii_lowercase().as_str(),
                    "wav" | "ogg" | "mp3"
                ) =>
            {
                game_audio::validate_file_sound(path)
                    .with_context(|| format!("could not decode sound '{}'", path.display()))?;
            }
            Some(extension) if extension.eq_ignore_ascii_case("txt") => {
                validate_text_map(path)?;
            }
            Some(extension) if extension.eq_ignore_ascii_case("tmx") => {
                game_map::load_tiled_map_file(path)
                    .with_context(|| format!("could not validate TMX map '{}'", path.display()))?;
            }
            Some(extension) if extension.eq_ignore_ascii_case("ldtk") => {
                let text = fs::read_to_string(path)
                    .with_context(|| format!("could not read LDtk project '{}'", path.display()))?;
                serde_json::from_str::<serde_json::Value>(&text).with_context(|| {
                    format!("could not parse LDtk project '{}'", path.display())
                })?;
            }
            _ => {}
        }
    }
    if checked == 0 {
        bail!("package assets directory '{}' is empty", assets.display());
    }
    Ok(())
}

fn validate_text_map(path: &Path) -> Result<()> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("could not read text map '{}'", path.display()))?;
    let rows = text
        .lines()
        .map(|line| line.trim_end_matches('\r'))
        .collect::<Vec<_>>();
    let Some(first) = rows.first() else {
        bail!("text map '{}' has no rows", path.display());
    };
    let width = first.chars().count();
    if width == 0 {
        bail!("text map '{}' has an empty first row", path.display());
    }
    for (index, row) in rows.iter().enumerate() {
        if row.chars().count() != width {
            bail!(
                "text map '{}' row {} has width {}, expected {width}",
                path.display(),
                index + 1,
                row.chars().count()
            );
        }
        if row.chars().any(char::is_whitespace) {
            bail!(
                "text map '{}' row {} contains whitespace; use visible tile symbols only",
                path.display(),
                index + 1
            );
        }
    }
    Ok(())
}

fn copy_directory(source: &Path, destination: &Path) -> Result<()> {
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

fn write_launchers(output: &Path, executable_name: &str) -> Result<()> {
    let shell = output.join("run.sh");
    fs::write(
        &shell,
        format!("#!/usr/bin/env sh\ncd \"$(dirname \"$0\")\"\nexec ./{executable_name} \"$@\"\n"),
    )
    .with_context(|| format!("failed to write '{}'", shell.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&shell, fs::Permissions::from_mode(0o755))
            .with_context(|| format!("failed to mark '{}' executable", shell.display()))?;
    }

    let batch = output.join("run.bat");
    fs::write(
        &batch,
        format!("@echo off\r\ncd /d \"%~dp0\"\r\n{executable_name}\r\n"),
    )
    .with_context(|| format!("failed to write '{}'", batch.display()))?;
    Ok(())
}

fn write_package_readme(output: &Path, executable_name: &str) -> Result<()> {
    let readme = output.join("README-RUN.txt");
    fs::write(
        &readme,
        format!(
            "Playable game package\n\nKeep this directory together: `{executable_name}` needs the adjacent `assets` folder.\n\nLinux: run ./run.sh from a terminal.\nWindows: double-click run.bat.\nmacOS: open Terminal in this folder and run ./run.sh; an app bundle is not created by this command.\n\nThe bundled binary defaults to the Arena demo. Set GAME_DEMO=simple or GAME_DEMO=testbed before launching to select those bundled demos.\n"
        ),
    )
    .with_context(|| format!("failed to write '{}'", readme.display()))?;
    Ok(())
}

fn doctor() {
    println!("game environment doctor\n");

    let shader = executable_on_path("glslc") || executable_on_path("shaderc");
    report(
        shader,
        "shader compiler (glslc or shaderc)",
        "install the Vulkan SDK or your distribution's shaderc package; set GLSLC to an explicit path if needed",
    );

    let vulkan = command_succeeds("vulkaninfo", &["--summary"]);
    report(
        vulkan,
        "Vulkan loader and a usable driver",
        "install a Vulkan loader/driver, then run `vulkaninfo --summary` to check it",
    );

    let sdl3 = command_succeeds("pkg-config", &["--exists", "sdl3"])
        || std::env::var_os("SDL3_DIR").is_some();
    report(
        sdl3,
        "SDL3 development files",
        "install SDL3 (or set SDL3_DIR); on Linux, `pkg-config --exists sdl3` should succeed",
    );

    let validation = vulkan
        && Command::new("vulkaninfo")
            .output()
            .ok()
            .is_some_and(|output| {
                String::from_utf8_lossy(&output.stdout).contains("VK_LAYER_KHRONOS_validation")
            });
    if validation {
        println!("[ok] Vulkan validation layers");
    } else {
        println!(
            "[warn] Vulkan validation layers — install them for debug diagnostics, or run with GAME_DISABLE_VALIDATION=1"
        );
    }

    if shader && vulkan && sdl3 {
        println!("\nCore prerequisites look available. Try: cargo run -p game");
    } else {
        println!("\nFix the failed checks above, then run this command again.");
    }
}

fn report(ok: bool, name: &str, fix: &str) {
    if ok {
        println!("[ok] {name}");
    } else {
        println!("[fail] {name} — {fix}");
    }
}

fn command_succeeds(command: &str, args: &[&str]) -> bool {
    Command::new(command)
        .args(args)
        .status()
        .is_ok_and(|status| status.success())
}

fn executable_on_path(name: &str) -> bool {
    let Some(paths) = env::var_os("PATH") else {
        return false;
    };
    env::split_paths(&paths).any(|dir| {
        let plain = dir.join(name);
        if plain.is_file() {
            return true;
        }
        #[cfg(windows)]
        {
            return dir.join(format!("{name}.exe")).is_file();
        }
        #[cfg(not(windows))]
        false
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DemoTemplate {
    Simple,
    DataDriven,
}

impl DemoTemplate {
    fn parse(value: &str) -> Result<Self> {
        match value {
            "simple" => Ok(Self::Simple),
            "data-driven" => Ok(Self::DataDriven),
            other => bail!("unknown template '{other}'; expected simple or data-driven"),
        }
    }

    fn is_data_driven(self) -> bool {
        matches!(self, Self::DataDriven)
    }
}

fn new_demo(name_or_path: &str, template: DemoTemplate) -> Result<()> {
    let workspace = workspace_root()?;
    let destination = demo_destination(&workspace, name_or_path)?;
    if destination.exists() {
        bail!("destination '{}' already exists", destination.display());
    }

    let crate_name = crate_name_from_destination(&destination)?;
    let game_path = game_path_from_destination(&workspace, &destination)?;
    let game_starter_dependency = format!("{{ path = \"{game_path}/crates/game-starter\" }}");
    let title = title_from_crate_name(&crate_name);

    let template_path = workspace.join(if template.is_data_driven() {
        "templates/data-driven-demo"
    } else {
        "templates/simple-demo"
    });
    let mut values = HashMap::new();
    values.insert("crate_name", crate_name.as_str());
    values.insert("game_starter_dependency", game_starter_dependency.as_str());
    values.insert("title", title.as_str());

    copy_template(&template_path, &destination, &values)?;
    seed_beginner_assets(&workspace, &destination)?;

    println!("created demo at {}", destination.display());
    if template.is_data_driven() {
        println!("setup lives in assets/game.ron; src/main.rs is ready for optional custom rules");
    } else {
        println!("setup lives in src/main.rs with beginner Rust builder chains");
    }
    println!("run it with:");
    println!("    cd {}", destination.display());
    println!("    cargo run");

    Ok(())
}

fn workspace_root() -> Result<PathBuf> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| anyhow!("xtask manifest has no parent directory"))
}

fn demo_destination(workspace: &Path, name_or_path: &str) -> Result<PathBuf> {
    let raw = Path::new(name_or_path);
    if raw.is_absolute() || raw.components().count() > 1 {
        return Ok(raw.to_path_buf());
    }

    let parent = workspace
        .parent()
        .ok_or_else(|| anyhow!("workspace '{}' has no parent", workspace.display()))?;
    Ok(parent.join(raw))
}

fn crate_name_from_destination(destination: &Path) -> Result<String> {
    let file_name = destination
        .file_name()
        .and_then(OsStr::to_str)
        .ok_or_else(|| {
            anyhow!(
                "destination '{}' has no final path segment",
                destination.display()
            )
        })?;
    let mut name = String::new();
    let mut last_was_dash = false;
    for ch in file_name.chars() {
        let ch = ch.to_ascii_lowercase();
        if ch.is_ascii_alphanumeric() {
            name.push(ch);
            last_was_dash = false;
        } else if matches!(ch, '-' | '_') && !last_was_dash && !name.is_empty() {
            name.push('-');
            last_was_dash = true;
        }
    }
    while name.ends_with('-') {
        name.pop();
    }
    if name.is_empty() {
        bail!("could not derive a crate name from '{}'", file_name);
    }
    Ok(name)
}

fn title_from_crate_name(crate_name: &str) -> String {
    crate_name
        .split('-')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn game_path_from_destination(workspace: &Path, destination: &Path) -> Result<String> {
    if destination.parent() == workspace.parent() {
        let root_name = workspace
            .file_name()
            .and_then(OsStr::to_str)
            .ok_or_else(|| {
                anyhow!(
                    "workspace '{}' has no final path segment",
                    workspace.display()
                )
            })?;
        Ok(format!("../{root_name}"))
    } else {
        Ok(workspace.display().to_string())
    }
}

fn copy_template(src: &Path, dst: &Path, values: &HashMap<&str, &str>) -> Result<()> {
    if !src.is_dir() {
        bail!("template directory '{}' does not exist", src.display());
    }
    fs::create_dir_all(dst).with_context(|| format!("failed to create '{}'", dst.display()))?;

    for entry in fs::read_dir(src).with_context(|| format!("failed to read '{}'", src.display()))? {
        let entry = entry?;
        if entry.file_name() == "cargo-generate.toml" {
            continue;
        }
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_template(&src_path, &dst_path, values)?;
        } else {
            copy_template_file(&src_path, &dst_path, values)?;
        }
    }

    Ok(())
}

fn copy_template_file(src: &Path, dst: &Path, values: &HashMap<&str, &str>) -> Result<()> {
    let mut text =
        fs::read_to_string(src).with_context(|| format!("failed to read '{}'", src.display()))?;
    for (key, value) in values {
        text = text.replace(&format!("{{{{{key}}}}}"), value);
    }
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create '{}'", parent.display()))?;
    }
    fs::write(dst, text).with_context(|| format!("failed to write '{}'", dst.display()))
}

fn seed_beginner_assets(workspace: &Path, destination: &Path) -> Result<()> {
    let assets = workspace.join("assets");
    let textures = destination.join("assets/textures");
    let sounds = destination.join("assets/sounds");
    let maps = destination.join("assets/maps");
    fs::create_dir_all(&textures)
        .with_context(|| format!("failed to create '{}'", textures.display()))?;
    fs::create_dir_all(&sounds)
        .with_context(|| format!("failed to create '{}'", sounds.display()))?;
    fs::create_dir_all(&maps).with_context(|| format!("failed to create '{}'", maps.display()))?;

    let placeholder_texture = assets.join("textures/test.png");
    for name in ["player", "slime", "coin", "floor", "wall"] {
        copy_asset(&placeholder_texture, &textures.join(format!("{name}.png")))?;
    }
    copy_asset(&assets.join("sounds/hit.wav"), &sounds.join("hit.wav"))?;
    copy_asset(
        &assets.join("maps/beginner_text_map.txt"),
        &maps.join("level_1.txt"),
    )?;
    Ok(())
}

fn copy_asset(src: &Path, dst: &Path) -> Result<()> {
    fs::copy(src, dst)
        .with_context(|| format!("failed to copy '{}' to '{}'", src.display(), dst.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{validate_package_assets, validate_text_map, workspace_root};

    #[test]
    fn workspace_assets_pass_the_same_prepackage_validation_as_a_release() {
        let workspace = workspace_root().unwrap();
        validate_package_assets(&workspace.join("assets")).unwrap();
    }

    #[test]
    fn text_map_validation_names_the_ragged_row() {
        let path = std::env::temp_dir().join(format!(
            "game-package-map-validation-{}.txt",
            std::process::id()
        ));
        std::fs::write(&path, "####\n##\n").unwrap();
        let error = validate_text_map(&path).unwrap_err().to_string();

        assert!(error.contains("row 2 has width 2, expected 4"));
    }
}

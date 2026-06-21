use std::collections::HashMap;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, anyhow, bail};

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("new-demo") => {
            let name = args
                .next()
                .ok_or_else(|| anyhow!("usage: cargo xtask new-demo <name-or-path>"))?;
            if let Some(extra) = args.next() {
                bail!("unexpected extra argument '{extra}'");
            }
            new_demo(&name)
        }
        Some("doctor") => {
            if let Some(extra) = args.next() {
                bail!("unexpected argument for doctor: '{extra}'");
            }
            doctor();
            Ok(())
        }
        _ => {
            bail!(
                "usage:\n    cargo xtask new-demo <name-or-path>\n    cargo xtask doctor\n\nCreates an outside-workspace beginner demo, or checks local graphics prerequisites."
            );
        }
    }
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

fn new_demo(name_or_path: &str) -> Result<()> {
    let workspace = workspace_root()?;
    let destination = demo_destination(&workspace, name_or_path)?;
    if destination.exists() {
        bail!("destination '{}' already exists", destination.display());
    }

    let crate_name = crate_name_from_destination(&destination)?;
    let game_path = game_path_from_destination(&workspace, &destination)?;
    let game_starter_dependency = format!("{{ path = \"{game_path}/crates/game-starter\" }}");
    let title = title_from_crate_name(&crate_name);

    let template = workspace.join("templates/simple-demo");
    let mut values = HashMap::new();
    values.insert("crate_name", crate_name.as_str());
    values.insert("game_starter_dependency", game_starter_dependency.as_str());
    values.insert("title", title.as_str());

    copy_template(&template, &destination, &values)?;
    seed_beginner_assets(&workspace, &destination)?;

    println!("created demo at {}", destination.display());
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
    for name in ["player", "slime", "floor", "wall"] {
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

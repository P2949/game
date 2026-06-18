use std::collections::HashMap;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

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
        _ => {
            bail!(
                "usage:\n    cargo xtask new-demo <name-or-path>\n\nCreates an outside-workspace beginner demo."
            );
        }
    }
}

fn new_demo(name_or_path: &str) -> Result<()> {
    let workspace = workspace_root()?;
    let destination = demo_destination(&workspace, name_or_path)?;
    if destination.exists() {
        bail!("destination '{}' already exists", destination.display());
    }

    let crate_name = crate_name_from_destination(&destination)?;
    let game_path = game_path_from_destination(&workspace, &destination)?;
    let title = title_from_crate_name(&crate_name);

    let template = workspace.join("templates/simple-demo");
    let mut values = HashMap::new();
    values.insert("crate_name", crate_name.as_str());
    values.insert("game_path", game_path.as_str());
    values.insert("title", title.as_str());

    copy_template(&template, &destination, &values)?;
    copy_dir(&workspace.join("assets"), &destination.join("assets"))?;

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

fn copy_dir(src: &Path, dst: &Path) -> Result<()> {
    if !src.is_dir() {
        bail!("directory '{}' does not exist", src.display());
    }
    fs::create_dir_all(dst).with_context(|| format!("failed to create '{}'", dst.display()))?;
    for entry in fs::read_dir(src).with_context(|| format!("failed to read '{}'", src.display()))? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path).with_context(|| {
                format!(
                    "failed to copy '{}' to '{}'",
                    src_path.display(),
                    dst_path.display()
                )
            })?;
        }
    }
    Ok(())
}

use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow, bail};

use crate::assets::{asset_ignore_patterns_from_game_file, validate_assets_dir_with_ignores};
use crate::project::{NoRustPathOverrides, resolve_no_rust_project_paths_with_env};

pub(crate) fn asset_check_command(args: impl Iterator<Item = String>) -> Result<()> {
    let overrides = parse_path_overrides(args, "asset-check")?;
    let current = std::env::current_dir()?;
    asset_check_at(&current, &overrides)
}

fn asset_check_at(current: &Path, overrides: &NoRustPathOverrides) -> Result<()> {
    let paths = resolve_no_rust_project_paths_with_env(current, overrides);
    let ignore = asset_ignore_patterns_from_game_file(&paths.game_file)?;
    validate_assets_dir_with_ignores(&paths.asset_dir, false, ignore)?;
    println!("assets look valid");
    Ok(())
}

fn parse_path_overrides(
    args: impl Iterator<Item = String>,
    command: &str,
) -> Result<NoRustPathOverrides> {
    let mut overrides = NoRustPathOverrides::default();
    let mut args = args.peekable();
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--project" => overrides.project = Some(next_path(&mut args, "--project")?),
            "--assets" => overrides.assets = Some(next_path(&mut args, "--assets")?),
            extra => bail!("unexpected {command} argument '{extra}'"),
        }
    }
    Ok(overrides)
}

fn next_path(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<PathBuf> {
    args.next()
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("{flag} needs a path"))
}

#[cfg(test)]
mod tests {
    use super::asset_check_at;
    use crate::project::NoRustPathOverrides;
    use crate::templates::{DemoTemplate, new_project};
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn asset_check_respects_project_assets_and_default_layout() {
        let parent = temp_dir("asset-check-parent");
        let project = parent.join("project");
        new_project(&project, DemoTemplate::NoRust, "{ path = \"unused\" }").unwrap();

        asset_check_at(&project, &NoRustPathOverrides::default()).unwrap();
        asset_check_at(
            &parent,
            &NoRustPathOverrides {
                project: Some(project.clone()),
                ..NoRustPathOverrides::default()
            },
        )
        .unwrap();

        let custom_assets = parent.join("custom-assets");
        fs::rename(project.join("assets"), &custom_assets).unwrap();
        asset_check_at(
            &parent,
            &NoRustPathOverrides {
                project: Some(project),
                assets: Some(custom_assets),
                ..NoRustPathOverrides::default()
            },
        )
        .unwrap();
    }

    fn temp_dir(name: &str) -> PathBuf {
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
        fs::create_dir_all(&dir).unwrap();
        dir
    }
}

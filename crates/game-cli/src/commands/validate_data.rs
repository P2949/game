use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow, bail};

use crate::project::{NoRustPathOverrides, resolve_no_rust_project_paths_with_env};

pub(crate) fn validate_data_command(args: impl Iterator<Item = String>) -> Result<()> {
    let options = parse_validate_data_args(args)?;
    let current = std::env::current_dir()?;
    let paths = resolve_no_rust_project_paths_with_env(&current, &options.overrides);
    let data_file = options.path.unwrap_or_else(|| paths.game_file.clone());
    validate_data_file(&data_file, &paths.asset_dir)?;
    println!("game config is valid");
    Ok(())
}

struct ValidateDataOptions {
    path: Option<PathBuf>,
    overrides: NoRustPathOverrides,
}

fn parse_validate_data_args(args: impl Iterator<Item = String>) -> Result<ValidateDataOptions> {
    let mut path = None;
    let mut overrides = NoRustPathOverrides::default();
    let mut args = args.peekable();
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--project" => overrides.project = Some(next_path(&mut args, "--project")?),
            "--file" => overrides.file = Some(next_path(&mut args, "--file")?),
            "--assets" => overrides.assets = Some(next_path(&mut args, "--assets")?),
            "--legacy" => {}
            value if value.starts_with("--") => {
                bail!("unexpected validate-data argument '{value}'")
            }
            value => {
                if path.replace(PathBuf::from(value)).is_some() {
                    bail!("validate-data accepts at most one data file path");
                }
            }
        }
    }
    Ok(ValidateDataOptions { path, overrides })
}

fn next_path(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<PathBuf> {
    args.next()
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("{flag} needs a path"))
}

fn validate_data_file(path: &Path, asset_root: &Path) -> Result<()> {
    let path = resolve_data_path(path, asset_root);
    match path.extension().and_then(|extension| extension.to_str()) {
        Some(extension) if extension.eq_ignore_ascii_case("toml") => {
            game_kit::data::validate_authoring_file_with_asset_root(&path, asset_root)
        }
        Some(extension) if extension.eq_ignore_ascii_case("ron") => {
            eprintln!(
                "warning: RON data files are legacy; use game.toml for primary no-Rust authoring"
            );
            game_kit::data::validate_beginner_game_file(&path)
        }
        _ => bail!(
            "unsupported data file '{}'. Use game.toml for primary no-Rust authoring; RON is legacy.",
            path.display()
        ),
    }
}

fn resolve_data_path(path: &Path, asset_root: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    let project_root = asset_root.parent().unwrap_or_else(|| Path::new("."));
    let project_relative = project_root.join(path);
    if project_relative.is_file() {
        project_relative
    } else {
        asset_root.join(path)
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_validate_data_args, resolve_data_path};
    use std::fs;
    use std::path::{Path, PathBuf};

    #[test]
    fn validate_data_defaults_to_project_game_toml() {
        let options = parse_validate_data_args(std::iter::empty()).unwrap();
        assert!(options.path.is_none());

        let project = temp_dir("validate-data-default");
        fs::create_dir_all(project.join("assets")).unwrap();
        fs::write(project.join("game.toml"), "version = 2\n").unwrap();

        assert_eq!(
            resolve_data_path(Path::new("game.toml"), &project.join("assets")),
            project.join("game.toml")
        );
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

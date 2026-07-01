use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};

pub(crate) fn configured_asset_root() -> PathBuf {
    env::var_os("GAME_ASSET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("assets"))
}

pub(crate) fn normalize_validate_data_path(path: impl AsRef<Path>, asset_root: &Path) -> PathBuf {
    let path = path.as_ref();
    if path.is_absolute() || asset_root.is_absolute() {
        return path.to_path_buf();
    }

    path.strip_prefix(asset_root)
        .map(Path::to_path_buf)
        .unwrap_or_else(|_| path.to_path_buf())
}

pub(crate) fn xtask_demo_destination(workspace: &Path, name_or_path: &str) -> Result<PathBuf> {
    let raw = Path::new(name_or_path);
    if raw.is_absolute() || raw.components().count() > 1 {
        return Ok(raw.to_path_buf());
    }

    let parent = workspace
        .parent()
        .ok_or_else(|| anyhow!("workspace '{}' has no parent", workspace.display()))?;
    Ok(parent.join(raw))
}

pub(crate) fn game_path_from_destination(workspace: &Path, destination: &Path) -> Result<String> {
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

pub(crate) fn workspace_root() -> Result<PathBuf> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .ok_or_else(|| anyhow!("game-cli manifest has no workspace parent"))
}

pub(crate) fn source_assets_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets")
}

pub(crate) fn absolutize_from_current(path: &Path) -> Result<PathBuf> {
    Ok(absolutize_from(&env::current_dir()?, path))
}

pub(crate) fn absolutize_from(base: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    }
}

pub(crate) fn executable_name(package_name: &str) -> String {
    if cfg!(windows) {
        format!("{package_name}.exe")
    } else {
        package_name.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_validate_data_path;

    #[test]
    fn validate_data_accepts_asset_relative_or_assets_prefixed_paths() {
        let asset_root = std::path::Path::new("assets");
        assert_eq!(
            normalize_validate_data_path("game.ron", asset_root),
            std::path::PathBuf::from("game.ron")
        );
        assert_eq!(
            normalize_validate_data_path("assets/game.ron", asset_root),
            std::path::PathBuf::from("game.ron")
        );

        let absolute = std::env::temp_dir().join("game.ron");
        assert_eq!(
            normalize_validate_data_path(&absolute, asset_root),
            absolute
        );
    }
}

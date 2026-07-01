use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};

pub(crate) struct PackageManifestInfo {
    pub(crate) package_name: String,
    pub(crate) asset_dir: PathBuf,
}

pub(crate) fn package_info_from_manifest(manifest: &Path) -> Result<PackageManifestInfo> {
    let source = fs::read_to_string(manifest)
        .with_context(|| format!("failed to read manifest '{}'", manifest.display()))?;
    let mut section = "";
    let mut package_name = None;
    let mut asset_dir = None;
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            section = trimmed.trim_matches(&['[', ']'][..]);
            continue;
        }
        let Some((key, value)) = trimmed.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim().trim_matches('"');
        match (section, key) {
            ("package", "name") if !value.is_empty() => package_name = Some(value.to_string()),
            ("package.metadata.game", "asset_dir") if !value.is_empty() => {
                asset_dir = Some(PathBuf::from(value));
            }
            _ => {}
        }
    }
    let package_name = package_name
        .ok_or_else(|| anyhow!("could not find [package] name in '{}'", manifest.display()))?;
    Ok(PackageManifestInfo {
        package_name,
        asset_dir: asset_dir.unwrap_or_else(|| PathBuf::from("assets")),
    })
}

#[cfg(test)]
mod tests {
    use super::package_info_from_manifest;

    #[test]
    fn package_name_parser_reads_package_section() {
        let path =
            std::env::temp_dir().join(format!("game-cli-manifest-{}.toml", std::process::id()));
        std::fs::write(&path, "[package]\nname = \"demo_game\"\n").unwrap();
        assert_eq!(
            package_info_from_manifest(&path).unwrap().package_name,
            "demo_game"
        );
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn package_metadata_parser_reads_asset_dir_with_default() {
        let path = std::env::temp_dir().join(format!(
            "game-cli-metadata-manifest-{}.toml",
            std::process::id()
        ));
        std::fs::write(
            &path,
            "[package]\nname = \"demo_game\"\n\n[package.metadata.game]\nasset_dir = \"game-assets\"\n",
        )
        .unwrap();
        let info = package_info_from_manifest(&path).unwrap();
        assert_eq!(info.package_name, "demo_game");
        assert_eq!(info.asset_dir, std::path::PathBuf::from("game-assets"));

        std::fs::write(&path, "[package]\nname = \"fallback_game\"\n").unwrap();
        let info = package_info_from_manifest(&path).unwrap();
        assert_eq!(info.package_name, "fallback_game");
        assert_eq!(info.asset_dir, std::path::PathBuf::from("assets"));
        std::fs::remove_file(path).unwrap();
    }
}

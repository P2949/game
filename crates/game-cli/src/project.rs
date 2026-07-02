use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectKind {
    NoRustPackage,
    RustStarterProject,
    WorkspaceDemo,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoRustProjectPaths {
    pub root: PathBuf,
    pub game_file: PathBuf,
    pub asset_dir: PathBuf,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NoRustPathOverrides {
    pub project: Option<PathBuf>,
    pub file: Option<PathBuf>,
    pub assets: Option<PathBuf>,
}

pub fn detect_project_kind(root: impl AsRef<Path>) -> Result<ProjectKind> {
    let root = root.as_ref();
    let game_file = root.join("game.toml");
    let cargo_manifest = root.join("Cargo.toml");
    let has_game_file = game_file.is_file();
    let has_cargo_manifest = cargo_manifest.is_file();

    if has_game_file && !has_cargo_manifest {
        return Ok(ProjectKind::NoRustPackage);
    }

    if has_cargo_manifest {
        let source = fs::read_to_string(&cargo_manifest).with_context(|| {
            format!(
                "failed to read project manifest '{}'",
                cargo_manifest.display()
            )
        })?;
        if has_section(&source, "package.metadata.game") {
            return Ok(ProjectKind::RustStarterProject);
        }
        if is_engine_workspace(root, &source) {
            return Ok(ProjectKind::WorkspaceDemo);
        }
        if has_game_file {
            return Ok(ProjectKind::NoRustPackage);
        }
        return Ok(ProjectKind::RustStarterProject);
    }

    if has_game_file {
        return Ok(ProjectKind::NoRustPackage);
    }

    bail!(
        "could not detect project kind in '{}'; expected game.toml or Cargo.toml",
        root.display()
    )
}

pub fn resolve_no_rust_project_paths(
    current_dir: impl AsRef<Path>,
    overrides: &NoRustPathOverrides,
) -> NoRustProjectPaths {
    let current_dir = current_dir.as_ref();
    let root = overrides
        .project
        .as_deref()
        .map(|project| absolutize_from(current_dir, project))
        .unwrap_or_else(|| current_dir.to_path_buf());
    let game_file = overrides
        .file
        .as_deref()
        .map(|file| absolutize_from(&root, file))
        .unwrap_or_else(|| root.join("game.toml"));
    let asset_dir = overrides
        .assets
        .as_deref()
        .map(|assets| absolutize_from(&root, assets))
        .unwrap_or_else(|| root.join("assets"));

    NoRustProjectPaths {
        root,
        game_file,
        asset_dir,
    }
}

pub fn resolve_no_rust_project_paths_with_env(
    current_dir: impl AsRef<Path>,
    overrides: &NoRustPathOverrides,
) -> NoRustProjectPaths {
    let mut overrides = overrides.clone();
    if overrides.assets.is_none() {
        overrides.assets = env::var_os("GAME_ASSET_DIR").map(PathBuf::from);
    }
    resolve_no_rust_project_paths(current_dir, &overrides)
}

fn absolutize_from(base: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    }
}

fn is_engine_workspace(root: &Path, manifest_source: &str) -> bool {
    has_section(manifest_source, "workspace")
        && root.join("crates/game-core/Cargo.toml").is_file()
        && root.join("bin/game/Cargo.toml").is_file()
}

fn has_section(source: &str, expected: &str) -> bool {
    source.lines().any(|line| {
        let trimmed = line.trim();
        trimmed.starts_with('[')
            && trimmed.ends_with(']')
            && trimmed.trim_matches(&['[', ']'][..]) == expected
    })
}

#[cfg(test)]
mod tests {
    use super::{
        NoRustPathOverrides, ProjectKind, detect_project_kind, resolve_no_rust_project_paths,
    };
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TempProject {
        root: PathBuf,
    }

    impl TempProject {
        fn new(name: &str) -> Self {
            let stamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let root = std::env::temp_dir().join(format!(
                "game-cli-project-{name}-{}-{stamp}",
                std::process::id()
            ));
            fs::create_dir_all(&root).unwrap();
            Self { root }
        }

        fn path(&self) -> &Path {
            &self.root
        }

        fn write(&self, relative: &str, contents: &str) {
            let path = self.root.join(relative);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(path, contents).unwrap();
        }
    }

    impl Drop for TempProject {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn detects_no_rust_package_from_root_game_toml_without_cargo() {
        let project = TempProject::new("no-rust");
        project.write("game.toml", "[game]\ntitle = \"Demo\"\n");

        assert_eq!(
            detect_project_kind(project.path()).unwrap(),
            ProjectKind::NoRustPackage
        );
    }

    #[test]
    fn detects_rust_starter_project_from_package_metadata() {
        let project = TempProject::new("rust-starter");
        project.write(
            "Cargo.toml",
            "[package]\nname = \"demo\"\n\n[package.metadata.game]\nasset_dir = \"assets\"\n",
        );

        assert_eq!(
            detect_project_kind(project.path()).unwrap(),
            ProjectKind::RustStarterProject
        );
    }

    #[test]
    fn detects_workspace_demo_from_engine_workspace_shape() {
        let project = TempProject::new("workspace");
        project.write("Cargo.toml", "[workspace]\nmembers = []\n");
        project.write(
            "crates/game-core/Cargo.toml",
            "[package]\nname = \"game-core\"\n",
        );
        project.write("bin/game/Cargo.toml", "[package]\nname = \"game\"\n");

        assert_eq!(
            detect_project_kind(project.path()).unwrap(),
            ProjectKind::WorkspaceDemo
        );
    }

    #[test]
    fn package_with_no_cargo_manifest_is_not_treated_as_rust_starter() {
        let project = TempProject::new("no-cargo");
        project.write("game.toml", "[game]\ntitle = \"No Cargo\"\n");

        assert_ne!(
            detect_project_kind(project.path()).unwrap(),
            ProjectKind::RustStarterProject
        );
    }

    #[test]
    fn resolves_default_no_rust_paths_from_current_directory() {
        let project = TempProject::new("default-paths");

        let paths = resolve_no_rust_project_paths(project.path(), &NoRustPathOverrides::default());

        assert_eq!(paths.root, project.path());
        assert_eq!(paths.game_file, project.path().join("game.toml"));
        assert_eq!(paths.asset_dir, project.path().join("assets"));
    }

    #[test]
    fn resolves_explicit_project_file_and_assets_overrides() {
        let current = TempProject::new("current");
        let project = current.path().join("nested-game");
        let absolute_assets = current.path().join("shared-assets");

        let paths = resolve_no_rust_project_paths(
            current.path(),
            &NoRustPathOverrides {
                project: Some(PathBuf::from("nested-game")),
                file: Some(PathBuf::from("config/game.toml")),
                assets: Some(absolute_assets.clone()),
            },
        );

        assert_eq!(paths.root, project);
        assert_eq!(paths.game_file, paths.root.join("config/game.toml"));
        assert_eq!(paths.asset_dir, absolute_assets);
    }
}

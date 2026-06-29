use std::path::{Path, PathBuf};

fn asset_root() -> PathBuf {
    std::env::var_os("GAME_ASSET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("assets"))
}

pub(crate) fn beginner_asset_path_exists(path: impl AsRef<Path>) -> bool {
    let path = path.as_ref();
    if path.is_absolute() {
        return path.is_file();
    }

    let root = asset_root();
    if root.is_absolute() {
        return root.join(path).is_file();
    }

    let relative = root.join(path);
    search_roots()
        .into_iter()
        .any(|directory| directory.join(&relative).is_file())
        || Path::new(&relative).is_file()
}

pub(crate) fn beginner_asset_file(path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();
    if path.is_absolute() {
        return path.to_path_buf();
    }

    let root = asset_root();
    if root.is_absolute() {
        return root.join(path);
    }

    let relative = root.join(path);
    search_roots()
        .into_iter()
        .map(|directory| directory.join(&relative))
        .find(|candidate| candidate.is_file())
        .unwrap_or_else(|| fallback_relative_path(relative))
}

pub(crate) fn beginner_asset_directory(folder: impl AsRef<Path>) -> PathBuf {
    let folder = folder.as_ref();
    if folder.is_absolute() {
        return folder.to_path_buf();
    }

    let root = asset_root();
    if root.is_absolute() {
        return root.join(folder);
    }

    let relative = root.join(folder);
    search_roots()
        .into_iter()
        .map(|directory| directory.join(&relative))
        .find(|candidate| candidate.is_dir())
        .unwrap_or_else(|| fallback_relative_path(relative))
}

fn fallback_relative_path(relative: PathBuf) -> PathBuf {
    std::env::current_dir()
        .map(|current_dir| current_dir.join(&relative))
        .unwrap_or(relative)
}

fn search_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(executable) = std::env::current_exe()
        && let Some(parent) = executable.parent()
    {
        push_ancestors(&mut roots, parent);
    }
    if let Ok(current_dir) = std::env::current_dir() {
        push_ancestors(&mut roots, &current_dir);
    }
    roots
}

fn push_ancestors(roots: &mut Vec<PathBuf>, path: &Path) {
    for ancestor in path.ancestors() {
        let candidate = ancestor.to_path_buf();
        if !roots.iter().any(|existing| existing == &candidate) {
            roots.push(candidate);
        }
    }
}

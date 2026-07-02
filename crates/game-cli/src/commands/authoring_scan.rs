use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use walkdir::WalkDir;

use crate::project::{
    NoRustPathOverrides, resolve_no_rust_project_paths, resolve_no_rust_project_paths_with_env,
};

const FORBIDDEN_PROJECT_FILES: &[&str] = &["Cargo.toml", "Cargo.lock", "build.rs", "src/main.rs"];

const FORBIDDEN_DATA_TOKENS: &[&str] = &[
    "Some(",
    "None",
    "Ok(",
    "Err(",
    "Result",
    "Vec",
    "HashMap",
    "BTreeMap",
    "Player((",
    "Enemy((",
    "Pickup((",
    "Door((",
    "Projectile((",
    "Spawner((",
    "Trigger((",
    "Checkpoint((",
    "TextMap((",
    "TextMapAuto((",
    "Tiled((",
    "Ldtk((",
    "TopDownControls",
    "PlayerCollectsPickups",
    "EnemiesDamagePlayer",
    "CameraFollowsPlayer",
    "ShowScore",
    "ShowPlayerHealth",
    "::",
    "=>",
    "fn ",
    "impl ",
    "struct ",
    "enum ",
    "trait ",
    "pub ",
    "use ",
    "match ",
    "<",
    ">",
];

const FORBIDDEN_ENGINE_VOCAB: &[&str] = &[
    "GameCtx",
    "StartupGameCtx",
    "EntityId",
    "Component",
    "World",
    "Transform",
    "Velocity",
    "Sprite::new",
    "Collider::box_of",
    "CommandQueue",
    "RuntimeConfig",
    "game_runtime",
    "game_core",
    "game_renderer_vulkan",
    "game_platform_sdl",
    "ash",
    "sdl3",
    "swapchain",
    "descriptor",
    "allocator",
    "lifetime",
    "generic",
    "trait",
    "cargo run",
    "cargo check",
    "rustc",
];

pub(crate) fn authoring_scan_command(args: impl Iterator<Item = String>) -> Result<()> {
    let options = parse_authoring_scan_options(args)?;
    let current = std::env::current_dir().context("failed to resolve current directory")?;
    let paths = resolve_no_rust_project_paths_with_env(&current, &options);
    scan_no_rust_project(&paths.root)?;
    println!("authoring surface looks clean");
    Ok(())
}

fn parse_authoring_scan_options(args: impl Iterator<Item = String>) -> Result<NoRustPathOverrides> {
    let mut overrides = NoRustPathOverrides::default();
    let mut args = args.peekable();
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--project" => {
                overrides.project = Some(next_path(&mut args, "--project")?);
            }
            "--file" => {
                overrides.file = Some(next_path(&mut args, "--file")?);
            }
            "--assets" => {
                overrides.assets = Some(next_path(&mut args, "--assets")?);
            }
            extra => bail!("unexpected authoring-scan argument '{extra}'"),
        }
    }
    Ok(overrides)
}

fn next_path(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<PathBuf> {
    args.next()
        .map(PathBuf::from)
        .ok_or_else(|| anyhow::anyhow!("{flag} needs a path"))
}

pub(crate) fn scan_no_rust_project(project: &Path) -> Result<()> {
    let paths = resolve_no_rust_project_paths(project, &NoRustPathOverrides::default());
    if !paths.game_file.is_file() {
        bail!(
            "primary no-Rust package '{}' needs game.toml",
            project.display()
        );
    }
    if !paths.asset_dir.is_dir() {
        bail!(
            "primary no-Rust package '{}' needs assets/",
            project.display()
        );
    }

    let mut findings = Vec::new();
    for forbidden in FORBIDDEN_PROJECT_FILES {
        if project.join(forbidden).exists() {
            findings.push(format!(
                "{forbidden} is not allowed in a primary no-Rust package"
            ));
        }
    }

    for entry in WalkDir::new(project) {
        let entry = entry.with_context(|| format!("failed to walk '{}'", project.display()))?;
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let relative = path.strip_prefix(project).unwrap_or(path);
        if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            findings.push(format!(
                "{} is Rust source; primary no-Rust packages must not contain .rs files",
                relative.display()
            ));
            continue;
        }
        if relative == Path::new("assets/game.ron") {
            findings.push("assets/game.ron is legacy RON, not primary authoring".to_owned());
            continue;
        }

        if path.extension().and_then(|extension| extension.to_str()) == Some("toml") {
            scan_file_for_tokens(path, relative, FORBIDDEN_DATA_TOKENS, &mut findings)?;
        }
        if is_text_surface_file(path) {
            scan_file_for_tokens(path, relative, FORBIDDEN_ENGINE_VOCAB, &mut findings)?;
        }
    }

    if findings.is_empty() {
        Ok(())
    } else {
        bail!(
            "primary no-Rust authoring scan failed for '{}':\n{}",
            project.display(),
            findings.join("\n")
        );
    }
}

fn is_text_surface_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("toml" | "txt" | "md")
    )
}

fn scan_file_for_tokens(
    path: &Path,
    relative: &Path,
    tokens: &[&str],
    findings: &mut Vec<String>,
) -> Result<()> {
    let source = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read '{}'", path.display()))?;
    for token in tokens {
        if source_contains_token(&source, token) {
            findings.push(format!(
                "{} contains forbidden token {token:?}",
                relative.display()
            ));
        }
    }
    Ok(())
}

fn source_contains_token(source: &str, token: &str) -> bool {
    match token {
        "fn " | "impl " | "struct " | "enum " | "trait " | "pub " | "use " | "match " => {
            contains_whole_word(source, token.trim())
        }
        "None" | "Result" | "Vec" | "HashMap" | "BTreeMap" => contains_whole_word(source, token),
        _ => source.contains(token),
    }
}

fn contains_whole_word(source: &str, word: &str) -> bool {
    source.match_indices(word).any(|(start, _)| {
        let before = source[..start].chars().next_back();
        let after = source[start + word.len()..].chars().next();
        before.is_none_or(|ch| !is_word_char(ch)) && after.is_none_or(|ch| !is_word_char(ch))
    })
}

fn is_word_char(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphanumeric()
}

#[cfg(test)]
mod tests {
    use super::scan_no_rust_project;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn authoring_scan_accepts_clean_primary_package() {
        let project = temp_project("scan-clean");
        fs::create_dir_all(project.join("assets")).unwrap();
        fs::write(
            project.join("game.toml"),
            "version = 2\n\n[game]\ntitle = \"Clean\"\n",
        )
        .unwrap();

        scan_no_rust_project(&project).unwrap();
    }

    #[test]
    fn authoring_scan_rejects_rust_project_files_and_ron_tokens() {
        let project = temp_project("scan-rejects");
        fs::create_dir_all(project.join("assets")).unwrap();
        fs::write(project.join("Cargo.toml"), "[package]\n").unwrap();
        fs::write(
            project.join("game.toml"),
            "version = 2\n\n[[prefab]]\nkind = \"player\"\n# Player((\n",
        )
        .unwrap();

        let error = scan_no_rust_project(&project).unwrap_err().to_string();
        assert!(error.contains("Cargo.toml is not allowed"));
        assert!(error.contains("forbidden token \"Player((\""));
    }

    #[test]
    fn authoring_scan_does_not_match_keywords_inside_words() {
        let project = temp_project("scan-keyword-boundaries");
        fs::create_dir_all(project.join("assets")).unwrap();
        fs::write(
            project.join("game.toml"),
            "version = 2\n\n[[rules.custom]]\nname = \"countdown\"\ndata = { fuse = 3.0 }\n",
        )
        .unwrap();

        scan_no_rust_project(&project).unwrap();
    }

    fn temp_project(name: &str) -> PathBuf {
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

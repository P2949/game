use std::ffi::OsStr;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, anyhow, bail};
use fontdue::{Font, FontSettings};
use image::ImageReader;
use walkdir::WalkDir;

#[derive(Clone, Copy)]
pub(crate) struct AssetCheckOptions {
    pub(crate) deny_unknown: bool,
}

impl AssetCheckOptions {
    pub(crate) fn deny_unknown() -> Self {
        Self { deny_unknown: true }
    }
}

pub(crate) fn validate_assets_dir(assets: &Path, require_builtin_font: bool) -> Result<()> {
    validate_assets_dir_with_options(
        assets,
        require_builtin_font,
        AssetCheckOptions::deny_unknown(),
    )
}

fn validate_assets_dir_with_options(
    assets: &Path,
    require_builtin_font: bool,
    options: AssetCheckOptions,
) -> Result<()> {
    if !assets.is_dir() {
        bail!("assets directory '{}' does not exist", assets.display());
    }
    if require_builtin_font {
        let font = assets.join("fonts/DejaVuSans.ttf");
        if !font.is_file() {
            bail!("required release font '{}' does not exist", font.display());
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
        validate_asset_file(entry.path(), assets, options)?;
    }
    if checked == 0 {
        bail!("assets directory '{}' is empty", assets.display());
    }
    Ok(())
}

fn validate_asset_file(path: &Path, asset_root: &Path, options: AssetCheckOptions) -> Result<()> {
    if is_ignored_asset_metadata(path) {
        return Ok(());
    }

    match path.extension().and_then(|extension| extension.to_str()) {
        Some(extension) if extension.eq_ignore_ascii_case("png") => {
            let image = ImageReader::open(path)
                .with_context(|| format!("could not open PNG '{}'", path.display()))?
                .with_guessed_format()
                .with_context(|| format!("could not identify PNG '{}'", path.display()))?
                .decode()
                .with_context(|| format!("could not decode PNG '{}'", path.display()))?;
            let width = image.width();
            let height = image.height();
            if width == 0 || height == 0 {
                bail!("PNG '{}' has zero width or height", path.display());
            }
            if width > 8192 || height > 8192 {
                bail!(
                    "PNG '{}' is {}x{}, which is unusually large for a beginner asset; keep textures at 8192px or smaller on each side",
                    path.display(),
                    width,
                    height
                );
            }
        }
        Some(extension) if extension.eq_ignore_ascii_case("ttf") => {
            let bytes = fs::read(path)
                .with_context(|| format!("could not read font '{}'", path.display()))?;
            Font::from_bytes(bytes, FontSettings::default())
                .map_err(|error| anyhow!("could not parse font '{}': {error}", path.display()))?;
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
            serde_json::from_str::<serde_json::Value>(&text)
                .with_context(|| format!("could not parse LDtk project '{}'", path.display()))?;
        }
        Some(extension)
            if extension.eq_ignore_ascii_case("ron") && is_beginner_data_file(path, asset_root) =>
        {
            game_kit::data::validate_beginner_game_file(path).with_context(|| {
                format!("could not validate beginner data file '{}'", path.display())
            })?;
        }
        Some(extension)
            if extension.eq_ignore_ascii_case("ron")
                && is_animation_metadata_file(path, asset_root) =>
        {
            game_kit::assets::validate_animation_sheet_file(path).with_context(|| {
                format!("could not validate animation metadata '{}'", path.display())
            })?;
        }
        _ if options.deny_unknown => bail!("{}", unknown_asset_message(path)),
        _ => eprintln!(
            "warning: {}",
            unknown_asset_message(path)
                .lines()
                .next()
                .unwrap_or("unknown asset file")
        ),
    }
    Ok(())
}

fn is_ignored_asset_metadata(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(OsStr::to_str),
        Some(".gitignore" | ".DS_Store")
    )
}

fn is_beginner_data_file(path: &Path, asset_root: &Path) -> bool {
    asset_relative_path(path, asset_root) == Path::new("game.ron")
}

fn is_animation_metadata_file(path: &Path, asset_root: &Path) -> bool {
    asset_relative_path(path, asset_root)
        .parent()
        .and_then(Path::file_name)
        .and_then(OsStr::to_str)
        == Some("animations")
}

fn asset_relative_path<'a>(path: &'a Path, asset_root: &Path) -> &'a Path {
    path.strip_prefix(asset_root).unwrap_or(path)
}

fn unknown_asset_message(path: &Path) -> String {
    let mut message = format!("unknown asset file '{}'", path.display());
    if let Some(suggestion) = suggested_asset_extension(path) {
        message.push_str(&format!("\n\nDid you mean '{suggestion}'?"));
    }
    message.push_str(
        "\nSupported beginner asset types: .png, .ttf, .wav, .ogg, .mp3, .txt, .tmx, .ldtk, assets/game.ron, and animation .ron files under assets/animations/.\nMove notes/source files outside assets/ or add an explicit ignore rule when ignore support exists.",
    );
    message
}

fn suggested_asset_extension(path: &Path) -> Option<&'static str> {
    match path
        .extension()
        .and_then(OsStr::to_str)
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("pgn" | "pnj") => Some(".png"),
        Some("wave") => Some(".wav"),
        Some("ogv") => Some(".ogg"),
        Some("mpeg" | "mpga") => Some(".mp3"),
        _ => None,
    }
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

#[cfg(test)]
mod tests {
    use super::{AssetCheckOptions, validate_asset_file, validate_assets_dir, validate_text_map};

    fn temp_assets(name: &str) -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!("game-cli-{name}-{}", std::process::id()));
        if root.exists() {
            std::fs::remove_dir_all(&root).unwrap();
        }
        let assets = root.join("assets");
        std::fs::create_dir_all(&assets).unwrap();
        assets
    }

    #[test]
    fn text_map_validation_names_the_ragged_row() {
        let path = std::env::temp_dir().join(format!(
            "game-cli-map-validation-{}.txt",
            std::process::id()
        ));
        std::fs::write(&path, "####\n##\n").unwrap();
        let error = validate_text_map(&path).unwrap_err().to_string();
        assert!(error.contains("row 2 has width 2, expected 4"));
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn pgn_texture_typo_suggests_png() {
        let assets = temp_assets("pgn-typo");
        let textures = assets.join("textures");
        std::fs::create_dir_all(&textures).unwrap();
        let path = textures.join("player.pgn");
        std::fs::write(&path, b"not actually a png").unwrap();

        let error = validate_asset_file(&path, &assets, AssetCheckOptions::deny_unknown())
            .unwrap_err()
            .to_string();

        assert!(error.contains("unknown asset file"));
        assert!(error.contains("Did you mean '.png'?"));
        assert!(error.contains("Supported beginner asset types"));

        std::fs::remove_dir_all(assets.parent().unwrap()).unwrap();
    }

    #[test]
    fn markdown_note_in_assets_is_unknown() {
        let assets = temp_assets("markdown-note");
        let path = assets.join("readme.md");
        std::fs::write(&path, "# notes\n").unwrap();

        let error = validate_asset_file(&path, &assets, AssetCheckOptions::deny_unknown())
            .unwrap_err()
            .to_string();

        assert!(error.contains("unknown asset file"));
        assert!(error.contains("readme.md"));

        std::fs::remove_dir_all(assets.parent().unwrap()).unwrap();
    }

    #[test]
    fn gitignore_inside_assets_is_allowed() {
        let assets = temp_assets("gitignore");
        let path = assets.join(".gitignore");
        std::fs::write(&path, "*\n!.gitignore\n").unwrap();

        validate_asset_file(&path, &assets, AssetCheckOptions::deny_unknown()).unwrap();
        validate_assets_dir(&assets, false).unwrap();

        std::fs::remove_dir_all(assets.parent().unwrap()).unwrap();
    }

    #[test]
    fn template_assets_pass_validation() {
        let workspace = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");

        validate_assets_dir(&workspace.join("templates/simple-demo/assets"), false).unwrap();
        validate_assets_dir(&workspace.join("templates/data-driven-demo/assets"), false).unwrap();
    }

    #[test]
    fn animation_metadata_ron_is_validated_as_animation_metadata() {
        let root = std::env::temp_dir().join(format!(
            "game-cli-animation-validation-{}",
            std::process::id()
        ));
        let animations = root.join("assets/animations");
        std::fs::create_dir_all(&animations).unwrap();
        let path = animations.join("player.ron");
        std::fs::write(
            &path,
            r#"(
    texture: "textures/player_sheet.png",
    columns: 4,
    rows: 1,
    clips: {"idle": (frames: [0])},
)"#,
        )
        .unwrap();

        let assets = root.join("assets");
        let error = format!(
            "{:?}",
            validate_asset_file(&path, &assets, AssetCheckOptions::deny_unknown()).unwrap_err()
        );

        assert!(error.contains("could not validate animation metadata"));
        assert!(error.contains("references missing texture 'textures/player_sheet.png'"));

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn arbitrary_ron_in_assets_is_unknown() {
        let assets = temp_assets("arbitrary-ron");
        let path = assets.join("foo.ron");
        std::fs::write(&path, "()").unwrap();

        let error = validate_asset_file(&path, &assets, AssetCheckOptions::deny_unknown())
            .unwrap_err()
            .to_string();

        assert!(error.contains("unknown asset file"));
        assert!(error.contains("foo.ron"));

        std::fs::remove_dir_all(assets.parent().unwrap()).unwrap();
    }
}

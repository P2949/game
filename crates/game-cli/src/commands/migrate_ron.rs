use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

pub(crate) fn migrate_ron_command(args: impl Iterator<Item = String>) -> Result<()> {
    let options = parse_migrate_ron_options(args)?;
    let current = std::env::current_dir().context("failed to resolve current directory")?;
    let input = absolutize_from(&current, &options.input);
    let output = absolutize_from(&current, &options.output);
    let asset_root = options
        .assets
        .as_deref()
        .map(|assets| absolutize_from(&current, assets))
        .unwrap_or_else(|| default_asset_root(&input, &output));

    let source = std::fs::read_to_string(&input)
        .with_context(|| format!("failed to read legacy RON file '{}'", input.display()))?;
    let migration =
        game_kit::data::migrate_legacy_ron_source_to_toml(&source, &input.display().to_string())?;

    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create output directory '{}'", parent.display()))?;
    }
    std::fs::write(&output, migration.toml)
        .with_context(|| format!("failed to write '{}'", output.display()))?;
    game_kit::data::validate_authoring_file_with_asset_root(&output, &asset_root)?;

    println!("wrote {}", output.display());
    for note in migration.notes {
        println!("note: {note}");
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MigrateRonOptions {
    input: PathBuf,
    output: PathBuf,
    assets: Option<PathBuf>,
}

fn parse_migrate_ron_options(args: impl Iterator<Item = String>) -> Result<MigrateRonOptions> {
    let mut input = None;
    let mut output = None;
    let mut assets = None;
    let mut args = args.peekable();
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--out" => output = Some(next_path(&mut args, "--out")?),
            "--assets" => assets = Some(next_path(&mut args, "--assets")?),
            value if value.starts_with("--") => bail!("unexpected migrate-ron argument '{value}'"),
            value => {
                if input.replace(PathBuf::from(value)).is_some() {
                    bail!("migrate-ron accepts one legacy RON input path");
                }
            }
        }
    }

    Ok(MigrateRonOptions {
        input: input.ok_or_else(|| {
            anyhow::anyhow!("usage: game-dev migrate-ron assets/game.ron --out game.toml")
        })?,
        output: output.ok_or_else(|| anyhow::anyhow!("migrate-ron needs --out game.toml"))?,
        assets,
    })
}

fn next_path(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<PathBuf> {
    args.next()
        .map(PathBuf::from)
        .ok_or_else(|| anyhow::anyhow!("{flag} needs a path"))
}

fn default_asset_root(input: &Path, output: &Path) -> PathBuf {
    input
        .parent()
        .filter(|parent| parent.file_name().and_then(|name| name.to_str()) == Some("assets"))
        .map(Path::to_path_buf)
        .unwrap_or_else(|| {
            output
                .parent()
                .map(|parent| parent.join("assets"))
                .unwrap_or_else(|| PathBuf::from("assets"))
        })
}

fn absolutize_from(base: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    }
}

#[cfg(test)]
mod tests {
    use super::{MigrateRonOptions, migrate_ron_command, parse_migrate_ron_options};
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn parse_migrate_ron_requires_input_and_output() {
        let options = parse_migrate_ron_options(
            [
                "assets/game.ron",
                "--out",
                "game.toml",
                "--assets",
                "assets",
            ]
            .into_iter()
            .map(str::to_owned),
        )
        .unwrap();

        assert_eq!(
            options,
            MigrateRonOptions {
                input: PathBuf::from("assets/game.ron"),
                output: PathBuf::from("game.toml"),
                assets: Some(PathBuf::from("assets")),
            }
        );
    }

    #[test]
    fn migrate_ron_command_writes_valid_game_toml() {
        let project = temp_project("migrate-ron");
        let assets = project.join("assets");
        fs::create_dir_all(assets.join("maps")).unwrap();
        fs::write(assets.join("maps/level_1.txt"), "#####\n#P..#\n#####\n").unwrap();
        let input = assets.join("game.ron");
        fs::write(
            &input,
            r#"
(
    version: 1,
    assets: (textures: ["player", "floor", "wall"]),
    controls: TopDown,
    prefabs: [
        Player((name: "player", sprite: "player")),
    ],
    maps: [
        TextMap((
            name: "level_1",
            path: "maps/level_1.txt",
            theme: ("floor", "wall"),
            legend: {'P': "player"},
            start: true,
        )),
    ],
    rules: [TopDownControls],
)
"#,
        )
        .unwrap();
        let output = project.join("game.toml");

        migrate_ron_command(
            [
                input.to_string_lossy().into_owned(),
                "--out".to_owned(),
                output.to_string_lossy().into_owned(),
            ]
            .into_iter(),
        )
        .unwrap();

        let migrated = fs::read_to_string(output).unwrap();
        assert!(migrated.contains("version = 2"));
        assert!(migrated.contains("kind = \"player\""));
        assert!(migrated.contains("[rules]"));
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

use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, anyhow, bail};

use crate::starter_assets;

pub(crate) const RELEASE_GAME_STARTER_DEPENDENCY: &str =
    r#"{ git = "https://github.com/P2949/game", tag = "v0.2.0", package = "game-starter" }"#;

pub(crate) struct TemplateFile {
    path: &'static str,
    contents: &'static str,
}

const SIMPLE_TEMPLATE: &[TemplateFile] = &[
    TemplateFile {
        path: "Cargo.toml",
        contents: include_str!("../../../templates/simple-demo/Cargo.toml"),
    },
    TemplateFile {
        path: "README.md",
        contents: include_str!("../../../templates/simple-demo/README.md"),
    },
    TemplateFile {
        path: "build.rs",
        contents: include_str!("../../../templates/simple-demo/build.rs"),
    },
    TemplateFile {
        path: "src/main.rs",
        contents: include_str!("../../../templates/simple-demo/src/main.rs"),
    },
    TemplateFile {
        path: "assets/maps/level_1.txt",
        contents: include_str!("../../../templates/simple-demo/assets/maps/level_1.txt"),
    },
];

const NO_RUST_TEMPLATE: &[TemplateFile] = &[
    TemplateFile {
        path: "game.toml",
        contents: include_str!("../../../templates/no-rust-demo/game.toml"),
    },
    TemplateFile {
        path: "README.txt",
        contents: include_str!("../../../templates/no-rust-demo/README.txt"),
    },
    TemplateFile {
        path: "assets/maps/level-1.txt",
        contents: include_str!("../../../templates/no-rust-demo/assets/maps/level-1.txt"),
    },
];

const DATA_DRIVEN_TEMPLATE: &[TemplateFile] = &[
    TemplateFile {
        path: "Cargo.toml",
        contents: include_str!("../../../templates/data-driven-demo/Cargo.toml"),
    },
    TemplateFile {
        path: "README.md",
        contents: include_str!("../../../templates/data-driven-demo/README.md"),
    },
    TemplateFile {
        path: "build.rs",
        contents: include_str!("../../../templates/data-driven-demo/build.rs"),
    },
    TemplateFile {
        path: "src/main.rs",
        contents: include_str!("../../../templates/data-driven-demo/src/main.rs"),
    },
    TemplateFile {
        path: "assets/game.ron",
        contents: include_str!("../../../templates/data-driven-demo/assets/game.ron"),
    },
    TemplateFile {
        path: "assets/maps/level_1.txt",
        contents: include_str!("../../../templates/data-driven-demo/assets/maps/level_1.txt"),
    },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DemoTemplate {
    NoRust,
    Simple,
    DataDriven,
}

impl DemoTemplate {
    fn parse(value: &str) -> Result<Self> {
        match value {
            "no-rust" => Ok(Self::NoRust),
            "simple" | "rust-simple" => Ok(Self::Simple),
            "data-driven" | "data-driven-legacy" => Ok(Self::DataDriven),
            other => bail!(
                "unknown template '{other}'; expected no-rust, simple, rust-simple, data-driven, or data-driven-legacy"
            ),
        }
    }

    fn files(self) -> &'static [TemplateFile] {
        match self {
            Self::NoRust => NO_RUST_TEMPLATE,
            Self::Simple => SIMPLE_TEMPLATE,
            Self::DataDriven => DATA_DRIVEN_TEMPLATE,
        }
    }

    fn is_data_driven(self) -> bool {
        matches!(self, Self::DataDriven)
    }

    fn is_no_rust(self) -> bool {
        matches!(self, Self::NoRust)
    }
}

pub(crate) fn parse_template_args(args: impl IntoIterator<Item = String>) -> Result<DemoTemplate> {
    let mut template = DemoTemplate::Simple;
    let mut args = args.into_iter();
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--data-driven" => template = DemoTemplate::DataDriven,
            "--template" => {
                let value = args
                    .next()
                    .ok_or_else(|| {
                        anyhow!(
                            "--template needs no-rust, simple, rust-simple, data-driven, or data-driven-legacy"
                        )
                    })?;
                template = DemoTemplate::parse(&value)?;
            }
            extra => bail!(
                "unexpected template argument '{extra}'; expected --template no-rust|simple|rust-simple|data-driven|data-driven-legacy"
            ),
        }
    }
    Ok(template)
}

pub(crate) fn new_project(
    destination: &Path,
    template: DemoTemplate,
    dependency: &str,
) -> Result<()> {
    if destination.exists() {
        bail!("destination '{}' already exists", destination.display());
    }

    let crate_name = crate_name_from_destination(destination)?;
    let title = title_from_crate_name(&crate_name);
    let mut values = HashMap::new();
    values.insert("crate_name", crate_name.as_str());
    values.insert("game_starter_dependency", dependency);
    values.insert("title", title.as_str());

    write_embedded_template(template.files(), destination, &values)?;
    if template.is_no_rust() {
        starter_assets::write_starter_assets(&destination.join("assets"))?;
    }

    println!("created demo at {}", destination.display());
    if template.is_no_rust() {
        println!("setup lives in game.toml; no Rust project files were generated");
    } else if template.is_data_driven() {
        println!("setup lives in assets/game.ron; src/main.rs is ready for optional custom rules");
    } else {
        println!("setup lives in src/main.rs with beginner Rust builder chains");
    }
    println!("next steps:");
    println!("    cd {}", destination.display());
    println!("    game-dev doctor");
    println!("    game-dev check");
    if template.is_no_rust() {
        println!("    game-dev preview");
    } else {
        println!("    game-dev run");
    }
    Ok(())
}

fn write_embedded_template(
    files: &[TemplateFile],
    destination: &Path,
    values: &HashMap<&str, &str>,
) -> Result<()> {
    fs::create_dir_all(destination)
        .with_context(|| format!("failed to create '{}'", destination.display()))?;
    for file in files {
        let mut contents = file.contents.to_string();
        for (key, value) in values {
            contents = contents.replace(&format!("{{{{{key}}}}}"), value);
        }
        let path = destination.join(file.path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create '{}'", parent.display()))?;
        }
        fs::write(&path, contents)
            .with_context(|| format!("failed to write '{}'", path.display()))?;
    }
    Ok(())
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
        } else if !last_was_dash && !name.is_empty() {
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

#[cfg(test)]
mod tests {
    use super::{DemoTemplate, crate_name_from_destination, new_project, parse_template_args};
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn crate_name_is_sanitized_from_destination() {
        assert_eq!(
            crate_name_from_destination(std::path::Path::new("My First Game")).unwrap(),
            "my-first-game"
        );
    }

    #[test]
    fn no_rust_template_argument_is_supported() {
        assert_eq!(
            parse_template_args(["--template".to_owned(), "no-rust".to_owned()]).unwrap(),
            DemoTemplate::NoRust
        );
        assert_eq!(
            parse_template_args(["--template".to_owned(), "rust-simple".to_owned()]).unwrap(),
            DemoTemplate::Simple
        );
        assert_eq!(
            parse_template_args(["--template".to_owned(), "data-driven-legacy".to_owned()])
                .unwrap(),
            DemoTemplate::DataDriven
        );
    }

    #[test]
    fn no_rust_template_generates_no_rust_project_files() {
        let destination = temp_project("no-rust-template");
        new_project(&destination, DemoTemplate::NoRust, "{ path = \"unused\" }").unwrap();

        for forbidden in [
            "Cargo.toml",
            "build.rs",
            "src/main.rs",
            "cargo-generate.toml",
        ] {
            assert!(
                !destination.join(forbidden).exists(),
                "no-rust template should not generate {forbidden}"
            );
        }
        for required in [
            "game.toml",
            "README.txt",
            "assets/maps/level-1.txt",
            "assets/textures/player.png",
            "assets/textures/slime.png",
            "assets/textures/coin.png",
            "assets/textures/floor.png",
            "assets/textures/wall.png",
            "assets/sounds/coin.wav",
            "assets/fonts/DejaVuSans.ttf",
        ] {
            assert!(
                destination.join(required).is_file(),
                "no-rust template should generate {required}"
            );
        }

        let game_toml = fs::read_to_string(destination.join("game.toml")).unwrap();
        assert!(game_toml.contains("kind = \"player\""));
        assert!(game_toml.contains("\"top-down-controls\""));
        assert!(!game_toml.contains("Some("));
        assert!(!game_toml.contains("Player(("));
        game_kit::data::validate_authoring_file(destination.join("game.toml")).unwrap();
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
        dir
    }
}

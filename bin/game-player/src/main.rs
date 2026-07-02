use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use game_kit::app::plugin_fn;
use game_runtime::{CommandErrorPolicy, RuntimeConfig};
use toml::Value;

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let command = PlayerCommand::parse(env::args().skip(1))?;
    let options = match command {
        PlayerCommand::Help => {
            print_usage();
            return Ok(());
        }
        PlayerCommand::Run(options) => options,
    };
    options.apply_environment();

    let metadata = GameMetadata::from_file(&options.game_file)?;
    let config = metadata
        .apply_to(RuntimeConfig::default())?
        .command_error_policy(CommandErrorPolicy::StoreResource);

    let game_file = options.game_file.clone();
    let asset_root = options.asset_root.clone();
    game_runtime::run(
        config,
        plugin_fn(move |game| {
            game.load_authoring_file_with_asset_root(&game_file, &asset_root)?;
            Ok(())
        }),
    )
}

enum PlayerCommand {
    Help,
    Run(PlayerOptions),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PlayerOptions {
    project_root: PathBuf,
    game_file: PathBuf,
    asset_root: PathBuf,
    smoke_frames: Option<u64>,
}

impl PlayerOptions {
    fn apply_environment(&self) {
        // SAFETY: this runs during single-threaded process startup, before the
        // runtime creates platform/audio/rendering threads.
        unsafe {
            env::set_var("GAME_ASSET_DIR", &self.asset_root);
            if let Some(frames) = self.smoke_frames {
                env::set_var("GAME_SMOKE_FRAMES", frames.to_string());
            }
        }
    }
}

#[derive(Default)]
struct RawOptions {
    project_root: Option<PathBuf>,
    game_file: Option<PathBuf>,
    asset_root: Option<PathBuf>,
    smoke_frames: Option<u64>,
}

impl PlayerCommand {
    fn parse(args: impl IntoIterator<Item = String>) -> Result<Self> {
        let mut raw = RawOptions::default();
        let mut args = args.into_iter();
        while let Some(argument) = args.next() {
            match argument.as_str() {
                "-h" | "--help" => return Ok(Self::Help),
                "--project" => {
                    raw.project_root = Some(next_path(&mut args, "--project")?);
                }
                "--file" => {
                    raw.game_file = Some(next_path(&mut args, "--file")?);
                }
                "--assets" => {
                    raw.asset_root = Some(next_path(&mut args, "--assets")?);
                }
                "--smoke-frames" => {
                    let value = args
                        .next()
                        .ok_or_else(|| anyhow::anyhow!("--smoke-frames needs a frame count"))?;
                    raw.smoke_frames = Some(value.parse::<u64>().with_context(|| {
                        format!("--smoke-frames value '{value}' is not a non-negative integer")
                    })?);
                }
                extra => bail!("unexpected argument '{extra}'. Use --help for usage."),
            }
        }
        Ok(Self::Run(raw.into_options()?))
    }
}

impl RawOptions {
    fn into_options(self) -> Result<PlayerOptions> {
        let project_root = self
            .project_root
            .or_else(|| env::var_os("GAME_PROJECT_DIR").map(PathBuf::from))
            .map(|path| absolutize_from_current(&path))
            .transpose()?
            .unwrap_or(env::current_dir().context("failed to resolve current directory")?);
        let game_file = resolve_from(
            &project_root,
            self.game_file
                .or_else(|| env::var_os("GAME_FILE").map(PathBuf::from))
                .unwrap_or_else(|| PathBuf::from("game.toml")),
        );
        let asset_root = resolve_from(
            &project_root,
            self.asset_root
                .or_else(|| env::var_os("GAME_ASSET_DIR").map(PathBuf::from))
                .unwrap_or_else(|| PathBuf::from("assets")),
        );
        let smoke_frames = match self.smoke_frames {
            Some(frames) => Some(frames),
            None => env::var("GAME_SMOKE_FRAMES")
                .ok()
                .map(|value| {
                    value.trim().parse::<u64>().with_context(|| {
                        format!(
                            "GAME_SMOKE_FRAMES value '{}' is not a non-negative integer",
                            value
                        )
                    })
                })
                .transpose()?,
        };

        Ok(PlayerOptions {
            project_root,
            game_file,
            asset_root,
            smoke_frames,
        })
    }
}

fn next_path(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<PathBuf> {
    args.next()
        .map(PathBuf::from)
        .ok_or_else(|| anyhow::anyhow!("{flag} needs a path"))
}

fn absolutize_from_current(path: &Path) -> Result<PathBuf> {
    Ok(resolve_from(
        &env::current_dir().context("failed to resolve current directory")?,
        path,
    ))
}

fn resolve_from(base: &Path, path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    }
}

#[derive(Default)]
struct GameMetadata {
    title: Option<String>,
    window_width: Option<u32>,
    window_height: Option<u32>,
    sim_hz: Option<f64>,
}

impl GameMetadata {
    fn from_file(path: &Path) -> Result<Self> {
        let source = fs::read_to_string(path)
            .with_context(|| format!("could not read game config '{}'", path.display()))?;
        let value = toml::from_str::<Value>(&source)
            .with_context(|| format!("could not parse game config '{}'", path.display()))?;
        let Some(game) = value.get("game").and_then(Value::as_table) else {
            return Ok(Self::default());
        };

        Ok(Self {
            title: string_field(game, "title")?,
            window_width: u32_field(game, "window_width")?,
            window_height: u32_field(game, "window_height")?,
            sim_hz: f64_field(game, "sim_hz")?,
        })
    }

    fn apply_to(self, mut config: RuntimeConfig) -> Result<RuntimeConfig> {
        if let Some(title) = self.title {
            config = config.title(title);
        }
        match (self.window_width, self.window_height) {
            (Some(width), Some(height)) => {
                config = config.window_size(width, height);
            }
            (None, None) => {}
            _ => bail!("game config [game] must set both window_width and window_height"),
        }
        if let Some(sim_hz) = self.sim_hz {
            config = config.sim_hz(sim_hz);
        }
        Ok(config)
    }
}

fn string_field(
    table: &toml::map::Map<String, Value>,
    key: &'static str,
) -> Result<Option<String>> {
    table
        .get(key)
        .map(|value| {
            value
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| anyhow::anyhow!("game config [game].{key} must be a string"))
        })
        .transpose()
}

fn u32_field(table: &toml::map::Map<String, Value>, key: &'static str) -> Result<Option<u32>> {
    table
        .get(key)
        .map(|value| {
            let raw = value
                .as_integer()
                .ok_or_else(|| anyhow::anyhow!("game config [game].{key} must be an integer"))?;
            u32::try_from(raw)
                .with_context(|| format!("game config [game].{key} must be non-negative"))
        })
        .transpose()
}

fn f64_field(table: &toml::map::Map<String, Value>, key: &'static str) -> Result<Option<f64>> {
    table
        .get(key)
        .map(|value| {
            value
                .as_float()
                .or_else(|| value.as_integer().map(|integer| integer as f64))
                .ok_or_else(|| anyhow::anyhow!("game config [game].{key} must be a number"))
        })
        .transpose()
}

fn print_usage() {
    println!(
        "usage: game-player [--project <dir>] [--file <path>] [--assets <dir>] [--smoke-frames <count>]\n\nLoads a no-Rust game.toml package through the prebuilt player."
    );
}

#[cfg(test)]
mod tests {
    use super::{PlayerCommand, PlayerOptions};
    use std::path::PathBuf;

    #[test]
    fn parses_explicit_player_paths() {
        let command = PlayerCommand::parse([
            "--project".to_owned(),
            "/tmp/my-game".to_owned(),
            "--file".to_owned(),
            "config/game.toml".to_owned(),
            "--assets".to_owned(),
            "art".to_owned(),
            "--smoke-frames".to_owned(),
            "1".to_owned(),
        ])
        .unwrap();
        let PlayerCommand::Run(options) = command else {
            panic!("expected run command");
        };
        assert_eq!(
            options,
            PlayerOptions {
                project_root: PathBuf::from("/tmp/my-game"),
                game_file: PathBuf::from("/tmp/my-game/config/game.toml"),
                asset_root: PathBuf::from("/tmp/my-game/art"),
                smoke_frames: Some(1),
            }
        );
    }

    #[test]
    fn parses_help() {
        assert!(matches!(
            PlayerCommand::parse(["--help".to_owned()]).unwrap(),
            PlayerCommand::Help
        ));
    }
}

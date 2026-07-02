use std::collections::BTreeMap;
use std::env;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::thread;
use std::time::{Duration, SystemTime};

use anyhow::{Context, Result, anyhow, bail};
use walkdir::WalkDir;

use crate::paths::{executable_name, workspace_root};
use crate::project::{NoRustPathOverrides, resolve_no_rust_project_paths_with_env};

pub(crate) fn preview_command(args: impl Iterator<Item = String>) -> Result<()> {
    let options = parse_preview_options(args)?;
    if options.watch {
        preview_watch(options)
    } else {
        let status = start_preview(&options)?
            .wait()
            .context("failed to wait for game-player")?;
        if !status.success() {
            bail!("game-player exited with status {status}");
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PreviewOptions {
    paths: crate::project::NoRustProjectPaths,
    player: PlayerCommand,
    smoke_frames: Option<u64>,
    watch: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum PlayerCommand {
    Executable(PathBuf),
    CargoFallback { workspace: PathBuf },
}

fn parse_preview_options(args: impl Iterator<Item = String>) -> Result<PreviewOptions> {
    let current = env::current_dir().context("failed to resolve current project directory")?;
    let mut overrides = NoRustPathOverrides::default();
    let mut explicit_player = None;
    let mut smoke_frames = None;
    let mut watch = false;
    let mut args = args.peekable();

    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--project" => overrides.project = Some(next_path(&mut args, "--project")?),
            "--file" => overrides.file = Some(next_path(&mut args, "--file")?),
            "--assets" => overrides.assets = Some(next_path(&mut args, "--assets")?),
            "--player" => explicit_player = Some(next_path(&mut args, "--player")?),
            "--smoke-frames" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow!("--smoke-frames needs a frame count"))?;
                smoke_frames = Some(value.parse::<u64>().with_context(|| {
                    format!("--smoke-frames value '{value}' is not a non-negative integer")
                })?);
            }
            "--watch" => watch = true,
            extra => bail!("unexpected preview argument '{extra}'"),
        }
    }

    let paths = resolve_no_rust_project_paths_with_env(&current, &overrides);
    let player = resolve_player_command(explicit_player.as_deref(), &current)?;
    Ok(PreviewOptions {
        paths,
        player,
        smoke_frames,
        watch,
    })
}

fn next_path(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<PathBuf> {
    args.next()
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("{flag} needs a path"))
}

fn resolve_player_command(explicit: Option<&Path>, current: &Path) -> Result<PlayerCommand> {
    if let Some(path) = explicit {
        return Ok(PlayerCommand::Executable(resolve_from(current, path)));
    }

    if let Ok(current_exe) = env::current_exe()
        && let Some(parent) = current_exe.parent()
    {
        let sibling = parent.join(executable_name("game-player"));
        if sibling.is_file() {
            return Ok(PlayerCommand::Executable(sibling));
        }
    }

    if let Some(path) = env::var_os("GAME_PLAYER") {
        return Ok(PlayerCommand::Executable(PathBuf::from(path)));
    }

    let workspace = workspace_root()?;
    if workspace.join("bin/game-player/Cargo.toml").is_file() {
        return Ok(PlayerCommand::CargoFallback { workspace });
    }

    bail!("could not find game-player; pass --player <path> or set GAME_PLAYER")
}

fn start_preview(options: &PreviewOptions) -> Result<Child> {
    let mut command = match &options.player {
        PlayerCommand::Executable(path) => {
            let mut command = Command::new(path);
            add_player_runtime_library_path(&mut command, path);
            command
        }
        PlayerCommand::CargoFallback { workspace } => {
            let mut command = Command::new("cargo");
            command
                .args(["run", "-p", "game-player", "--"])
                .current_dir(workspace);
            command
        }
    };
    command
        .arg("--project")
        .arg(&options.paths.root)
        .arg("--file")
        .arg(&options.paths.game_file)
        .arg("--assets")
        .arg(&options.paths.asset_dir);
    if let Some(frames) = options.smoke_frames {
        command.arg("--smoke-frames").arg(frames.to_string());
    }
    command.spawn().context("failed to start game-player")
}

fn add_player_runtime_library_path(command: &mut Command, player: &Path) {
    let Some(player_dir) = player.parent() else {
        return;
    };
    prepend_env_path(command, "LD_LIBRARY_PATH", player_dir);
    prepend_env_path(command, "DYLD_LIBRARY_PATH", player_dir);
    prepend_env_path(command, "PATH", player_dir);
}

fn prepend_env_path(command: &mut Command, key: &str, path: &Path) {
    let mut paths = vec![path.to_path_buf()];
    if let Some(existing) = env::var_os(key) {
        paths.extend(env::split_paths(&existing));
    }
    if let Ok(joined) = env::join_paths(paths) {
        command.env(key, joined);
    }
}

fn preview_watch(options: PreviewOptions) -> Result<()> {
    let mut snapshot = watch_snapshot(&options.paths)?;
    let mut child = start_preview(&options)?;
    loop {
        thread::sleep(Duration::from_millis(250));
        if let Some(status) = child.try_wait().context("failed to poll game-player")?
            && !status.success()
        {
            bail!("game-player exited with status {status}");
        }
        let next = watch_snapshot(&options.paths)?;
        if let Some(changed) = first_changed_path(&snapshot, &next) {
            println!("changed {}; restarting preview", changed.display());
            stop_child(&mut child)?;
            thread::sleep(Duration::from_millis(150));
            child = start_preview(&options)?;
            snapshot = next;
        }
    }
}

fn stop_child(child: &mut Child) -> Result<()> {
    match child.try_wait().context("failed to poll game-player")? {
        Some(_) => Ok(()),
        None => {
            child.kill().context("failed to stop game-player")?;
            child
                .wait()
                .context("failed to wait for game-player stop")?;
            Ok(())
        }
    }
}

fn watch_snapshot(
    paths: &crate::project::NoRustProjectPaths,
) -> Result<BTreeMap<PathBuf, Option<SystemTime>>> {
    let mut snapshot = BTreeMap::new();
    snapshot.insert(paths.game_file.clone(), modified_time(&paths.game_file));
    if paths.asset_dir.is_dir() {
        for entry in WalkDir::new(&paths.asset_dir) {
            let entry =
                entry.with_context(|| format!("failed to walk '{}'", paths.asset_dir.display()))?;
            if entry.file_type().is_file() {
                snapshot.insert(entry.path().to_path_buf(), modified_time(entry.path()));
            }
        }
    }
    Ok(snapshot)
}

fn modified_time(path: &Path) -> Option<SystemTime> {
    path.metadata()
        .and_then(|metadata| metadata.modified())
        .ok()
}

fn first_changed_path(
    before: &BTreeMap<PathBuf, Option<SystemTime>>,
    after: &BTreeMap<PathBuf, Option<SystemTime>>,
) -> Option<PathBuf> {
    for path in before.keys().chain(after.keys()) {
        if before.get(path) != after.get(path) {
            return Some(path.clone());
        }
    }
    None
}

fn resolve_from(base: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        PlayerCommand, add_player_runtime_library_path, first_changed_path, resolve_player_command,
        watch_snapshot,
    };
    use crate::project::NoRustProjectPaths;
    use std::env;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::time::Duration;

    #[test]
    fn explicit_player_path_wins() {
        assert_eq!(
            resolve_player_command(
                Some(Path::new("bin/game-player")),
                Path::new("/tmp/project")
            )
            .unwrap(),
            PlayerCommand::Executable(PathBuf::from("/tmp/project/bin/game-player"))
        );
    }

    #[test]
    fn watch_snapshot_tracks_game_file_and_assets() {
        let root = temp_dir("preview-watch");
        let assets = root.join("assets");
        fs::create_dir_all(assets.join("maps")).unwrap();
        fs::write(root.join("game.toml"), "version = 2\n").unwrap();
        fs::write(assets.join("maps/level-1.txt"), "###\n#P#\n###\n").unwrap();
        let paths = NoRustProjectPaths {
            root: root.clone(),
            game_file: root.join("game.toml"),
            asset_dir: assets.clone(),
        };

        let before = watch_snapshot(&paths).unwrap();
        assert!(before.contains_key(&paths.game_file));
        assert!(before.contains_key(&assets.join("maps/level-1.txt")));

        std::thread::sleep(Duration::from_millis(20));
        fs::write(
            &paths.game_file,
            "version = 2\n[game]\ntitle = \"Edited\"\n",
        )
        .unwrap();
        let after_game_edit = watch_snapshot(&paths).unwrap();
        assert_eq!(
            first_changed_path(&before, &after_game_edit).as_deref(),
            Some(paths.game_file.as_path())
        );

        fs::write(assets.join("maps/level-2.txt"), "###\n#P#\n###\n").unwrap();
        let after_asset_edit = watch_snapshot(&paths).unwrap();
        assert_eq!(
            first_changed_path(&after_game_edit, &after_asset_edit).as_deref(),
            Some(assets.join("maps/level-2.txt").as_path())
        );
    }

    #[test]
    fn executable_preview_searches_player_directory_for_runtime_libraries() {
        let root = temp_dir("preview-runtime-libraries");
        let player = root.join(crate::paths::executable_name("game-player"));
        let mut command = Command::new(&player);

        add_player_runtime_library_path(&mut command, &player);

        assert!(command_env_path_contains(
            &command,
            "LD_LIBRARY_PATH",
            &root
        ));
        assert!(command_env_path_contains(
            &command,
            "DYLD_LIBRARY_PATH",
            &root
        ));
        assert!(command_env_path_contains(&command, "PATH", &root));
    }

    fn command_env_path_contains(command: &Command, key: &str, expected: &Path) -> bool {
        command
            .get_envs()
            .find(|(name, _)| *name == key)
            .and_then(|(_, value)| value)
            .map(|value| env::split_paths(value).any(|path| path == expected))
            .unwrap_or(false)
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

use super::super::{
    AuthoringFormat, BeginnerPrefabFile, read_authoring_game_file, validate_authoring_file,
    validate_authoring_file_with_asset_root,
};
use crate::app::{GameApp, GamePlugin};
use crate::beginner::actors::Enemy;
use crate::harness::GameTestHarness;
use anyhow::Result;
use game_combat::Health;
use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn authoring_format_detects_toml_and_legacy_ron() {
    assert_eq!(
        AuthoringFormat::from_path(Path::new("game.toml")).unwrap(),
        AuthoringFormat::Toml
    );
    assert_eq!(
        AuthoringFormat::from_path(Path::new("assets/game.ron")).unwrap(),
        AuthoringFormat::RonLegacy
    );
}

#[test]
fn authoring_format_rejects_missing_or_unknown_extensions_with_primary_help() {
    let missing = AuthoringFormat::from_path(Path::new("game"))
        .unwrap_err()
        .to_string();
    assert!(missing.contains("Use `game.toml` for primary no-Rust authoring"));
    assert!(missing.contains("RON is legacy"));

    let unknown = AuthoringFormat::from_path(Path::new("game.txt"))
        .unwrap_err()
        .to_string();
    assert!(unknown.contains("unsupported extension '.txt'"));
    assert!(unknown.contains("Use `game.toml` for primary no-Rust authoring"));
    assert!(unknown.contains("game-dev migrate-ron"));
}

#[test]
fn validate_authoring_file_reads_root_game_toml_with_sibling_assets() {
    let dir = temp_toml_project("toml-authoring");
    let assets = dir.join("assets");
    fs::create_dir_all(assets.join("maps")).unwrap();
    fs::write(
        assets.join("maps").join("level-1.txt"),
        "#####\n#P..#\n#####\n",
    )
    .unwrap();

    let game_file = dir.join("game.toml");
    fs::write(
        &game_file,
        r#"
version = 2

[assets]
textures = ["player", "floor", "wall"]

[controls]
preset = "top-down"

[[prefab]]
kind = "player"
name = "player"
sprite = "player"

[[map]]
kind = "text"
name = "level-1"
file = "assets/maps/level-1.txt"
floor = "floor"
wall = "wall"
start = true

[map.legend]
P = "player"

[rules]
enabled = ["top-down-controls"]
"#,
    )
    .unwrap();

    validate_authoring_file(&game_file).unwrap();

    let loaded = read_authoring_game_file(&game_file).unwrap();
    assert_eq!(loaded.context.project_root, dir);
    assert_eq!(loaded.context.asset_root, assets);
    assert_eq!(loaded.context.source_file, game_file);
    assert_eq!(loaded.label, "game.toml");
}

#[test]
fn full_data_driven_demo_toml_stays_valid_and_uses_lower_kebab_strings() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/data-driven-full-demo");
    let game_file = root.join("game.toml");
    let source = fs::read_to_string(&game_file).unwrap();

    for required in [
        "kind = \"player\"",
        "kind = \"projectile\"",
        "kind = \"spawner\"",
        "kind = \"checkpoint\"",
        "preset = \"top-down\"",
        "direction = \"towards-mouse\"",
        "\"top-down-controls\"",
        "\"player-collects-pickups\"",
        "\"win-when-all-enemies-dead\"",
        "action = \"damage-tagged\"",
    ] {
        assert!(
            source.contains(required),
            "full demo game.toml should demonstrate {required:?}"
        );
    }

    for forbidden in [
        "Some(",
        "Player((",
        "Enemy((",
        "Pickup((",
        "Projectile((",
        "Spawner((",
        "Door((",
        "Trigger((",
        "Checkpoint((",
        "TopDownControls",
        "WinWhenAllEnemiesDead",
        "TowardsMouse",
    ] {
        assert!(
            !source.contains(forbidden),
            "primary game.toml must not use RON/Rust-shaped spelling {forbidden:?}"
        );
    }

    validate_authoring_file(&game_file).unwrap();
}

#[test]
fn validate_authoring_file_accepts_explicit_relative_asset_root() {
    let dir = temp_toml_project("toml-explicit-assets");
    let assets = dir.join("custom-assets");
    fs::create_dir_all(assets.join("maps")).unwrap();
    fs::write(
        assets.join("maps").join("level.txt"),
        "#####\n#P..#\n#####\n",
    )
    .unwrap();
    let game_file = dir.join("game.toml");
    fs::write(
        &game_file,
        r#"
version = 2

[assets]
textures = ["player", "floor", "wall"]

[controls]
preset = "top-down"

[[prefab]]
kind = "player"
name = "player"
sprite = "player"

[[map]]
kind = "text"
name = "level"
file = "maps/level.txt"
floor = "floor"
wall = "wall"
start = true

[map.legend]
P = "player"

[rules]
enabled = ["top-down-controls"]
"#,
    )
    .unwrap();

    validate_authoring_file_with_asset_root(&game_file, "custom-assets").unwrap();
}

#[test]
fn projectile_toml_uses_duration_key() {
    let dir = temp_toml_project("toml-projectile-duration");
    let game_file = dir.join("game.toml");
    let source = r#"
version = 2

[assets]
textures = ["bolt"]

[[prefab]]
kind = "projectile"
name = "bolt"
sprite = "bolt"
duration = 0.8
"#;
    fs::write(&game_file, source).unwrap();

    let loaded = read_authoring_game_file(&game_file).unwrap();
    let projectile = loaded
        .file
        .prefabs
        .iter()
        .find_map(|prefab| match prefab {
            BeginnerPrefabFile::Projectile(projectile) => Some(projectile),
            _ => None,
        })
        .expect("expected projectile prefab");
    assert_eq!(projectile.lifetime, 0.8);

    fs::write(
        &game_file,
        source.replace("duration = 0.8", "lifetime = 0.8"),
    )
    .unwrap();
    let error = match read_authoring_game_file(&game_file) {
        Ok(_) => panic!("old projectile timer key should be rejected"),
        Err(error) => error.to_string(),
    };
    assert!(error.contains("use duration"));
}

#[test]
fn f5_reload_reports_primary_game_toml_by_name() {
    let dir = temp_toml_project("toml-reload-label");
    let assets = dir.join("assets");
    fs::create_dir_all(assets.join("maps")).unwrap();
    fs::write(
        assets.join("maps").join("level-a.txt"),
        "#####\n#P..#\n#####\n",
    )
    .unwrap();
    fs::write(
        assets.join("maps").join("level-b.txt"),
        "#####\n#PE.#\n#####\n",
    )
    .unwrap();
    let game_file = dir.join("game.toml");
    fs::write(&game_file, reload_game_toml("level-a.txt", 30)).unwrap();

    let mut game = GameTestHarness::from_plugin(TempTomlPlugin {
        path: game_file.to_string_lossy().into_owned(),
    })
    .unwrap();
    assert_eq!(game.count::<Enemy>(), 0);

    fs::write(&game_file, reload_game_toml("level-b.txt", 77)).unwrap();
    game.tap_action("reload");

    assert_eq!(game.count::<Enemy>(), 1);
    let enemy = game.world().ids_with::<Enemy>()[0];
    let health = game.world().get::<Health>(enemy).unwrap();
    assert_eq!(health.max, 77);
    assert_eq!(health.current, 77);
    game.frame(1.0 / 60.0);
    game.assert_ui_contains("game.toml reload: partial");
    game.assert_ui_contains("last reload: game.toml ok (level-1)");
}

struct TempTomlPlugin {
    path: String,
}

impl GamePlugin for TempTomlPlugin {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        game.load_authoring_file(&self.path)?;
        game.enable_debug_overlay();
        game.on_start(|game| game.spawn_start_map());
        Ok(())
    }
}

fn reload_game_toml(map_file: &str, enemy_health: i32) -> String {
    format!(
        r#"
version = 2

[assets]
textures = ["player", "slime", "floor", "wall"]

[controls]
preset = "top-down"

[[prefab]]
kind = "player"
name = "player"
sprite = "player"

[[prefab]]
kind = "enemy"
name = "slime"
sprite = "slime"
health = {enemy_health}
chase_player = true

[[map]]
kind = "text"
name = "level-1"
file = "assets/maps/{map_file}"
floor = "floor"
wall = "wall"
start = true

[map.legend]
P = "player"
E = "slime"

[rules]
enabled = ["top-down-controls"]
"#
    )
}

fn temp_toml_project(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "game-kit-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

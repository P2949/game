use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;

struct EnvVarGuard {
    key: &'static str,
    previous: Option<OsString>,
}

impl EnvVarGuard {
    fn set(key: &'static str, value: impl AsRef<std::ffi::OsStr>) -> Self {
        let previous = std::env::var_os(key);
        // SAFETY: this integration test is a single-test process, so no other
        // test in this binary observes the temporary environment mutation.
        unsafe {
            std::env::set_var(key, value);
        }
        Self { key, previous }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        // SAFETY: see the matching `set` call above; the mutation is scoped to
        // this single-test process.
        unsafe {
            match &self.previous {
                Some(value) => std::env::set_var(self.key, value),
                None => std::env::remove_var(self.key),
            }
        }
    }
}

#[test]
fn data_game_asset_dir_compatibility_validates_legacy_game_ron() {
    let dir = temp_data_project("game-asset-dir-compat");
    fs::create_dir_all(dir.join("maps")).unwrap();
    fs::write(dir.join("maps").join("level.txt"), "#####\n#P..#\n#####\n").unwrap();
    fs::write(
        dir.join("game.ron"),
        r#"(
            version: 1,
            assets: (textures: ["player", "floor", "wall"]),
            controls: TopDown,
            prefabs: [Player((name: "player", sprite: "player"))],
            maps: [
                TextMap((
                    name: "level",
                    path: "maps/level.txt",
                    theme: ("floor", "wall"),
                    legend: {'P': "player"},
                    start: true,
                )),
            ],
            rules: [TopDownControls],
        )"#,
    )
    .unwrap();

    let _guard = EnvVarGuard::set("GAME_ASSET_DIR", dir.as_os_str());
    game_kit::data::validate_beginner_game_file("game.ron").unwrap();
}

fn temp_data_project(name: &str) -> PathBuf {
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

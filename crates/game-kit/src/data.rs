//! Data-driven beginner game setup.
//!
//! `assets/game.ron` is a small layer over the public beginner builders, not a
//! second runtime. It covers conventional named assets, player/enemy/pickup
//! prefabs, text maps, standard top-down controls, and common rules. Rust can
//! add custom behavior after [`load_beginner_game_file`] returns.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::{Context, Result, anyhow};
use serde::Deserialize;

use crate::app::GameApp;
use crate::input::TopDownControls;
use crate::map::beginner_asset_path;

/// Loads a beginner RON file from the asset root and compiles it through the
/// normal `GameApp` asset, prefab, map, input, and rule builders.
pub fn load_beginner_game_file(
    game: &mut GameApp<'_>,
    path: impl AsRef<Path>,
) -> Result<TopDownControls> {
    let requested = path.as_ref();
    let path_text = requested.to_string_lossy();
    let full_path = beginner_asset_path(&path_text);
    let source = std::fs::read_to_string(&full_path).with_context(|| {
        format!(
            "could not read beginner game file 'assets/{}' (looked for '{}')",
            requested.display(),
            full_path.display()
        )
    })?;
    load_beginner_game_text(game, &source, &requested.display().to_string())
}

/// The file-shaped data model used by `assets/game.ron`.
#[derive(Clone, Debug, Deserialize)]
pub struct BeginnerGameFile {
    #[serde(default = "default_beginner_game_version")]
    pub version: u32,
    #[serde(default)]
    pub assets: BeginnerAssetsFile,
    #[serde(default = "default_controls")]
    pub controls: String,
    #[serde(default)]
    pub prefabs: Vec<BeginnerPrefabFile>,
    #[serde(default)]
    pub maps: Vec<BeginnerMapFile>,
    #[serde(default)]
    pub rules: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct BeginnerAssetsFile {
    #[serde(default)]
    pub textures: Vec<String>,
    #[serde(default)]
    pub sounds: Vec<String>,
    #[serde(default)]
    pub music: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub enum BeginnerPrefabFile {
    Player(PlayerPrefabFile),
    Enemy(EnemyPrefabFile),
    Pickup(PickupPrefabFile),
}

#[derive(Clone, Debug, Deserialize)]
pub struct PlayerPrefabFile {
    pub name: String,
    pub sprite: String,
    #[serde(default = "default_player_speed")]
    pub speed: f32,
    #[serde(default = "default_player_health")]
    pub health: i32,
    #[serde(default)]
    pub melee: Option<MeleeFile>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct EnemyPrefabFile {
    pub name: String,
    pub sprite: String,
    #[serde(default = "default_enemy_speed")]
    pub speed: f32,
    #[serde(default = "default_enemy_health")]
    pub health: i32,
    #[serde(default)]
    pub chase_player: bool,
    #[serde(default)]
    pub melee: Option<MeleeFile>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PickupPrefabFile {
    pub name: String,
    pub sprite: String,
    #[serde(default = "default_pickup_score")]
    pub score: i32,
    #[serde(default)]
    pub sound: Option<String>,
    #[serde(default = "default_despawn_on_collect")]
    pub despawn_on_collect: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MeleeFile {
    pub range: f32,
    pub damage: i32,
}

#[derive(Clone, Debug, Deserialize)]
pub enum BeginnerMapFile {
    TextMap(TextMapFile),
}

#[derive(Clone, Debug, Deserialize)]
pub struct TextMapFile {
    pub name: String,
    pub path: String,
    pub theme: (String, String),
    #[serde(default)]
    pub legend: BTreeMap<char, String>,
    #[serde(default)]
    pub start: bool,
}

fn load_beginner_game_text(
    game: &mut GameApp<'_>,
    source: &str,
    label: &str,
) -> Result<TopDownControls> {
    let file: BeginnerGameFile = ron::from_str(source)
        .map_err(|error| anyhow!("beginner game file '{label}' is not valid RON: {error}"))?;
    validate_file(&file, label)?;

    game.assets(|assets| {
        for key in &file.assets.textures {
            assets.texture(key.clone(), format!("textures/{key}.png"))?;
        }
        for key in &file.assets.sounds {
            assets.sound(key.clone(), format!("sounds/{key}.wav"))?;
        }
        for key in &file.assets.music {
            assets.music(key.clone(), format!("music/{key}.ogg"))?;
        }
        Ok(())
    })?;

    let controls = match file.controls.as_str() {
        "top_down" => game.input(|input| input.top_down_controls())?,
        other => anyhow::bail!(
            "beginner game file '{label}' has unsupported controls '{other}'. Supported controls: top_down"
        ),
    };

    for prefab in file.prefabs {
        match prefab {
            BeginnerPrefabFile::Player(player) => {
                let prefab = game
                    .player_prefab(player.name)
                    .sprite(player.sprite)
                    .moves_with(controls.movement, player.speed)
                    .health(player.health);
                let prefab = match player.melee {
                    Some(melee) => prefab.melee(melee.range, melee.damage),
                    None => prefab,
                };
                prefab.build()?;
            }
            BeginnerPrefabFile::Enemy(enemy) => {
                let prefab = game
                    .enemy_prefab(enemy.name)
                    .sprite(enemy.sprite)
                    .speed(enemy.speed)
                    .health(enemy.health);
                let prefab = if enemy.chase_player {
                    prefab.chases_player()
                } else {
                    prefab
                };
                let prefab = match enemy.melee {
                    Some(melee) => prefab.melee(melee.range, melee.damage),
                    None => prefab,
                };
                prefab.build()?;
            }
            BeginnerPrefabFile::Pickup(pickup) => {
                let prefab = game
                    .pickup_prefab(pickup.name)
                    .sprite(pickup.sprite)
                    .score(pickup.score);
                let prefab = match pickup.sound {
                    Some(sound) => prefab.play_sound(sound),
                    None => prefab,
                };
                let prefab = if pickup.despawn_on_collect {
                    prefab.despawn_on_collect()
                } else {
                    prefab
                };
                prefab.build()?;
            }
        }
    }

    for map in file.maps {
        match map {
            BeginnerMapFile::TextMap(map) => {
                let mut author = game
                    .map_from_text(map.name.as_str(), map.path.as_str())
                    .simple_theme(map.theme.0.as_str(), map.theme.1.as_str());
                for (symbol, prefab) in map.legend {
                    author = author.legend(symbol, prefab);
                }
                if map.start {
                    author.start();
                } else {
                    author.finish();
                }
            }
        }
    }

    let mut rules = game.rules();
    for rule in &file.rules {
        rules = match rule.as_str() {
            "top_down_controls" => rules.top_down_controls(controls),
            "player_collects_pickups" => rules.player_collects_pickups(),
            "enemies_damage_player" => rules.enemies_damage_player(),
            "camera_follows_player" => rules.camera_follows_player(),
            "show_score" => rules.show_score(),
            "show_player_health" => rules.show_player_health(),
            "show_enemy_count" => rules.show_enemy_count(),
            "pause_and_reset" => rules.pause_and_reset(),
            "doors_change_maps" => rules.doors_change_maps(),
            other => anyhow::bail!(
                "beginner game file '{label}' has unknown rule '{other}'. Supported rules: top_down_controls, player_collects_pickups, enemies_damage_player, camera_follows_player, show_score, show_player_health, show_enemy_count, pause_and_reset, doors_change_maps"
            ),
        };
    }
    rules.build();
    Ok(controls)
}

fn validate_file(file: &BeginnerGameFile, label: &str) -> Result<()> {
    if file.version != 1 {
        anyhow::bail!(
            "unsupported beginner game file version {}. Supported version: 1",
            file.version
        );
    }
    let textures = file
        .assets
        .textures
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let sounds = file
        .assets
        .sounds
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let names = file
        .prefabs
        .iter()
        .map(BeginnerPrefabFile::name)
        .collect::<Vec<_>>();
    let unique_names = names.iter().copied().collect::<BTreeSet<_>>();
    if unique_names.len() != names.len() {
        anyhow::bail!("beginner game file '{label}' defines duplicate prefab names");
    }

    for prefab in &file.prefabs {
        require_known(label, "texture", prefab.name(), prefab.sprite(), &textures)?;
        if let BeginnerPrefabFile::Pickup(pickup) = prefab
            && let Some(sound) = pickup.sound.as_deref()
        {
            require_known(label, "sound", &pickup.name, sound, &sounds)?;
        }
    }

    for map in &file.maps {
        let BeginnerMapFile::TextMap(map) = map;
        require_known(label, "texture", &map.name, &map.theme.0, &textures)?;
        require_known(label, "texture", &map.name, &map.theme.1, &textures)?;
        for (symbol, prefab) in &map.legend {
            if !unique_names.iter().any(|candidate| *candidate == prefab) {
                let suggestion = suggest(prefab, unique_names.iter().copied())
                    .map(|candidate| format!(" Did you mean '{candidate}'?"))
                    .unwrap_or_default();
                anyhow::bail!(
                    "beginner game file '{label}' map '{}' legend symbol {:?} references unknown prefab '{}'.{suggestion}",
                    map.name,
                    symbol,
                    prefab
                );
            }
        }
    }
    Ok(())
}

fn require_known(
    label: &str,
    kind: &str,
    owner: &str,
    key: &str,
    known: &BTreeSet<&str>,
) -> Result<()> {
    if known.iter().any(|candidate| *candidate == key) {
        return Ok(());
    }
    let listed = known.iter().copied().collect::<Vec<_>>().join(", ");
    let suggestion = suggest(key, known.iter().copied())
        .map(|candidate| format!(" Did you mean '{candidate}'?"))
        .unwrap_or_default();
    anyhow::bail!(
        "beginner game file '{label}' prefab/map '{}' uses {kind} '{}', but no such {kind} was registered. Known {kind}s: {}.{suggestion}",
        owner,
        key,
        if listed.is_empty() { "(none)" } else { &listed },
    )
}

fn suggest<'a>(needle: &str, candidates: impl Iterator<Item = &'a str>) -> Option<&'a str> {
    let candidate = candidates.min_by_key(|candidate| edit_distance(needle, candidate))?;
    let distance = edit_distance(needle, candidate);
    let threshold = (needle.chars().count().max(candidate.chars().count()) / 3).max(2);
    (distance <= threshold).then_some(candidate)
}

fn edit_distance(left: &str, right: &str) -> usize {
    let right = right.chars().collect::<Vec<_>>();
    let mut previous = (0..=right.len()).collect::<Vec<_>>();
    for (left_index, left_char) in left.chars().enumerate() {
        let mut current = vec![left_index + 1];
        for (right_index, right_char) in right.iter().enumerate() {
            let replace = previous[right_index] + usize::from(left_char != *right_char);
            let insert = current[right_index] + 1;
            let delete = previous[right_index + 1] + 1;
            current.push(replace.min(insert).min(delete));
        }
        previous = current;
    }
    previous[right.len()]
}

impl BeginnerPrefabFile {
    fn name(&self) -> &str {
        match self {
            Self::Player(prefab) => &prefab.name,
            Self::Enemy(prefab) => &prefab.name,
            Self::Pickup(prefab) => &prefab.name,
        }
    }

    fn sprite(&self) -> &str {
        match self {
            Self::Player(prefab) => &prefab.sprite,
            Self::Enemy(prefab) => &prefab.sprite,
            Self::Pickup(prefab) => &prefab.sprite,
        }
    }
}

fn default_controls() -> String {
    "top_down".to_owned()
}

const fn default_beginner_game_version() -> u32 {
    1
}

const fn default_player_speed() -> f32 {
    130.0
}

const fn default_player_health() -> i32 {
    100
}

const fn default_enemy_speed() -> f32 {
    80.0
}

const fn default_enemy_health() -> i32 {
    30
}

const fn default_pickup_score() -> i32 {
    1
}

const fn default_despawn_on_collect() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::{BeginnerGameFile, load_beginner_game_text, validate_file};
    use crate::app::{GameApp, GamePlugin};
    use crate::harness::GameTestHarness;
    use anyhow::Result;

    const GAME: &str = r#"(
    version: 1,
    assets: (
        textures: ["player", "slime", "coin", "floor", "wall"],
        sounds: ["hit"],
    ),
    prefabs: [
        Player((name: "player", sprite: "player")),
        Enemy((name: "slime", sprite: "slime", chase_player: true)),
        Pickup((name: "coin", sprite: "coin", score: 1, sound: Some("hit"))),
    ],
    maps: [
        TextMap((
            name: "level_1",
            path: "maps/beginner_text_map.txt",
            theme: ("floor", "wall"),
            legend: {'P': "player", 'E': "slime", 'C': "coin"},
            start: true,
        )),
    ],
    rules: ["top_down_controls", "player_collects_pickups", "show_score"],
)"#;

    struct DataPlugin;

    impl GamePlugin for DataPlugin {
        fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
            load_beginner_game_text(game, GAME, "inline.ron").map(|_| ())
        }
    }

    struct FileDataPlugin;

    impl GamePlugin for FileDataPlugin {
        fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
            game.load_beginner_file("game.ron").map(|_| ())
        }
    }

    #[test]
    fn compiles_the_small_game_file_through_the_normal_beginner_builders() {
        let game = GameTestHarness::from_plugin(DataPlugin).unwrap();

        assert_eq!(game.current_map_name().as_deref(), Some("level_1"));
        assert_eq!(game.count::<crate::beginner::actors::Player>(), 1);
        assert_eq!(game.count::<crate::beginner::actors::Enemy>(), 1);
        assert_eq!(game.count::<crate::beginner::actors::Pickup>(), 1);
    }

    #[test]
    fn validation_names_unknown_legend_prefabs_and_offers_a_suggestion() {
        let source = GAME.replace("'E': \"slime\"", "'E': \"slimee\"");
        let file: BeginnerGameFile = ron::from_str(&source).unwrap();
        let error = validate_file(&file, "game.ron").unwrap_err().to_string();

        assert!(error.contains("legend symbol 'E' references unknown prefab 'slimee'"));
        assert!(error.contains("Did you mean 'slime'?"));
    }

    #[test]
    fn validation_names_unknown_prefab_assets_and_lists_known_keys() {
        let source = GAME.replace("sprite: \"player\"", "sprite: \"plaeyr\"");
        let file: BeginnerGameFile = ron::from_str(&source).unwrap();
        let error = validate_file(&file, "game.ron").unwrap_err().to_string();

        assert!(error.contains("uses texture 'plaeyr'"));
        assert!(error.contains("Known textures: coin, floor, player, slime, wall"));
        assert!(error.contains("Did you mean 'player'?"));
    }

    #[test]
    fn public_file_loader_reads_the_checked_in_game_ron() {
        let game = GameTestHarness::from_plugin(FileDataPlugin).unwrap();

        assert_eq!(game.current_map_name().as_deref(), Some("level_1"));
        assert_eq!(game.count::<crate::beginner::actors::Player>(), 1);
    }

    #[test]
    fn missing_version_defaults_to_one_and_validates() {
        let source = "(assets: (), prefabs: [], maps: [], rules: [])";
        let file: BeginnerGameFile = ron::from_str(source).unwrap();
        assert_eq!(file.version, 1);
        validate_file(&file, "test.ron").unwrap();
    }

    #[test]
    fn unsupported_version_rejects_with_helpful_error() {
        let source = "(version: 2, assets: (), prefabs: [], maps: [], rules: [])";
        let file: BeginnerGameFile = ron::from_str(source).unwrap();
        let err = validate_file(&file, "test.ron").unwrap_err().to_string();
        assert!(err.contains("unsupported beginner game file version 2"));
        assert!(err.contains("Supported version: 1"));
    }
}

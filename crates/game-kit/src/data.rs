//! Data-driven beginner game setup.
//!
//! `assets/game.ron` is a small layer over the public beginner builders, not a
//! second runtime. It covers conventional named assets, common beginner
//! prefabs, text maps, scene/audio hooks, standard top-down controls, and
//! declarative rules. Rust can add custom behavior after
//! [`load_beginner_game_file`] returns.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use game_core::builder::GameBuilder;
use game_core::input::ActionId;
use glam::vec2;
use serde::Deserialize;

use crate::app::GameApp;
use crate::beginner::context::Game as BeginnerGame;
use crate::beginner::custom_rules::register_runtime_countdown_rule;
use crate::beginner::events::EnemyDeathEvent;
use crate::beginner::rules::RulesAuthor;
use crate::context::{GameCtx, StartupGameCtx};
use crate::diagnostics::{
    bad_map_symbol_error, bad_rule_combo_error, closest_name, missing_file_error,
    unknown_reference_error,
};
use crate::input::TopDownControls;
use crate::map::{ContentRuntime, beginner_asset_path};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BeginnerReloadLevel {
    NotSupported,
    Partial,
    Ok,
}

impl BeginnerReloadLevel {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::NotSupported => "not supported",
            Self::Partial => "partial",
            Self::Ok => "ok",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BeginnerFileRuntime {
    pub(crate) path: PathBuf,
    pub(crate) last_loaded_version: u64,
    pub(crate) last_error: Option<String>,
    pub(crate) reload_level: BeginnerReloadLevel,
    identity: BeginnerReloadIdentity,
}

impl BeginnerFileRuntime {
    fn new(path: PathBuf, identity: BeginnerReloadIdentity) -> Self {
        Self {
            path,
            last_loaded_version: 1,
            last_error: None,
            reload_level: BeginnerReloadLevel::Partial,
            identity,
        }
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn identity(&self) -> &BeginnerReloadIdentity {
        &self.identity
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct BeginnerReloadIdentity {
    textures: Vec<String>,
    sounds: Vec<String>,
    music: Vec<String>,
    animation_sheets: Vec<String>,
    prefabs: Vec<String>,
    maps: Vec<String>,
    custom_rules: Vec<String>,
    rules: Vec<BeginnerRuleIdentity>,
    scene_flow: Option<SceneFlowIdentity>,
    actions: Vec<BeginnerActionIdentity>,
}

impl BeginnerReloadIdentity {
    fn from_file(file: &BeginnerGameFile, label: &str) -> Result<Self> {
        Ok(Self {
            textures: file.assets.textures.clone(),
            sounds: file.assets.sounds.clone(),
            music: file.assets.music.clone(),
            animation_sheets: file.assets.animation_sheets.clone(),
            prefabs: file
                .prefabs
                .iter()
                .map(|prefab| prefab.name().to_owned())
                .collect(),
            maps: file.maps.iter().map(|map| map.name().to_owned()).collect(),
            custom_rules: file
                .custom_rules
                .iter()
                .map(|rule| rule.name().to_owned())
                .collect(),
            rules: file
                .rules
                .iter()
                .map(|rule| rule.identity(label))
                .collect::<Result<Vec<_>>>()?,
            scene_flow: file.scene_flow.as_ref().map(SceneFlowIdentity::from_file),
            actions: file
                .actions
                .iter()
                .map(BeginnerActionIdentity::from_file)
                .collect(),
        })
    }

    fn ensure_matches(&self, other: &Self, label: &str) -> Result<()> {
        ensure_same_list(label, "texture assets", &self.textures, &other.textures)?;
        ensure_same_list(label, "sound assets", &self.sounds, &other.sounds)?;
        ensure_same_list(label, "music assets", &self.music, &other.music)?;
        ensure_same_list(
            label,
            "animation sheet assets",
            &self.animation_sheets,
            &other.animation_sheets,
        )?;
        ensure_same_list(label, "prefabs", &self.prefabs, &other.prefabs)?;
        ensure_same_list(label, "maps", &self.maps, &other.maps)?;
        ensure_same_list(
            label,
            "custom rules",
            &self.custom_rules,
            &other.custom_rules,
        )?;
        ensure_same_values(label, "enabled rules", &self.rules, &other.rules)?;
        ensure_same_values(
            label,
            "scene flow structure",
            &self.scene_flow,
            &other.scene_flow,
        )?;
        ensure_same_values(label, "actions", &self.actions, &other.actions)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SceneFlowIdentity {
    menu: Option<String>,
    game: Option<String>,
    game_over: Option<String>,
    win: Option<String>,
    start_on: Option<ActionFile>,
    restart_on: Option<ActionFile>,
}

impl SceneFlowIdentity {
    fn from_file(flow: &SceneFlowFile) -> Self {
        Self {
            menu: flow.menu.clone(),
            game: flow.game.clone(),
            game_over: flow.game_over.clone(),
            win: flow.win.clone(),
            start_on: flow.start_on,
            restart_on: flow.restart_on,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum BeginnerRuleIdentity {
    Simple(BeginnerRuleKind),
    Script(BeginnerScriptRuleFile),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum BeginnerActionIdentity {
    PlayerShoots { action: ActionFile },
}

impl BeginnerActionIdentity {
    fn from_file(action: &BeginnerActionFile) -> Self {
        match action {
            BeginnerActionFile::PlayerShoots(shoot) => Self::PlayerShoots {
                action: shoot.action,
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct BeginnerRuntimeConfig {
    custom_countdowns: Vec<BeginnerCountdownRuleConfig>,
    scene_flow: Option<SceneFlowFile>,
    audio: AudioFile,
    actions: Vec<BeginnerActionFile>,
}

impl BeginnerRuntimeConfig {
    fn from_file(file: &BeginnerGameFile) -> Self {
        Self {
            custom_countdowns: file
                .custom_rules
                .iter()
                .map(BeginnerCountdownRuleConfig::from_file)
                .collect(),
            scene_flow: file.scene_flow.clone(),
            audio: file.audio.clone(),
            actions: file.actions.clone(),
        }
    }

    pub(crate) fn custom_countdown_rule(&self, name: &str) -> Option<&BeginnerCountdownRuleConfig> {
        self.custom_countdowns.iter().find(|rule| rule.name == name)
    }

    pub(crate) fn scene_flow(&self) -> Option<&SceneFlowFile> {
        self.scene_flow.as_ref()
    }

    pub(crate) fn audio(&self) -> &AudioFile {
        &self.audio
    }

    fn player_shoots_action(&self, index: usize) -> Option<&PlayerShootsFile> {
        match self.actions.get(index)? {
            BeginnerActionFile::PlayerShoots(shoot) => Some(shoot),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct BeginnerCountdownRuleConfig {
    pub(crate) name: String,
    pub(crate) tag: String,
    pub(crate) key: String,
    pub(crate) effects: Vec<BeginnerCountdownEffectConfig>,
}

impl BeginnerCountdownRuleConfig {
    fn from_file(rule: &CustomRuleFile) -> Self {
        match rule {
            CustomRuleFile::Countdown(rule) => Self {
                name: rule.name.clone(),
                tag: rule.tag.clone(),
                key: rule.key.clone(),
                effects: rule
                    .when_zero
                    .iter()
                    .map(BeginnerCountdownEffectConfig::from_file)
                    .collect(),
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum BeginnerCountdownEffectConfig {
    AddScore(i32),
    SetScore(i32),
    DamageTagged {
        tag: String,
        amount: i32,
        radius: f32,
    },
    DamagePlayer {
        amount: i32,
        radius: f32,
    },
    DespawnSelf,
    PlaySound(String),
    PlayMusic(String),
    StopMusic,
    SpawnPrefab(String),
    SpawnNearPlayer {
        prefab: String,
        radius: f32,
    },
    ChangeScene(String),
    ChangeMap(String),
    RestartCurrentMap,
    ShowUiText(String),
    HealPlayer(i32),
    SetData {
        tag: String,
        key: String,
        value: f32,
    },
    DespawnTagged(String),
}

impl BeginnerCountdownEffectConfig {
    fn from_file(effect: &RuleEffectFile) -> Self {
        match effect {
            RuleEffectFile::AddScore(amount) => Self::AddScore(*amount),
            RuleEffectFile::SetScore(score) => Self::SetScore(*score),
            RuleEffectFile::DamageTagged {
                tag,
                amount,
                radius,
            } => Self::DamageTagged {
                tag: tag.clone(),
                amount: *amount,
                radius: *radius,
            },
            RuleEffectFile::DamagePlayer { amount, radius } => Self::DamagePlayer {
                amount: *amount,
                radius: *radius,
            },
            RuleEffectFile::DespawnSelf => Self::DespawnSelf,
            RuleEffectFile::PlaySound(sound) => Self::PlaySound(sound.clone()),
            RuleEffectFile::PlayMusic(music) => Self::PlayMusic(music.clone()),
            RuleEffectFile::StopMusic => Self::StopMusic,
            RuleEffectFile::SpawnPrefab(prefab) => Self::SpawnPrefab(prefab.clone()),
            RuleEffectFile::SpawnNearPlayer { prefab, radius } => Self::SpawnNearPlayer {
                prefab: prefab.clone(),
                radius: *radius,
            },
            RuleEffectFile::ChangeScene(scene) => Self::ChangeScene(scene.clone()),
            RuleEffectFile::ChangeMap(map) => Self::ChangeMap(map.clone()),
            RuleEffectFile::RestartCurrentMap => Self::RestartCurrentMap,
            RuleEffectFile::ShowUiText(text) => Self::ShowUiText(text.clone()),
            RuleEffectFile::HealPlayer(amount) => Self::HealPlayer(*amount),
            RuleEffectFile::SetData { tag, key, value } => Self::SetData {
                tag: tag.clone(),
                key: key.clone(),
                value: *value,
            },
            RuleEffectFile::DespawnTagged(tag) => Self::DespawnTagged(tag.clone()),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct BeginnerRuleUiText {
    pub(crate) lines: Vec<String>,
}

pub(crate) struct RebuiltBeginnerContent {
    pub(crate) runtime: ContentRuntime,
    pub(crate) config: BeginnerRuntimeConfig,
}

/// Loads a beginner RON file from the asset root and compiles it through the
/// normal `GameApp` asset, prefab, map, input, action, scene, audio, and rule
/// builders.
pub fn load_beginner_game_file(
    game: &mut GameApp<'_>,
    path: impl AsRef<Path>,
) -> Result<TopDownControls> {
    let loaded = read_beginner_game_file(path)?;
    let identity = BeginnerReloadIdentity::from_file(&loaded.file, &loaded.label)?;
    let runtime = BeginnerFileRuntime::new(loaded.full_path.clone(), identity);
    let controls =
        build_beginner_game_file(game, loaded.file, &loaded.label, loaded.full_path.parent())?;
    game.startup(move |game: &mut crate::StartupGameCtx<'_, '_>| {
        game.insert_resource(runtime.clone());
        Ok(())
    });
    Ok(controls)
}

/// Validates a beginner RON file without starting runtime backends.
///
/// This is the same path [`GameApp::load_beginner_file`] uses, followed by the
/// normal content finalization checks for maps, prefabs, and start-map state.
pub fn validate_beginner_game_file(path: impl AsRef<Path>) -> Result<()> {
    let mut builder = GameBuilder::new();
    let mut game = GameApp::new(&mut builder);
    load_beginner_game_file(&mut game, path)?;
    game.finish()
}

pub(crate) fn rebuild_beginner_content_runtime(
    path: &Path,
    expected_identity: &BeginnerReloadIdentity,
) -> Result<RebuiltBeginnerContent> {
    let loaded = read_beginner_game_file(path)?;
    let identity = BeginnerReloadIdentity::from_file(&loaded.file, &loaded.label)?;
    expected_identity.ensure_matches(&identity, &loaded.label)?;
    let config = BeginnerRuntimeConfig::from_file(&loaded.file);

    let mut builder = GameBuilder::new();
    let mut game = GameApp::new(&mut builder);
    build_beginner_game_file(
        &mut game,
        loaded.file,
        &loaded.label,
        loaded.full_path.parent(),
    )?;
    let runtime = game.finish_for_reload()?;
    Ok(RebuiltBeginnerContent { runtime, config })
}

/// The file-shaped data model used by `assets/game.ron`.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct BeginnerGameFile {
    #[serde(default = "default_beginner_game_version")]
    pub version: u32,
    #[serde(default)]
    pub assets: BeginnerAssetsFile,
    #[serde(default = "default_controls")]
    pub controls: BeginnerControlsFile,
    #[serde(default)]
    pub prefabs: Vec<BeginnerPrefabFile>,
    #[serde(default)]
    pub maps: Vec<BeginnerMapFile>,
    #[serde(default)]
    pub scene_flow: Option<SceneFlowFile>,
    #[serde(default)]
    pub audio: AudioFile,
    #[serde(default)]
    pub actions: Vec<BeginnerActionFile>,
    #[serde(default)]
    pub custom_rules: Vec<CustomRuleFile>,
    #[serde(default)]
    pub rules: Vec<BeginnerRuleFile>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct BeginnerAssetsFile {
    #[serde(default)]
    pub textures: Vec<String>,
    #[serde(default)]
    pub sounds: Vec<String>,
    #[serde(default)]
    pub music: Vec<String>,
    #[serde(default)]
    pub animation_sheets: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum BeginnerControlsFile {
    Structured(BeginnerControlsKind),
    Legacy(String),
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
pub enum BeginnerControlsKind {
    TopDown,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub enum BeginnerPrefabFile {
    Player(PlayerPrefabFile),
    Enemy(EnemyPrefabFile),
    Pickup(PickupPrefabFile),
    Door(DoorPrefabFile),
    Projectile(ProjectilePrefabFile),
    Spawner(SpawnerPrefabFile),
    Trigger(TriggerPrefabFile),
    Checkpoint(CheckpointPrefabFile),
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct PlayerPrefabFile {
    pub name: String,
    pub sprite: String,
    #[serde(default)]
    pub animation_sheet: Option<String>,
    #[serde(default = "default_player_speed")]
    pub speed: f32,
    #[serde(default = "default_player_health")]
    pub health: i32,
    #[serde(default)]
    pub melee: Option<MeleeFile>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub data: BTreeMap<String, f32>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct EnemyPrefabFile {
    pub name: String,
    pub sprite: String,
    #[serde(default)]
    pub animation_sheet: Option<String>,
    #[serde(default = "default_enemy_speed")]
    pub speed: f32,
    #[serde(default = "default_enemy_health")]
    pub health: i32,
    #[serde(default)]
    pub chase_player: bool,
    #[serde(default)]
    pub melee: Option<MeleeFile>,
    #[serde(default)]
    pub drops: Option<String>,
    #[serde(default)]
    pub drop_chance: Option<f32>,
    #[serde(default)]
    pub despawn_after_death_animation: bool,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub data: BTreeMap<String, f32>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct PickupPrefabFile {
    pub name: String,
    pub sprite: String,
    #[serde(default = "default_pickup_score")]
    pub score: i32,
    #[serde(default)]
    pub heal_player: Option<i32>,
    #[serde(default)]
    pub sound: Option<String>,
    #[serde(default = "default_despawn_on_collect")]
    pub despawn_on_collect: bool,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub data: BTreeMap<String, f32>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct DoorPrefabFile {
    pub name: String,
    pub sprite: String,
    pub action: DoorActionFile,
    #[serde(default)]
    pub requires_all_enemies_dead: bool,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub data: BTreeMap<String, f32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub enum DoorActionFile {
    ChangeMap(String),
    ChangeScene(String),
    RestartLevel,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct ProjectilePrefabFile {
    pub name: String,
    pub sprite: String,
    #[serde(default)]
    pub animation_sheet: Option<String>,
    #[serde(default = "default_projectile_damage")]
    pub damage: i32,
    #[serde(default = "default_projectile_speed")]
    pub speed: f32,
    #[serde(default = "default_projectile_lifetime")]
    pub lifetime: f32,
    #[serde(default = "default_true")]
    pub despawn_on_hit: bool,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub data: BTreeMap<String, f32>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct SpawnerPrefabFile {
    pub name: String,
    pub spawn: String,
    #[serde(default = "default_spawn_every")]
    pub every_seconds: f32,
    #[serde(default)]
    pub max_alive: Option<usize>,
    #[serde(default)]
    pub placement: SpawnPlacementFile,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub enum SpawnPlacementFile {
    #[default]
    AtSpawner,
    NearPlayer(f32),
    AtFirstFloor,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct TriggerPrefabFile {
    pub name: String,
    #[serde(default = "default_area_size")]
    pub size: (f32, f32),
    #[serde(default)]
    pub visible_debug: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub data: BTreeMap<String, f32>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct CheckpointPrefabFile {
    pub name: String,
    pub sprite: String,
    #[serde(default = "default_area_size")]
    pub size: (f32, f32),
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub data: BTreeMap<String, f32>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct MeleeFile {
    pub range: f32,
    pub damage: i32,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub enum BeginnerMapFile {
    TextMap(TextMapFile),
    TextMapAuto(TextMapAutoFile),
    Tiled(TiledMapFile),
    Ldtk(LdtkMapFile),
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct TextMapFile {
    pub name: String,
    pub path: String,
    pub theme: (String, String),
    #[serde(default = "default_tile_size")]
    pub tile_size: f32,
    #[serde(default)]
    pub legend: BTreeMap<char, String>,
    #[serde(default)]
    pub start: bool,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct TextMapAutoFile {
    pub name: String,
    pub theme: (String, String),
    #[serde(default = "default_tile_size")]
    pub tile_size: f32,
    #[serde(default)]
    pub legend: BTreeMap<char, String>,
    #[serde(default)]
    pub start: bool,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct TiledMapFile {
    pub name: String,
    pub path: String,
    pub theme: (String, String),
    #[serde(default)]
    pub objects: BTreeMap<String, String>,
    #[serde(default)]
    pub start: bool,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct LdtkMapFile {
    pub name: String,
    pub path: String,
    pub level: String,
    pub theme: (String, String),
    #[serde(default)]
    pub entities: BTreeMap<String, String>,
    #[serde(default)]
    pub start: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct SceneFlowFile {
    #[serde(default)]
    pub menu: Option<String>,
    #[serde(default)]
    pub game: Option<String>,
    #[serde(default)]
    pub game_over: Option<String>,
    #[serde(default)]
    pub win: Option<String>,
    #[serde(default)]
    pub menu_text: Option<String>,
    #[serde(default)]
    pub menu_button: Option<SceneButtonFile>,
    #[serde(default)]
    pub game_over_text: Option<String>,
    #[serde(default)]
    pub game_over_button: Option<String>,
    #[serde(default)]
    pub win_text: Option<String>,
    #[serde(default)]
    pub win_button: Option<String>,
    #[serde(default)]
    pub start_on: Option<ActionFile>,
    #[serde(default)]
    pub restart_on: Option<ActionFile>,
    #[serde(default)]
    pub win_condition: Option<WinConditionFile>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct SceneButtonFile {
    pub label: String,
    pub map: String,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
pub enum WinConditionFile {
    AllPickupsCollected,
    AllEnemiesDead,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct AudioFile {
    #[serde(default)]
    pub music_on_scene: BTreeMap<String, MusicPlaybackFile>,
    #[serde(default)]
    pub master_volume: Option<f32>,
    #[serde(default)]
    pub music_volume: Option<f32>,
    #[serde(default)]
    pub sfx_volume: Option<f32>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct MusicPlaybackFile {
    pub track: String,
    #[serde(default = "default_music_volume")]
    pub volume: f32,
    #[serde(default)]
    pub fade_in: Option<f32>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub enum BeginnerActionFile {
    PlayerShoots(PlayerShootsFile),
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct PlayerShootsFile {
    pub prefab: String,
    #[serde(default)]
    pub action: ActionFile,
    #[serde(default = "default_shoot_cooldown")]
    pub cooldown: f32,
    #[serde(default)]
    pub direction: ShotDirectionFile,
    #[serde(default)]
    pub sound: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ActionFile {
    #[default]
    Attack,
    Pause,
    Reset,
    Reload,
    MenuAccept,
}

impl<'de> Deserialize<'de> for ActionFile {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ActionFileVisitor;

        impl serde::de::Visitor<'_> for ActionFileVisitor {
            type Value = ActionFile;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter
                    .write_str("an action name such as Attack, Pause, Reset, Reload, or MenuAccept")
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                action_file_from_name(value)
                    .ok_or_else(|| E::custom(unknown_action_file_message(value)))
            }

            fn visit_string<E>(self, value: String) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_str(&value)
            }
        }

        deserializer.deserialize_identifier(ActionFileVisitor)
    }
}

const ACTION_FILE_NAMES: &[&str] = &["Attack", "Pause", "Reset", "Reload", "MenuAccept"];

fn action_file_from_name(name: &str) -> Option<ActionFile> {
    match name {
        "Attack" => Some(ActionFile::Attack),
        "Pause" => Some(ActionFile::Pause),
        "Reset" => Some(ActionFile::Reset),
        "Reload" => Some(ActionFile::Reload),
        "MenuAccept" => Some(ActionFile::MenuAccept),
        _ => None,
    }
}

fn unknown_action_file_message(name: &str) -> String {
    let suggestion = closest_name(name, ACTION_FILE_NAMES.iter().copied())
        .map(|candidate| format!(" Did you mean '{candidate}'?"))
        .unwrap_or_default();
    format!(
        "unknown action '{name}'. Known actions: {}.{suggestion}",
        ACTION_FILE_NAMES.join(", ")
    )
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
pub enum ShotDirectionFile {
    #[default]
    TowardsMouse,
    Right,
    Left,
    Up,
    Down,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub enum CustomRuleFile {
    Countdown(CountdownRuleFile),
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct CountdownRuleFile {
    pub name: String,
    pub tag: String,
    pub key: String,
    pub when_zero: Vec<RuleEffectFile>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub enum RuleEffectFile {
    AddScore(i32),
    SetScore(i32),
    DamageTagged {
        tag: String,
        amount: i32,
        radius: f32,
    },
    DamagePlayer {
        amount: i32,
        #[serde(default)]
        radius: f32,
    },
    DespawnSelf,
    PlaySound(String),
    PlayMusic(String),
    StopMusic,
    SpawnPrefab(String),
    SpawnNearPlayer {
        prefab: String,
        radius: f32,
    },
    ChangeScene(String),
    ChangeMap(String),
    RestartCurrentMap,
    ShowUiText(String),
    HealPlayer(i32),
    SetData {
        tag: String,
        key: String,
        value: f32,
    },
    DespawnTagged(String),
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub enum RuleConditionFile {
    AllEnemiesDead,
    AllPickupsCollected,
    ScoreAtLeast(i32),
    PlayerHealthBelow(i32),
    TimerReached { name: String, seconds: f32 },
    MapIs(String),
    SceneIs(String),
    TagCountZero(String),
    ActionPressed(ActionFile),
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum BeginnerRuleFile {
    Structured(BeginnerRuleKind),
    Script(BeginnerScriptRuleFile),
    Legacy(String),
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub enum BeginnerScriptRuleFile {
    When {
        condition: RuleConditionFile,
        effects: Vec<RuleEffectFile>,
    },
    OnEnemyDeath {
        prefab: String,
        effects: Vec<RuleEffectFile>,
    },
    EverySeconds {
        seconds: f32,
        effects: Vec<RuleEffectFile>,
    },
    OnScoreReaches {
        score: i32,
        effects: Vec<RuleEffectFile>,
    },
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
pub enum BeginnerRuleKind {
    TopDownControls,
    PlayerCollectsPickups,
    EnemiesDamagePlayer,
    DeadEnemiesDespawn,
    EnemyDrops,
    Projectiles,
    ProjectilesMove,
    ProjectilesExpireAfterLifetime,
    ProjectilesDamageEnemies,
    ProjectilesDespawnOnHit,
    ProjectileImpactAnimationBeforeDespawn,
    SpawnersSpawnPrefabs,
    DoorsChangeMaps,
    PlayerActivatesCheckpoints,
    RespawnAtCheckpoint,
    CameraFollowsPlayer,
    PauseAndReset,
    ShowBasicUi,
    ShowScore,
    ShowEnemyCount,
    ShowPlayerHealth,
    ShowMenu,
    ShowPauseMenu,
    ShowGameOverPanel,
    ShowWinPanel,
    WinWhenAllPickupsCollected,
    WinWhenAllEnemiesDead,
    AnimateEnemiesByMovement,
    AnimatePlayerDirectionally,
    AnimateEnemiesDirectionally,
    AnimateAttacksDirectionally,
    DeadEnemiesPlayDeathAnimation,
    DeadEnemiesDespawnAfterAnimation,
}

#[cfg(test)]
fn load_beginner_game_text(
    game: &mut GameApp<'_>,
    source: &str,
    label: &str,
) -> Result<TopDownControls> {
    load_beginner_game_text_with_base(game, source, label, None)
}

#[cfg(test)]
fn load_beginner_game_text_with_base(
    game: &mut GameApp<'_>,
    source: &str,
    label: &str,
    asset_base: Option<&Path>,
) -> Result<TopDownControls> {
    let file = parse_beginner_game_source(source, label)?;
    validate_file_with_base(&file, label, asset_base)?;
    build_beginner_game_file(game, file, label, asset_base)
}

struct LoadedBeginnerGameFile {
    file: BeginnerGameFile,
    label: String,
    full_path: PathBuf,
}

fn read_beginner_game_file(path: impl AsRef<Path>) -> Result<LoadedBeginnerGameFile> {
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
    let label = requested.display().to_string();
    let file = parse_beginner_game_source(&source, &label)?;
    validate_file_with_base(&file, &label, full_path.parent())?;
    Ok(LoadedBeginnerGameFile {
        file,
        label,
        full_path,
    })
}

fn parse_beginner_game_source(source: &str, label: &str) -> Result<BeginnerGameFile> {
    ron::from_str(source).map_err(|error| {
        anyhow!(
            "beginner game file '{label}' is not valid RON: {error}\n\nUse controls like TopDown and rules like TopDownControls, PlayerCollectsPickups, ShowScore. They are case-sensitive."
        )
    })
}

fn build_beginner_game_file(
    game: &mut GameApp<'_>,
    file: BeginnerGameFile,
    label: &str,
    asset_base: Option<&Path>,
) -> Result<TopDownControls> {
    let runtime_config = BeginnerRuntimeConfig::from_file(&file);
    game.startup(move |game: &mut StartupGameCtx<'_, '_>| {
        game.insert_resource(runtime_config.clone());
        game.insert_resource(BeginnerRuleUiText::default());
        Ok(())
    });

    let mut asset_author = game.asset_bag();
    for key in &file.assets.textures {
        asset_author = match conventional_asset_path(asset_base, "textures", key, &["png"]) {
            Some(path) => asset_author.texture(key.clone(), path)?,
            None => asset_author.texture_auto(key.clone())?,
        };
    }
    for key in &file.assets.sounds {
        asset_author =
            match conventional_asset_path(asset_base, "sounds", key, &["wav", "ogg", "mp3"]) {
                Some(path) => asset_author.sound(key.clone(), path)?,
                None => asset_author.sound_auto(key.clone())?,
            };
    }
    for key in &file.assets.music {
        asset_author =
            match conventional_asset_path(asset_base, "music", key, &["wav", "ogg", "mp3"]) {
                Some(path) => asset_author.music(key.clone(), path)?,
                None => asset_author.music_auto(key.clone())?,
            };
    }
    for key in &file.assets.animation_sheets {
        asset_author = match asset_path(asset_base, &format!("animations/{key}.ron")) {
            Some(path) => asset_author.spritesheet_from_meta(key.clone(), path)?,
            None => asset_author.animation_sheet_auto(key.clone())?,
        };
    }
    let assets = asset_author.build();

    let controls = match file.controls.kind(label)? {
        BeginnerControlsKind::TopDown => game.input(|input| input.top_down_controls())?,
    };

    for prefab in file.prefabs {
        build_prefab(game, &assets, prefab, controls)?;
    }

    for map in file.maps {
        build_map(game, map, asset_base);
    }

    if let Some(scene_flow) = file.scene_flow {
        build_scene_flow(game, scene_flow, controls);
    }

    build_audio(game, file.audio);
    build_rule_ui_text(game);
    build_actions(game, file.actions, controls);
    build_custom_rules(game, file.custom_rules);

    let mut rules = game.rules();
    for rule in &file.rules {
        if let Some(kind) = rule.simple_kind(label)? {
            rules = apply_rule(rules, kind, controls);
        }
    }
    rules.build();
    for rule in file.rules {
        if let BeginnerRuleFile::Script(rule) = rule {
            build_script_rule(game, rule, controls);
        }
    }
    Ok(controls)
}

fn ensure_same_list(label: &str, kind: &str, expected: &[String], found: &[String]) -> Result<()> {
    if expected == found {
        return Ok(());
    }
    anyhow::bail!(
        "beginner game file '{label}' changed its {kind} list. F5 data reload can edit existing values and map file paths, but adding, removing, or reordering {kind} requires restarting the game.\nStartup {kind}: [{}]\nCurrent {kind}: [{}]",
        expected.join(", "),
        found.join(", "),
    )
}

fn ensure_same_values<T>(label: &str, kind: &str, expected: &T, found: &T) -> Result<()>
where
    T: PartialEq,
{
    if expected == found {
        return Ok(());
    }
    anyhow::bail!(
        "beginner game file '{label}' changed its {kind}. F5 data reload can edit existing prefab values, custom countdown rule details, and map file paths, but changing {kind} requires restarting the game.",
    )
}

fn build_prefab(
    game: &mut GameApp<'_>,
    assets: &crate::assets::AssetBag,
    prefab: BeginnerPrefabFile,
    controls: TopDownControls,
) -> Result<()> {
    match prefab {
        BeginnerPrefabFile::Player(player) => {
            let mut author = game
                .player_prefab(player.name)
                .sprite(player.sprite)
                .moves_with(controls.movement, player.speed)
                .health(player.health);
            if let Some(sheet) = player.animation_sheet {
                author = author.animation_sheet(assets.animation_sheet_result(&sheet)?);
            }
            if let Some(melee) = player.melee {
                author = author.melee(melee.range, melee.damage);
            }
            for tag in player.tags {
                author = author.tag(tag);
            }
            for (key, value) in player.data {
                author = author.data(key, value);
            }
            author.build()?;
        }
        BeginnerPrefabFile::Enemy(enemy) => {
            let mut author = game
                .enemy_prefab(enemy.name)
                .sprite(enemy.sprite)
                .speed(enemy.speed)
                .health(enemy.health);
            if let Some(sheet) = enemy.animation_sheet {
                author = author.animation_sheet(assets.animation_sheet_result(&sheet)?);
            }
            if enemy.chase_player {
                author = author.chases_player();
            }
            if let Some(melee) = enemy.melee {
                author = author.melee(melee.range, melee.damage);
            }
            if let Some(drop) = enemy.drops {
                author = author.drops(drop);
            }
            if let Some(chance) = enemy.drop_chance {
                author = author.drop_chance(chance);
            }
            if enemy.despawn_after_death_animation {
                author = author.despawn_after_death_animation();
            }
            for tag in enemy.tags {
                author = author.tag(tag);
            }
            for (key, value) in enemy.data {
                author = author.data(key, value);
            }
            author.build()?;
        }
        BeginnerPrefabFile::Pickup(pickup) => {
            let mut author = game
                .pickup_prefab(pickup.name)
                .sprite(pickup.sprite)
                .score(pickup.score);
            if let Some(heal) = pickup.heal_player {
                author = author.heal_player(heal);
            }
            if let Some(sound) = pickup.sound {
                author = author.play_sound(sound);
            }
            if pickup.despawn_on_collect {
                author = author.despawn_on_collect();
            }
            for tag in pickup.tags {
                author = author.tag(tag);
            }
            for (key, value) in pickup.data {
                author = author.data(key, value);
            }
            author.build()?;
        }
        BeginnerPrefabFile::Door(door) => {
            let mut author = game.door_prefab(door.name).sprite(door.sprite);
            author = match door.action {
                DoorActionFile::ChangeMap(map) => author.change_map(map),
                DoorActionFile::ChangeScene(scene) => author.change_scene(scene),
                DoorActionFile::RestartLevel => author.restart_level(),
            };
            if door.requires_all_enemies_dead {
                author = author.requires_all_enemies_dead();
            }
            for tag in door.tags {
                author = author.tag(tag);
            }
            for (key, value) in door.data {
                author = author.data(key, value);
            }
            author.build()?;
        }
        BeginnerPrefabFile::Projectile(projectile) => {
            let mut author = game
                .projectile_prefab(projectile.name)
                .sprite(projectile.sprite)
                .damage(projectile.damage)
                .speed(projectile.speed)
                .lifetime(projectile.lifetime);
            if let Some(sheet) = projectile.animation_sheet {
                author = author.animation_sheet(assets.animation_sheet_result(&sheet)?);
            }
            if projectile.despawn_on_hit {
                author = author.despawn_on_hit();
            }
            for tag in projectile.tags {
                author = author.tag(tag);
            }
            for (key, value) in projectile.data {
                author = author.data(key, value);
            }
            author.build()?;
        }
        BeginnerPrefabFile::Spawner(spawner) => {
            let mut author = game
                .spawner_prefab(spawner.name)
                .spawn(spawner.spawn)
                .every_seconds(spawner.every_seconds);
            if let Some(max_alive) = spawner.max_alive {
                author = author.max_alive(max_alive);
            }
            author = match spawner.placement {
                SpawnPlacementFile::AtSpawner => author.at_spawner(),
                SpawnPlacementFile::NearPlayer(radius) => author.near_player(radius),
                SpawnPlacementFile::AtFirstFloor => author.at_first_floor(),
            };
            author.build()?;
        }
        BeginnerPrefabFile::Trigger(trigger) => {
            let mut author = game
                .trigger_prefab(trigger.name)
                .size(vec2(trigger.size.0, trigger.size.1));
            if let Some(texture) = trigger.visible_debug {
                author = author.visible_debug(texture);
            }
            for tag in trigger.tags {
                author = author.tag(tag);
            }
            for (key, value) in trigger.data {
                author = author.data(key, value);
            }
            author.build()?;
        }
        BeginnerPrefabFile::Checkpoint(checkpoint) => {
            let mut author = game
                .checkpoint_prefab(checkpoint.name)
                .sprite(checkpoint.sprite)
                .size(vec2(checkpoint.size.0, checkpoint.size.1));
            for tag in checkpoint.tags {
                author = author.tag(tag);
            }
            for (key, value) in checkpoint.data {
                author = author.data(key, value);
            }
            author.build()?;
        }
    }
    Ok(())
}

fn build_map(game: &mut GameApp<'_>, map: BeginnerMapFile, asset_base: Option<&Path>) {
    match map {
        BeginnerMapFile::TextMap(map) => {
            let path = asset_path(asset_base, &map.path).unwrap_or(map.path);
            let mut author = game
                .map_from_text(map.name.as_str(), path)
                .tile_size(map.tile_size)
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
        BeginnerMapFile::TextMapAuto(map) => {
            let path = asset_path(asset_base, &format!("maps/{}.txt", map.name));
            let mut author = game
                .map_from_text(
                    map.name.as_str(),
                    path.unwrap_or_else(|| format!("maps/{}.txt", map.name)),
                )
                .tile_size(map.tile_size)
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
        BeginnerMapFile::Tiled(map) => {
            let path = asset_path(asset_base, &map.path).unwrap_or(map.path);
            let mut author = game
                .map_from_tiled(map.name.as_str(), path)
                .simple_theme(map.theme.0.as_str(), map.theme.1.as_str());
            for (object, prefab) in map.objects {
                author = author.object(object, prefab);
            }
            if map.start {
                author.start();
            } else {
                author.finish();
            }
        }
        BeginnerMapFile::Ldtk(map) => {
            let path = asset_path(asset_base, &map.path).unwrap_or(map.path);
            let mut author = game
                .map_from_ldtk(map.name.as_str(), path)
                .level(map.level)
                .simple_theme(map.theme.0.as_str(), map.theme.1.as_str());
            for (entity, prefab) in map.entities {
                author = author.entity(entity, prefab);
            }
            if map.start {
                author.start();
            } else {
                author.finish();
            }
        }
    }
}

fn build_scene_flow(game: &mut GameApp<'_>, flow: SceneFlowFile, controls: TopDownControls) {
    let mut author = game.use_simple_scene_flow();
    if let Some(menu) = flow.menu {
        author = author.menu(menu);
    }
    if let Some(game_scene) = flow.game {
        author = author.game(game_scene);
    }
    if let Some(game_over) = flow.game_over {
        author = author.game_over(game_over);
    }
    if let Some(win) = flow.win {
        author = author.win(win);
    }
    if let Some(text) = flow.menu_text {
        author = author.menu_text(text);
    }
    if let Some(button) = flow.menu_button {
        author = author.menu_button(button.label, button.map);
    }
    if let Some(text) = flow.game_over_text {
        author = author.game_over_text(text);
    }
    if let Some(button) = flow.game_over_button {
        author = author.game_over_button(button);
    }
    if let Some(text) = flow.win_text {
        author = author.win_text(text);
    }
    if let Some(button) = flow.win_button {
        author = author.win_button(button);
    }
    if let Some(action) = flow.start_on {
        author = author.start_on(action.resolve(controls));
    }
    if let Some(action) = flow.restart_on {
        author = author.restart_on(action.resolve(controls));
    }
    match flow.win_condition {
        Some(WinConditionFile::AllPickupsCollected) => {
            author = author.win_when_all_pickups_collected();
        }
        Some(WinConditionFile::AllEnemiesDead) => {
            author = author.win_when_all_enemies_dead();
        }
        None => {}
    }
    author.build();
}

fn build_audio(game: &mut GameApp<'_>, audio: AudioFile) {
    let initial_audio = audio;
    let mut state = RuntimeAudioState::default();
    game.update(move |game: &mut GameCtx<'_, '_>, _dt| {
        let audio = game
            .resource::<BeginnerRuntimeConfig>()
            .map(|config| config.audio().clone())
            .unwrap_or_else(|| initial_audio.clone());
        apply_runtime_audio(game, &audio, &mut state);
    });
}

fn build_rule_ui_text(game: &mut GameApp<'_>) {
    game.ui(|game: &mut GameCtx<'_, '_>, _dt| {
        let lines = game
            .resource::<BeginnerRuleUiText>()
            .map(|text| text.lines.clone())
            .unwrap_or_default();
        let Some((first, rest)) = lines.split_first() else {
            return;
        };
        let mut panel = game.ui().panel(first);
        for line in rest {
            panel = panel.line(line);
        }
        panel.center();
    });
}

#[derive(Clone, Debug, Default, PartialEq)]
struct RuntimeAudioState {
    volumes: Option<(Option<f32>, Option<f32>, Option<f32>)>,
    active_music: Option<RuntimeMusicPlayback>,
}

#[derive(Clone, Debug, PartialEq)]
struct RuntimeMusicPlayback {
    scene: String,
    playback: MusicPlaybackFile,
}

fn apply_runtime_audio(
    game: &mut GameCtx<'_, '_>,
    audio: &AudioFile,
    state: &mut RuntimeAudioState,
) {
    let volumes = (audio.master_volume, audio.music_volume, audio.sfx_volume);
    if state.volumes != Some(volumes) {
        if let Some(volume) = audio.master_volume {
            game.audio().set_master_volume(volume);
        }
        if let Some(volume) = audio.music_volume {
            game.audio().set_music_volume(volume);
        }
        if let Some(volume) = audio.sfx_volume {
            game.audio().set_sfx_volume(volume);
        }
        state.volumes = Some(volumes);
    }

    let Some(scene) = game.current_scene_name() else {
        return;
    };
    let Some(playback) = audio.music_on_scene.get(&scene).cloned() else {
        return;
    };
    let requested = RuntimeMusicPlayback { scene, playback };
    if state.active_music.as_ref() == Some(&requested) {
        return;
    }

    let music = game
        .audio()
        .play_music(&requested.playback.track)
        .volume(requested.playback.volume);
    if let Some(fade) = requested.playback.fade_in {
        music.fade_in(fade);
    }
    state.active_music = Some(requested);
}

fn build_actions(
    game: &mut GameApp<'_>,
    actions: Vec<BeginnerActionFile>,
    controls: TopDownControls,
) {
    for (index, action) in actions.into_iter().enumerate() {
        match action {
            BeginnerActionFile::PlayerShoots(shoot) => {
                register_runtime_player_shoots_action(game, index, shoot, controls);
            }
        }
    }
}

fn register_runtime_player_shoots_action(
    game: &mut GameApp<'_>,
    index: usize,
    initial: PlayerShootsFile,
    controls: TopDownControls,
) {
    let action = initial.action.resolve(controls);
    let mut cooldown: f32 = 0.0;
    game.fixed(move |game: &mut GameCtx<'_, '_>, dt: f32| {
        cooldown = (cooldown - dt).max(0.0);
        let shoot = game
            .resource::<BeginnerRuntimeConfig>()
            .and_then(|config| config.player_shoots_action(index))
            .cloned()
            .unwrap_or_else(|| initial.clone());
        if cooldown == 0.0 && game.pressed(action) {
            cooldown = shoot.cooldown.max(0.0);
            fire_runtime_player_shot(game, &shoot);
        }
    });
}

fn fire_runtime_player_shot(game: &mut GameCtx<'_, '_>, shoot: &PlayerShootsFile) {
    let fired = match shoot.direction {
        ShotDirectionFile::TowardsMouse => {
            game.player().shoot(shoot.prefab.clone()).towards_mouse()
        }
        ShotDirectionFile::Right => game.player().shoot(shoot.prefab.clone()).right(),
        ShotDirectionFile::Left => game.player().shoot(shoot.prefab.clone()).left(),
        ShotDirectionFile::Up => game.player().shoot(shoot.prefab.clone()).up(),
        ShotDirectionFile::Down => game.player().shoot(shoot.prefab.clone()).down(),
    };
    if let Some(sound) = &shoot.sound {
        fired.play_sound_named(sound);
    }
}

fn build_custom_rules(game: &mut GameApp<'_>, custom_rules: Vec<CustomRuleFile>) {
    for custom_rule in custom_rules {
        match custom_rule {
            CustomRuleFile::Countdown(rule) => {
                register_runtime_countdown_rule(game, rule.name);
            }
        }
    }
}

#[derive(Default)]
struct RuleConditionRuntime {
    elapsed: f32,
    fired: bool,
}

fn build_script_rule(
    game: &mut GameApp<'_>,
    rule: BeginnerScriptRuleFile,
    controls: TopDownControls,
) {
    match rule {
        BeginnerScriptRuleFile::When { condition, effects } => {
            let mut runtime = RuleConditionRuntime::default();
            game.every_tick(move |game, dt| {
                let active = script_condition_active(game, &condition, controls, dt, &mut runtime);
                if active && !runtime.fired {
                    apply_game_rule_effects(game, &effects);
                    runtime.fired = true;
                } else if !active {
                    runtime.fired = false;
                }
            });
        }
        BeginnerScriptRuleFile::OnEnemyDeath { prefab, effects } => {
            game.on_enemy_death_event(move |event| {
                let matches = {
                    let enemy = event.enemy();
                    enemy.is_prefab(&prefab)
                };
                if matches {
                    apply_enemy_death_rule_effects(event, &effects);
                }
            });
        }
        BeginnerScriptRuleFile::EverySeconds { seconds, effects } => {
            game.every_seconds(seconds, move |game| {
                apply_game_rule_effects(game, &effects);
            });
        }
        BeginnerScriptRuleFile::OnScoreReaches { score, effects } => {
            game.on_score_reaches(score, move |game| {
                apply_game_rule_effects(game, &effects);
            });
        }
    }
}

fn script_condition_active(
    game: &mut BeginnerGame<'_, '_, '_>,
    condition: &RuleConditionFile,
    controls: TopDownControls,
    dt: f32,
    runtime: &mut RuleConditionRuntime,
) -> bool {
    match condition {
        RuleConditionFile::AllEnemiesDead => game.enemies().alive().count() == 0,
        RuleConditionFile::AllPickupsCollected => game.pickups().alive().count() == 0,
        RuleConditionFile::ScoreAtLeast(score) => game.score().value() >= *score,
        RuleConditionFile::PlayerHealthBelow(health) => game
            .player()
            .health()
            .is_some_and(|current| current < *health),
        RuleConditionFile::TimerReached { seconds, .. } => {
            runtime.elapsed += dt.max(0.0);
            runtime.elapsed >= seconds.max(0.0)
        }
        RuleConditionFile::MapIs(map) => game.current_map_name().as_deref() == Some(map.as_str()),
        RuleConditionFile::SceneIs(scene) => {
            game.current_scene_name().as_deref() == Some(scene.as_str())
        }
        RuleConditionFile::TagCountZero(tag) => game.actors_tagged(tag).alive().count() == 0,
        RuleConditionFile::ActionPressed(action) => game.pressed(action.resolve(controls)),
    }
}

fn apply_game_rule_effects(game: &mut BeginnerGame<'_, '_, '_>, effects: &[RuleEffectFile]) {
    for effect in effects {
        match effect {
            RuleEffectFile::AddScore(amount) => game.score().add(*amount),
            RuleEffectFile::SetScore(score) => game.score().set(*score),
            RuleEffectFile::DamageTagged { tag, amount, .. } => {
                game.actors_tagged(tag).damage(*amount);
            }
            RuleEffectFile::DamagePlayer { amount, .. } => {
                game.player().damage(*amount);
            }
            RuleEffectFile::DespawnSelf => {}
            RuleEffectFile::PlaySound(key) => game.play_sound_named(key),
            RuleEffectFile::PlayMusic(key) => game.play_music_named(key),
            RuleEffectFile::StopMusic => game.audio().stop_music(),
            RuleEffectFile::SpawnPrefab(prefab) => {
                game.spawn(prefab.clone()).at_first_floor();
            }
            RuleEffectFile::SpawnNearPlayer { prefab, radius } => {
                game.spawn(prefab.clone()).near_player(*radius);
            }
            RuleEffectFile::ChangeScene(scene) => game.change_scene_or_log(scene),
            RuleEffectFile::ChangeMap(map) => game.change_map_or_log(map),
            RuleEffectFile::RestartCurrentMap => game.restart_current_map_or_log(),
            RuleEffectFile::ShowUiText(text) => game.show_rule_text(text),
            RuleEffectFile::HealPlayer(amount) => {
                game.player().heal(*amount);
            }
            RuleEffectFile::SetData { tag, key, value } => {
                game.actors_tagged(tag).set_data(key, *value);
            }
            RuleEffectFile::DespawnTagged(tag) => {
                game.actors_tagged(tag).despawn();
            }
        }
    }
}

fn apply_enemy_death_rule_effects(
    event: &mut EnemyDeathEvent<'_, '_, '_>,
    effects: &[RuleEffectFile],
) {
    let position = event.enemy_position();
    for effect in effects {
        match effect {
            RuleEffectFile::AddScore(amount) => event.score().add(*amount),
            RuleEffectFile::SetScore(score) => event.score().set(*score),
            RuleEffectFile::DespawnSelf => {
                event.enemy().despawn();
            }
            RuleEffectFile::PlaySound(key) => event.play_sound(key),
            RuleEffectFile::SpawnPrefab(prefab) => {
                if let Some(position) = position {
                    event.spawn(prefab.clone()).at_world(position);
                }
            }
            RuleEffectFile::SpawnNearPlayer { prefab, radius } => {
                event.spawn(prefab.clone()).near_player(*radius);
            }
            RuleEffectFile::ChangeScene(scene) => event.change_scene(scene),
            RuleEffectFile::ChangeMap(map) => event.change_map(map),
            RuleEffectFile::DamageTagged { .. }
            | RuleEffectFile::DamagePlayer { .. }
            | RuleEffectFile::PlayMusic(_)
            | RuleEffectFile::StopMusic
            | RuleEffectFile::RestartCurrentMap
            | RuleEffectFile::ShowUiText(_)
            | RuleEffectFile::HealPlayer(_)
            | RuleEffectFile::SetData { .. }
            | RuleEffectFile::DespawnTagged(_) => {}
        }
    }
}

fn apply_rule<'a, 'app>(
    rules: RulesAuthor<'a, 'app>,
    rule: BeginnerRuleKind,
    controls: TopDownControls,
) -> RulesAuthor<'a, 'app> {
    match rule {
        BeginnerRuleKind::TopDownControls => rules.top_down_controls(controls),
        BeginnerRuleKind::PlayerCollectsPickups => rules.player_collects_pickups(),
        BeginnerRuleKind::EnemiesDamagePlayer => rules.enemies_damage_player(),
        BeginnerRuleKind::DeadEnemiesDespawn => rules.dead_enemies_despawn(),
        BeginnerRuleKind::EnemyDrops => rules.enemy_drops(),
        BeginnerRuleKind::Projectiles => rules.projectiles(),
        BeginnerRuleKind::ProjectilesMove => rules.projectiles_move(),
        BeginnerRuleKind::ProjectilesExpireAfterLifetime => {
            rules.projectiles_expire_after_lifetime()
        }
        BeginnerRuleKind::ProjectilesDamageEnemies => rules.projectiles_damage_enemies(),
        BeginnerRuleKind::ProjectilesDespawnOnHit => rules.projectiles_despawn_on_hit(),
        BeginnerRuleKind::ProjectileImpactAnimationBeforeDespawn => {
            rules.projectile_impact_animation_before_despawn()
        }
        BeginnerRuleKind::SpawnersSpawnPrefabs => rules.spawners_spawn_prefabs(),
        BeginnerRuleKind::DoorsChangeMaps => rules.doors_change_maps(),
        BeginnerRuleKind::PlayerActivatesCheckpoints => rules.player_activates_checkpoints(),
        BeginnerRuleKind::RespawnAtCheckpoint => rules.respawn_at_checkpoint(),
        BeginnerRuleKind::CameraFollowsPlayer => rules.camera_follows_player(),
        BeginnerRuleKind::PauseAndReset => rules.pause_and_reset(),
        BeginnerRuleKind::ShowBasicUi => rules.show_basic_ui(),
        BeginnerRuleKind::ShowScore => rules.show_score(),
        BeginnerRuleKind::ShowEnemyCount => rules.show_enemy_count(),
        BeginnerRuleKind::ShowPlayerHealth => rules.show_player_health(),
        BeginnerRuleKind::ShowMenu => rules.show_menu(),
        BeginnerRuleKind::ShowPauseMenu => rules.show_pause_menu(),
        BeginnerRuleKind::ShowGameOverPanel => rules.show_game_over_panel(),
        BeginnerRuleKind::ShowWinPanel => rules.show_win_panel(),
        BeginnerRuleKind::WinWhenAllPickupsCollected => rules.win_when_all_pickups_collected(),
        BeginnerRuleKind::WinWhenAllEnemiesDead => rules.win_when_all_enemies_dead(),
        BeginnerRuleKind::AnimateEnemiesByMovement => rules.animate_enemies_by_movement(),
        BeginnerRuleKind::AnimatePlayerDirectionally => rules.animate_player_directionally(),
        BeginnerRuleKind::AnimateEnemiesDirectionally => rules.animate_enemies_directionally(),
        BeginnerRuleKind::AnimateAttacksDirectionally => rules.animate_attacks_directionally(),
        BeginnerRuleKind::DeadEnemiesPlayDeathAnimation => {
            rules.dead_enemies_play_death_animation()
        }
        BeginnerRuleKind::DeadEnemiesDespawnAfterAnimation => {
            rules.dead_enemies_despawn_after_animation()
        }
    }
}

#[cfg(test)]
fn validate_file(file: &BeginnerGameFile, label: &str) -> Result<()> {
    validate_file_with_base(file, label, None)
}

fn validate_file_with_base(
    file: &BeginnerGameFile,
    label: &str,
    asset_base: Option<&Path>,
) -> Result<()> {
    if file.version != 1 {
        anyhow::bail!(
            "unsupported beginner game file version {}. Supported version: 1",
            file.version
        );
    }
    file.controls.kind(label)?;

    let textures = file
        .assets
        .textures
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let sounds = file
        .assets
        .sounds
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let music = file
        .assets
        .music
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let animation_sheets = file
        .assets
        .animation_sheets
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    reject_duplicates(label, "texture asset", &textures)?;
    reject_duplicates(label, "sound asset", &sounds)?;
    reject_duplicates(label, "music asset", &music)?;
    reject_duplicates(label, "animation sheet asset", &animation_sheets)?;

    let prefab_names = file
        .prefabs
        .iter()
        .map(BeginnerPrefabFile::name)
        .collect::<Vec<_>>();
    reject_duplicates(label, "prefab", &prefab_names)?;
    let custom_rule_names = file
        .custom_rules
        .iter()
        .map(CustomRuleFile::name)
        .collect::<Vec<_>>();
    reject_duplicates(label, "custom rule", &custom_rule_names)?;
    let map_names = file
        .maps
        .iter()
        .map(BeginnerMapFile::name)
        .collect::<Vec<_>>();
    reject_duplicates(label, "map", &map_names)?;

    if !file.maps.is_empty() {
        let starts = file.maps.iter().filter(|map| map.start()).count();
        if starts != 1 {
            anyhow::bail!(
                "beginner game file '{label}' must mark exactly one map with start: true; found {starts}"
            );
        }
    }

    let tags = file
        .prefabs
        .iter()
        .flat_map(BeginnerPrefabFile::tags)
        .collect::<Vec<_>>();
    let prefab_data = build_prefab_data_index(&file.prefabs);
    let scene_names = scene_names(file);
    let scene_name_refs = scene_names.iter().map(String::as_str).collect::<Vec<_>>();

    for prefab in &file.prefabs {
        for (owner, texture) in prefab.texture_refs() {
            require_known(label, "texture", owner, texture, &textures)?;
        }
        for (owner, sound) in prefab.sound_refs() {
            require_known(label, "sound", owner, sound, &sounds)?;
        }
        for (owner, sheet) in prefab.animation_sheet_refs() {
            require_known(label, "animation sheet", owner, sheet, &animation_sheets)?;
        }
        for (owner, referenced) in prefab.prefab_refs() {
            require_known(label, "prefab", owner, referenced, &prefab_names)?;
        }
        for (owner, map) in prefab.map_refs() {
            require_known(label, "map", owner, map, &map_names)?;
        }
        for (owner, scene) in prefab.scene_refs() {
            if scene_name_refs.is_empty() {
                anyhow::bail!(
                    "beginner game file '{label}' {owner} changes to scene '{scene}', but no scene_flow declares scenes"
                );
            }
            require_known(label, "scene", owner, scene, &scene_name_refs)?;
        }
        prefab.validate_numbers(label)?;
    }

    for map in &file.maps {
        for (owner, texture) in map.texture_refs() {
            require_known(
                label,
                "texture",
                &format!("map '{owner}'"),
                texture,
                &textures,
            )?;
        }
        for (owner, prefab) in map.prefab_refs() {
            require_known(
                label,
                "prefab",
                &format!("map '{owner}'"),
                prefab,
                &prefab_names,
            )?;
        }
        validate_map_file(label, map, asset_base)?;
    }

    if let Some(flow) = &file.scene_flow {
        validate_scene_flow(label, flow, &map_names)?;
    }
    validate_audio(label, &file.audio, &music, &scene_name_refs)?;
    validate_actions(label, &file.actions, &prefab_names, &sounds)?;
    let names = ValidationNames {
        prefabs: &prefab_names,
        sounds: &sounds,
        music: &music,
        maps: &map_names,
        scenes: &scene_name_refs,
        tags: &tags,
    };
    validate_custom_rules(label, &file.custom_rules, &names, &prefab_data)?;

    for rule in &file.rules {
        rule.identity(label)?;
        if let BeginnerRuleFile::Script(rule) = rule {
            validate_script_rule(label, rule, &names)?;
        }
    }
    validate_rule_combinations(label, &file.rules, &file.prefabs)
}

fn reject_duplicates(label: &str, kind: &str, names: &[&str]) -> Result<()> {
    let unique = names.iter().copied().collect::<BTreeSet<_>>();
    if unique.len() != names.len() {
        anyhow::bail!("beginner game file '{label}' defines duplicate {kind} names");
    }
    Ok(())
}

fn require_known(label: &str, kind: &str, owner: &str, key: &str, known: &[&str]) -> Result<()> {
    if known.contains(&key) {
        return Ok(());
    }
    Err(unknown_reference_error(label, owner, kind, key, known))
}

fn validate_map_file(label: &str, map: &BeginnerMapFile, asset_base: Option<&Path>) -> Result<()> {
    match map {
        BeginnerMapFile::TextMap(map) => {
            validate_text_map_symbols(label, &map.name, &map.path, &map.legend, asset_base)
        }
        BeginnerMapFile::TextMapAuto(map) => {
            let path = format!("maps/{}.txt", map.name);
            validate_text_map_symbols(label, &map.name, &path, &map.legend, asset_base)
        }
        BeginnerMapFile::Tiled(map) => {
            require_asset_file(label, "Tiled map", &map.name, &map.path, asset_base)
        }
        BeginnerMapFile::Ldtk(map) => {
            require_asset_file(label, "LDtk map", &map.name, &map.path, asset_base)
        }
    }
}

fn validate_text_map_symbols(
    label: &str,
    map_name: &str,
    path: &str,
    legend: &BTreeMap<char, String>,
    asset_base: Option<&Path>,
) -> Result<()> {
    let full_path = resolved_asset_file(asset_base, path);
    let source = std::fs::read_to_string(&full_path).with_context(|| {
        format!(
            "beginner game file '{label}' map '{map_name}' could not read text map 'assets/{path}' (looked for '{}')",
            full_path.display()
        )
    })?;
    let known_symbols = legend.keys().copied().collect::<Vec<_>>();
    for (row, line) in source.lines().enumerate() {
        for (col, symbol) in line.trim_end_matches('\r').chars().enumerate() {
            if symbol == '.' || symbol == '#' || legend.contains_key(&symbol) {
                continue;
            }
            anyhow::bail!(
                "beginner game file '{label}' map '{map_name}' has an invalid symbol.\n\n{}",
                bad_map_symbol_error(map_name, symbol, row, col, &known_symbols)
            );
        }
    }
    Ok(())
}

fn require_asset_file(
    label: &str,
    kind: &str,
    owner: &str,
    path: &str,
    asset_base: Option<&Path>,
) -> Result<()> {
    let full_path = resolved_asset_file(asset_base, path);
    if full_path.is_file() {
        return Ok(());
    }
    anyhow::bail!(
        "beginner game file '{label}' {kind} '{owner}' references a missing file.\n\n{}",
        missing_file_error(kind, &format!("assets/{path}"), &full_path)
    )
}

fn asset_path(asset_base: Option<&Path>, relative: &str) -> Option<String> {
    let base = asset_base?;
    let path = Path::new(relative);
    let full_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    };
    Some(full_path.to_string_lossy().into_owned())
}

fn conventional_asset_path(
    asset_base: Option<&Path>,
    folder: &str,
    key: &str,
    extensions: &[&str],
) -> Option<String> {
    let base = asset_base?;
    let first = extensions.first()?;
    let preferred = base.join(folder).join(format!("{key}.{first}"));
    Some(
        extensions
            .iter()
            .map(|extension| base.join(folder).join(format!("{key}.{extension}")))
            .find(|candidate| candidate.is_file())
            .unwrap_or(preferred)
            .to_string_lossy()
            .into_owned(),
    )
}

fn resolved_asset_file(asset_base: Option<&Path>, relative: &str) -> std::path::PathBuf {
    let path = Path::new(relative);
    if path.is_absolute() {
        return path.to_path_buf();
    }
    if let Some(base) = asset_base {
        return base.join(path);
    }
    beginner_asset_path(relative)
}

fn validate_scene_flow(label: &str, flow: &SceneFlowFile, map_names: &[&str]) -> Result<()> {
    if let Some(game_map) = flow.game.as_deref() {
        require_known(label, "map", "scene_flow.game", game_map, map_names)?;
    }
    if let Some(game_over) = flow.game_over.as_deref() {
        require_known(label, "map", "scene_flow.game_over", game_over, map_names)?;
    }
    if let Some(win) = flow.win.as_deref() {
        require_known(label, "map", "scene_flow.win", win, map_names)?;
    }
    if let Some(button) = &flow.menu_button {
        require_known(
            label,
            "map",
            "scene_flow.menu_button",
            &button.map,
            map_names,
        )?;
    }
    Ok(())
}

fn validate_audio(label: &str, audio: &AudioFile, music: &[&str], scenes: &[&str]) -> Result<()> {
    if !audio.music_on_scene.is_empty() && scenes.is_empty() {
        anyhow::bail!(
            "beginner game file '{label}' audio.music_on_scene needs scene_flow so scene names exist"
        );
    }
    for (scene, playback) in &audio.music_on_scene {
        require_known(label, "scene", "audio.music_on_scene", scene, scenes)?;
        require_known(
            label,
            "music",
            &format!("audio.music_on_scene '{scene}'"),
            &playback.track,
            music,
        )?;
        if !playback.volume.is_finite() || playback.volume < 0.0 {
            anyhow::bail!(
                "beginner game file '{label}' audio.music_on_scene '{scene}' has invalid volume {}; use a finite non-negative number",
                playback.volume
            );
        }
    }
    for (field, volume) in [
        ("master_volume", audio.master_volume),
        ("music_volume", audio.music_volume),
        ("sfx_volume", audio.sfx_volume),
    ] {
        if let Some(volume) = volume
            && (!volume.is_finite() || volume < 0.0)
        {
            anyhow::bail!(
                "beginner game file '{label}' audio.{field} has invalid volume {volume}; use a finite non-negative number"
            );
        }
    }
    Ok(())
}

fn validate_actions(
    label: &str,
    actions: &[BeginnerActionFile],
    prefabs: &[&str],
    sounds: &[&str],
) -> Result<()> {
    for action in actions {
        match action {
            BeginnerActionFile::PlayerShoots(shoot) => {
                require_known(
                    label,
                    "prefab",
                    "action PlayerShoots",
                    &shoot.prefab,
                    prefabs,
                )?;
                if let Some(sound) = shoot.sound.as_deref() {
                    require_known(label, "sound", "action PlayerShoots", sound, sounds)?;
                }
                if !shoot.cooldown.is_finite() || shoot.cooldown < 0.0 {
                    anyhow::bail!(
                        "beginner game file '{label}' action PlayerShoots has invalid cooldown {}; use a finite non-negative number",
                        shoot.cooldown
                    );
                }
            }
        }
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct ValidationNames<'a> {
    prefabs: &'a [&'a str],
    sounds: &'a [&'a str],
    music: &'a [&'a str],
    maps: &'a [&'a str],
    scenes: &'a [&'a str],
    tags: &'a [&'a str],
}

#[derive(Default)]
struct PrefabDataIndex {
    tags: BTreeSet<String>,
    tag_to_data_keys: BTreeMap<String, BTreeSet<String>>,
    tag_to_prefabs: BTreeMap<String, Vec<String>>,
}

fn build_prefab_data_index(prefabs: &[BeginnerPrefabFile]) -> PrefabDataIndex {
    let mut index = PrefabDataIndex::default();
    for prefab in prefabs {
        let name = prefab.name();
        let data_keys = prefab.data_keys();
        for tag in prefab.tags() {
            index.tags.insert(tag.to_owned());
            index
                .tag_to_prefabs
                .entry(tag.to_owned())
                .or_default()
                .push(name.to_owned());
            let keys = index.tag_to_data_keys.entry(tag.to_owned()).or_default();
            keys.extend(data_keys.iter().map(|key| (*key).to_owned()));
        }
    }
    index
}

fn validate_custom_rules(
    label: &str,
    custom_rules: &[CustomRuleFile],
    names: &ValidationNames<'_>,
    prefab_data: &PrefabDataIndex,
) -> Result<()> {
    for custom_rule in custom_rules {
        match custom_rule {
            CustomRuleFile::Countdown(rule) => {
                if rule.tag.trim().is_empty() {
                    anyhow::bail!(
                        "beginner game file '{label}' custom rule '{}' has an empty tag",
                        rule.name
                    );
                }
                if rule.key.trim().is_empty() {
                    anyhow::bail!(
                        "beginner game file '{label}' custom rule '{}' has an empty countdown key",
                        rule.name
                    );
                }
                require_known(
                    label,
                    "tag",
                    &format!("custom rule '{}'", rule.name),
                    &rule.tag,
                    names.tags,
                )?;
                validate_countdown_key_for_tag(label, rule, prefab_data)?;
                for effect in &rule.when_zero {
                    match effect {
                        RuleEffectFile::DamageTagged { tag, radius, .. } => {
                            require_known(
                                label,
                                "tag",
                                &format!("custom rule '{}'", rule.name),
                                tag,
                                names.tags,
                            )?;
                            validate_radius(label, &rule.name, *radius)?;
                        }
                        RuleEffectFile::DamagePlayer { radius, .. } => {
                            validate_radius(label, &rule.name, *radius)?;
                        }
                        RuleEffectFile::AddScore(_) => {}
                        RuleEffectFile::SetScore(_) => {}
                        RuleEffectFile::DespawnSelf => {}
                        RuleEffectFile::PlaySound(sound) => {
                            require_known(
                                label,
                                "sound",
                                &format!("custom rule '{}'", rule.name),
                                sound,
                                names.sounds,
                            )?;
                        }
                        RuleEffectFile::PlayMusic(track) => {
                            require_known(
                                label,
                                "music",
                                &format!("custom rule '{}'", rule.name),
                                track,
                                names.music,
                            )?;
                        }
                        RuleEffectFile::StopMusic => {}
                        RuleEffectFile::SpawnPrefab(prefab) => {
                            require_known(
                                label,
                                "prefab",
                                &format!("custom rule '{}'", rule.name),
                                prefab,
                                names.prefabs,
                            )?;
                        }
                        RuleEffectFile::SpawnNearPlayer { prefab, radius } => {
                            require_known(
                                label,
                                "prefab",
                                &format!("custom rule '{}'", rule.name),
                                prefab,
                                names.prefabs,
                            )?;
                            validate_radius(label, &rule.name, *radius)?;
                        }
                        RuleEffectFile::ChangeScene(scene) => {
                            if names.scenes.is_empty() {
                                anyhow::bail!(
                                    "beginner game file '{label}' custom rule '{}' changes to scene '{scene}', but no scene_flow declares scenes",
                                    rule.name
                                );
                            }
                            require_known(
                                label,
                                "scene",
                                &format!("custom rule '{}'", rule.name),
                                scene,
                                names.scenes,
                            )?;
                        }
                        RuleEffectFile::ChangeMap(map) => {
                            require_known(
                                label,
                                "map",
                                &format!("custom rule '{}'", rule.name),
                                map,
                                names.maps,
                            )?;
                        }
                        RuleEffectFile::RestartCurrentMap => {}
                        RuleEffectFile::ShowUiText(text) => {
                            validate_text(label, &format!("custom rule '{}'", rule.name), text)?;
                        }
                        RuleEffectFile::HealPlayer(_) => {}
                        RuleEffectFile::SetData { tag, key, value } => {
                            require_known(
                                label,
                                "tag",
                                &format!("custom rule '{}'", rule.name),
                                tag,
                                names.tags,
                            )?;
                            validate_data_assignment(
                                label,
                                &format!("custom rule '{}'", rule.name),
                                key,
                                *value,
                            )?;
                        }
                        RuleEffectFile::DespawnTagged(tag) => {
                            require_known(
                                label,
                                "tag",
                                &format!("custom rule '{}'", rule.name),
                                tag,
                                names.tags,
                            )?;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn validate_countdown_key_for_tag(
    label: &str,
    rule: &CountdownRuleFile,
    prefab_data: &PrefabDataIndex,
) -> Result<()> {
    if !prefab_data.tags.contains(&rule.tag) {
        anyhow::bail!(
            "beginner game file '{label}' custom rule '{}' counts down tag '{}', but no prefab declares that tag",
            rule.name,
            rule.tag
        );
    }

    let keys = prefab_data
        .tag_to_data_keys
        .get(&rule.tag)
        .cloned()
        .unwrap_or_default();
    if keys.contains(&rule.key) {
        return Ok(());
    }

    let prefabs = prefab_data
        .tag_to_prefabs
        .get(&rule.tag)
        .cloned()
        .unwrap_or_default();
    let known = keys.iter().map(String::as_str).collect::<Vec<_>>();
    let suggestion = closest_name(&rule.key, known.iter().copied())
        .map(|candidate| format!("\n\nDid you mean '{candidate}'?"))
        .unwrap_or_default();
    let empty_keys_note = if keys.is_empty() {
        format!(
            "\n\nNo prefab tagged '{}' declares any data keys.",
            rule.tag
        )
    } else {
        String::new()
    };

    anyhow::bail!(
        "beginner game file '{label}' custom rule '{}' counts down key '{}' on tag '{}', but no prefab with that tag declares that data key.\n\nPrefabs with tag '{}': {}\nKnown data keys for tag '{}': {}{}{}\n\nFix: add data: {{\"{}\": 3.0}} to one of those prefabs, or change the rule key to one of the known data keys.",
        rule.name,
        rule.key,
        rule.tag,
        rule.tag,
        joined_strings_or_none(&prefabs),
        rule.tag,
        joined_names_or_none(&known),
        empty_keys_note,
        suggestion,
        rule.key,
    );
}

fn joined_strings_or_none(values: &[String]) -> String {
    if values.is_empty() {
        "(none)".to_owned()
    } else {
        values.join(", ")
    }
}

fn joined_names_or_none(values: &[&str]) -> String {
    if values.is_empty() {
        "(none)".to_owned()
    } else {
        values.join(", ")
    }
}

fn validate_radius(label: &str, owner: &str, radius: f32) -> Result<()> {
    if !radius.is_finite() || radius < 0.0 {
        anyhow::bail!(
            "beginner game file '{label}' custom rule '{owner}' has invalid radius {radius}; use a finite non-negative number"
        );
    }
    Ok(())
}

fn validate_text(label: &str, owner: &str, text: &str) -> Result<()> {
    if text.trim().is_empty() {
        anyhow::bail!("beginner game file '{label}' {owner} has empty text");
    }
    Ok(())
}

fn validate_data_assignment(label: &str, owner: &str, key: &str, value: f32) -> Result<()> {
    if key.trim().is_empty() {
        anyhow::bail!("beginner game file '{label}' {owner} has SetData with an empty key");
    }
    if !value.is_finite() {
        anyhow::bail!(
            "beginner game file '{label}' {owner} has invalid SetData value {value}; use a finite number"
        );
    }
    Ok(())
}

fn validate_non_negative(label: &str, owner: &str, field: &str, value: i32) -> Result<()> {
    if value < 0 {
        anyhow::bail!(
            "beginner game file '{label}' {owner} has negative {field} {value}; use zero or a positive number"
        );
    }
    Ok(())
}

fn validate_script_rule(
    label: &str,
    rule: &BeginnerScriptRuleFile,
    names: &ValidationNames<'_>,
) -> Result<()> {
    match rule {
        BeginnerScriptRuleFile::When { condition, effects } => {
            validate_rule_condition(label, "When", condition, names)?;
            validate_script_effects(label, "When", ScriptEffectScope::Game, effects, names)
        }
        BeginnerScriptRuleFile::OnEnemyDeath { prefab, effects } => {
            require_known(label, "prefab", "OnEnemyDeath", prefab, names.prefabs)?;
            validate_script_effects(
                label,
                "OnEnemyDeath",
                ScriptEffectScope::EnemyDeath,
                effects,
                names,
            )
        }
        BeginnerScriptRuleFile::EverySeconds { seconds, effects } => {
            if !seconds.is_finite() || *seconds <= 0.0 {
                anyhow::bail!(
                    "beginner game file '{label}' EverySeconds has invalid seconds {seconds}; use a positive number"
                );
            }
            validate_script_effects(
                label,
                "EverySeconds",
                ScriptEffectScope::Game,
                effects,
                names,
            )
        }
        BeginnerScriptRuleFile::OnScoreReaches { effects, .. } => validate_script_effects(
            label,
            "OnScoreReaches",
            ScriptEffectScope::Game,
            effects,
            names,
        ),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ScriptEffectScope {
    Game,
    EnemyDeath,
}

fn validate_rule_condition(
    label: &str,
    owner: &str,
    condition: &RuleConditionFile,
    names: &ValidationNames<'_>,
) -> Result<()> {
    match condition {
        RuleConditionFile::AllEnemiesDead
        | RuleConditionFile::AllPickupsCollected
        | RuleConditionFile::ScoreAtLeast(_)
        | RuleConditionFile::PlayerHealthBelow(_)
        | RuleConditionFile::ActionPressed(_) => {}
        RuleConditionFile::TimerReached { name, seconds } => {
            if name.trim().is_empty() {
                anyhow::bail!(
                    "beginner game file '{label}' {owner} has TimerReached with an empty name"
                );
            }
            if !seconds.is_finite() || *seconds < 0.0 {
                anyhow::bail!(
                    "beginner game file '{label}' {owner} has invalid TimerReached seconds {seconds}; use a finite non-negative number"
                );
            }
        }
        RuleConditionFile::MapIs(map) => {
            require_known(label, "map", owner, map, names.maps)?;
        }
        RuleConditionFile::SceneIs(scene) => {
            if names.scenes.is_empty() {
                anyhow::bail!(
                    "beginner game file '{label}' {owner} checks scene '{scene}', but no scene_flow declares scenes"
                );
            }
            require_known(label, "scene", owner, scene, names.scenes)?;
        }
        RuleConditionFile::TagCountZero(tag) => {
            require_known(label, "tag", owner, tag, names.tags)?;
        }
    }
    Ok(())
}

fn validate_script_effects(
    label: &str,
    owner: &str,
    scope: ScriptEffectScope,
    effects: &[RuleEffectFile],
    names: &ValidationNames<'_>,
) -> Result<()> {
    for effect in effects {
        match effect {
            RuleEffectFile::AddScore(_)
            | RuleEffectFile::SetScore(_)
            | RuleEffectFile::DespawnSelf
            | RuleEffectFile::StopMusic
            | RuleEffectFile::RestartCurrentMap => {
                validate_game_effect_scope(label, owner, scope, effect)?;
            }
            RuleEffectFile::PlaySound(sound) => {
                validate_game_effect_scope(label, owner, scope, effect)?;
                require_known(label, "sound", owner, sound, names.sounds)?;
            }
            RuleEffectFile::PlayMusic(track) => {
                validate_game_effect_scope(label, owner, scope, effect)?;
                require_known(label, "music", owner, track, names.music)?;
            }
            RuleEffectFile::SpawnPrefab(prefab) => {
                validate_game_effect_scope(label, owner, scope, effect)?;
                require_known(label, "prefab", owner, prefab, names.prefabs)?;
            }
            RuleEffectFile::SpawnNearPlayer { prefab, radius } => {
                validate_game_effect_scope(label, owner, scope, effect)?;
                require_known(label, "prefab", owner, prefab, names.prefabs)?;
                if !radius.is_finite() || *radius < 0.0 {
                    anyhow::bail!(
                        "beginner game file '{label}' {owner} has invalid SpawnNearPlayer radius {radius}; use a finite non-negative number"
                    );
                }
            }
            RuleEffectFile::ChangeScene(scene) => {
                validate_game_effect_scope(label, owner, scope, effect)?;
                if names.scenes.is_empty() {
                    anyhow::bail!(
                        "beginner game file '{label}' {owner} changes to scene '{scene}', but no scene_flow declares scenes"
                    );
                }
                require_known(label, "scene", owner, scene, names.scenes)?;
            }
            RuleEffectFile::ChangeMap(map) => {
                validate_game_effect_scope(label, owner, scope, effect)?;
                require_known(label, "map", owner, map, names.maps)?;
            }
            RuleEffectFile::ShowUiText(text) => {
                validate_game_effect_scope(label, owner, scope, effect)?;
                validate_text(label, owner, text)?;
            }
            RuleEffectFile::DamagePlayer { amount, .. } => {
                validate_game_effect_scope(label, owner, scope, effect)?;
                validate_non_negative(label, owner, "DamagePlayer amount", *amount)?;
            }
            RuleEffectFile::HealPlayer(amount) => {
                validate_game_effect_scope(label, owner, scope, effect)?;
                validate_non_negative(label, owner, "HealPlayer amount", *amount)?;
            }
            RuleEffectFile::SetData { tag, key, value } => {
                validate_game_effect_scope(label, owner, scope, effect)?;
                require_known(label, "tag", owner, tag, names.tags)?;
                validate_data_assignment(label, owner, key, *value)?;
            }
            RuleEffectFile::DespawnTagged(tag) => {
                validate_game_effect_scope(label, owner, scope, effect)?;
                require_known(label, "tag", owner, tag, names.tags)?;
            }
            RuleEffectFile::DamageTagged { .. } => {
                anyhow::bail!(
                    "beginner game file '{label}' {owner} uses the countdown-only DamageTagged effect. Use DamageTagged inside custom_rules countdowns, or use script effects like AddScore, PlaySound, PlayMusic, SpawnPrefab, SpawnNearPlayer, ChangeScene, ChangeMap, DamagePlayer, HealPlayer, SetData, DespawnTagged, and ShowUiText."
                );
            }
        }
    }
    Ok(())
}

fn validate_game_effect_scope(
    label: &str,
    owner: &str,
    scope: ScriptEffectScope,
    effect: &RuleEffectFile,
) -> Result<()> {
    if scope == ScriptEffectScope::Game {
        return Ok(());
    }
    match effect {
        RuleEffectFile::AddScore(_)
        | RuleEffectFile::SetScore(_)
        | RuleEffectFile::DespawnSelf
        | RuleEffectFile::PlaySound(_)
        | RuleEffectFile::SpawnPrefab(_)
        | RuleEffectFile::SpawnNearPlayer { .. }
        | RuleEffectFile::ChangeScene(_)
        | RuleEffectFile::ChangeMap(_) => Ok(()),
        _ => {
            anyhow::bail!(
                "beginner game file '{label}' {owner} uses an effect that only works in When, EverySeconds, or OnScoreReaches rules"
            );
        }
    }
}

fn validate_rule_combinations(
    label: &str,
    rules: &[BeginnerRuleFile],
    prefabs: &[BeginnerPrefabFile],
) -> Result<()> {
    let kinds = rules
        .iter()
        .filter_map(|rule| rule.simple_kind(label).ok().flatten())
        .collect::<Vec<_>>();
    let has_checkpoint = prefabs
        .iter()
        .any(|prefab| matches!(prefab, BeginnerPrefabFile::Checkpoint(_)));
    let has_projectile = prefabs
        .iter()
        .any(|prefab| matches!(prefab, BeginnerPrefabFile::Projectile(_)));
    if !has_checkpoint
        && kinds.iter().any(|kind| {
            matches!(
                kind,
                BeginnerRuleKind::PlayerActivatesCheckpoints
                    | BeginnerRuleKind::RespawnAtCheckpoint
            )
        })
    {
        anyhow::bail!(
            "beginner game file '{label}' enables checkpoint rules but defines no Checkpoint prefab"
        );
    }
    if !has_projectile
        && kinds.iter().any(|kind| {
            matches!(
                kind,
                BeginnerRuleKind::Projectiles
                    | BeginnerRuleKind::ProjectilesMove
                    | BeginnerRuleKind::ProjectilesExpireAfterLifetime
                    | BeginnerRuleKind::ProjectilesDamageEnemies
                    | BeginnerRuleKind::ProjectilesDespawnOnHit
                    | BeginnerRuleKind::ProjectileImpactAnimationBeforeDespawn
            )
        })
    {
        anyhow::bail!(
            "beginner game file '{label}' enables projectile rules but defines no Projectile prefab.\n\nAdd a Projectile prefab such as:\n    Projectile((name: \"bolt\", sprite: \"bolt\"))"
        );
    }
    if kinds.contains(&BeginnerRuleKind::ProjectilesDamageEnemies)
        && !kinds.contains(&BeginnerRuleKind::Projectiles)
        && !kinds.contains(&BeginnerRuleKind::ProjectilesMove)
    {
        return Err(bad_rule_combo_error(
            "ProjectilesDamageEnemies",
            "Projectiles",
        ));
    }
    if kinds.contains(&BeginnerRuleKind::ProjectileImpactAnimationBeforeDespawn)
        && !kinds.contains(&BeginnerRuleKind::ProjectilesDespawnOnHit)
        && !kinds.contains(&BeginnerRuleKind::Projectiles)
    {
        return Err(bad_rule_combo_error(
            "ProjectileImpactAnimationBeforeDespawn",
            "ProjectilesDespawnOnHit",
        ));
    }
    Ok(())
}

fn scene_names(file: &BeginnerGameFile) -> Vec<String> {
    let Some(flow) = &file.scene_flow else {
        return Vec::new();
    };
    let mut names = Vec::new();
    for name in [&flow.menu, &flow.game, &flow.game_over, &flow.win]
        .into_iter()
        .filter_map(Option::as_deref)
    {
        if !names.iter().any(|candidate| candidate == name) {
            names.push(name.to_owned());
        }
    }
    names
}

impl BeginnerControlsFile {
    fn kind(&self, label: &str) -> Result<BeginnerControlsKind> {
        match self {
            Self::Structured(kind) => Ok(*kind),
            Self::Legacy(name) if name == "top_down" || name == "TopDown" => {
                Ok(BeginnerControlsKind::TopDown)
            }
            Self::Legacy(name) => {
                let supported = ["top_down"];
                let suggestion = closest_name(name, supported.into_iter())
                    .map(|candidate| format!(" Did you mean '{candidate}'?"))
                    .unwrap_or_default();
                anyhow::bail!(
                    "beginner game file '{label}' has unsupported controls '{name}'. Supported controls: TopDown or legacy \"top_down\".{suggestion}"
                )
            }
        }
    }
}

impl BeginnerRuleFile {
    fn simple_kind(&self, label: &str) -> Result<Option<BeginnerRuleKind>> {
        match self {
            Self::Structured(kind) => Ok(Some(*kind)),
            Self::Script(_) => Ok(None),
            Self::Legacy(name) => legacy_rule_kind(name, label).map(Some),
        }
    }

    fn identity(&self, label: &str) -> Result<BeginnerRuleIdentity> {
        match self {
            Self::Structured(kind) => Ok(BeginnerRuleIdentity::Simple(*kind)),
            Self::Script(rule) => Ok(BeginnerRuleIdentity::Script(rule.clone())),
            Self::Legacy(name) => legacy_rule_kind(name, label).map(BeginnerRuleIdentity::Simple),
        }
    }
}

fn legacy_rule_kind(name: &str, label: &str) -> Result<BeginnerRuleKind> {
    let kind = match name {
        "top_down_controls" | "TopDownControls" => BeginnerRuleKind::TopDownControls,
        "player_collects_pickups" | "PlayerCollectsPickups" => {
            BeginnerRuleKind::PlayerCollectsPickups
        }
        "enemies_damage_player" | "EnemiesDamagePlayer" => BeginnerRuleKind::EnemiesDamagePlayer,
        "dead_enemies_despawn" | "DeadEnemiesDespawn" => BeginnerRuleKind::DeadEnemiesDespawn,
        "enemy_drops" | "EnemyDrops" => BeginnerRuleKind::EnemyDrops,
        "projectiles" | "Projectiles" => BeginnerRuleKind::Projectiles,
        "projectiles_move" | "ProjectilesMove" => BeginnerRuleKind::ProjectilesMove,
        "projectiles_expire_after_lifetime" | "ProjectilesExpireAfterLifetime" => {
            BeginnerRuleKind::ProjectilesExpireAfterLifetime
        }
        "projectiles_damage_enemies" | "ProjectilesDamageEnemies" => {
            BeginnerRuleKind::ProjectilesDamageEnemies
        }
        "projectiles_despawn_on_hit" | "ProjectilesDespawnOnHit" => {
            BeginnerRuleKind::ProjectilesDespawnOnHit
        }
        "projectile_impact_animation_before_despawn" | "ProjectileImpactAnimationBeforeDespawn" => {
            BeginnerRuleKind::ProjectileImpactAnimationBeforeDespawn
        }
        "spawners_spawn_prefabs" | "SpawnersSpawnPrefabs" => BeginnerRuleKind::SpawnersSpawnPrefabs,
        "doors_change_maps" | "DoorsChangeMaps" => BeginnerRuleKind::DoorsChangeMaps,
        "player_activates_checkpoints" | "PlayerActivatesCheckpoints" => {
            BeginnerRuleKind::PlayerActivatesCheckpoints
        }
        "respawn_at_checkpoint" | "RespawnAtCheckpoint" => BeginnerRuleKind::RespawnAtCheckpoint,
        "camera_follows_player" | "CameraFollowsPlayer" => BeginnerRuleKind::CameraFollowsPlayer,
        "pause_and_reset" | "PauseAndReset" => BeginnerRuleKind::PauseAndReset,
        "show_basic_ui" | "ShowBasicUi" => BeginnerRuleKind::ShowBasicUi,
        "show_score" | "ShowScore" => BeginnerRuleKind::ShowScore,
        "show_enemy_count" | "ShowEnemyCount" => BeginnerRuleKind::ShowEnemyCount,
        "show_player_health" | "ShowPlayerHealth" => BeginnerRuleKind::ShowPlayerHealth,
        "show_menu" | "ShowMenu" => BeginnerRuleKind::ShowMenu,
        "show_pause_menu" | "ShowPauseMenu" => BeginnerRuleKind::ShowPauseMenu,
        "show_game_over_panel" | "ShowGameOverPanel" => BeginnerRuleKind::ShowGameOverPanel,
        "show_win_panel" | "ShowWinPanel" => BeginnerRuleKind::ShowWinPanel,
        "win_when_all_pickups_collected" | "WinWhenAllPickupsCollected" => {
            BeginnerRuleKind::WinWhenAllPickupsCollected
        }
        "win_when_all_enemies_dead" | "WinWhenAllEnemiesDead" => {
            BeginnerRuleKind::WinWhenAllEnemiesDead
        }
        "animate_enemies_by_movement" | "AnimateEnemiesByMovement" => {
            BeginnerRuleKind::AnimateEnemiesByMovement
        }
        "animate_player_directionally" | "AnimatePlayerDirectionally" => {
            BeginnerRuleKind::AnimatePlayerDirectionally
        }
        "animate_enemies_directionally" | "AnimateEnemiesDirectionally" => {
            BeginnerRuleKind::AnimateEnemiesDirectionally
        }
        "animate_attacks_directionally" | "AnimateAttacksDirectionally" => {
            BeginnerRuleKind::AnimateAttacksDirectionally
        }
        "dead_enemies_play_death_animation" | "DeadEnemiesPlayDeathAnimation" => {
            BeginnerRuleKind::DeadEnemiesPlayDeathAnimation
        }
        "dead_enemies_despawn_after_animation" | "DeadEnemiesDespawnAfterAnimation" => {
            BeginnerRuleKind::DeadEnemiesDespawnAfterAnimation
        }
        other => {
            let suggestion = closest_name(other, LEGACY_RULES.iter().copied())
                .map(|candidate| format!(" Did you mean '{candidate}'?"))
                .unwrap_or_default();
            anyhow::bail!(
                "beginner game file '{label}' has unknown rule '{other}'. Supported legacy rules: {}.{suggestion}",
                LEGACY_RULES.join(", ")
            );
        }
    };
    Ok(kind)
}

impl ActionFile {
    fn resolve(self, controls: TopDownControls) -> ActionId {
        match self {
            Self::Attack => controls.attack,
            Self::Pause => controls.pause,
            Self::Reset => controls.reset,
            Self::Reload => controls.reload,
            Self::MenuAccept => controls.menu_accept,
        }
    }
}

impl BeginnerPrefabFile {
    fn name(&self) -> &str {
        match self {
            Self::Player(prefab) => &prefab.name,
            Self::Enemy(prefab) => &prefab.name,
            Self::Pickup(prefab) => &prefab.name,
            Self::Door(prefab) => &prefab.name,
            Self::Projectile(prefab) => &prefab.name,
            Self::Spawner(prefab) => &prefab.name,
            Self::Trigger(prefab) => &prefab.name,
            Self::Checkpoint(prefab) => &prefab.name,
        }
    }

    fn texture_refs(&self) -> Vec<(&str, &str)> {
        let mut refs = Vec::new();
        match self {
            Self::Player(prefab) => refs.push((prefab.name.as_str(), prefab.sprite.as_str())),
            Self::Enemy(prefab) => refs.push((prefab.name.as_str(), prefab.sprite.as_str())),
            Self::Pickup(prefab) => refs.push((prefab.name.as_str(), prefab.sprite.as_str())),
            Self::Door(prefab) => refs.push((prefab.name.as_str(), prefab.sprite.as_str())),
            Self::Projectile(prefab) => refs.push((prefab.name.as_str(), prefab.sprite.as_str())),
            Self::Spawner(_) => {}
            Self::Trigger(prefab) => {
                if let Some(texture) = prefab.visible_debug.as_deref() {
                    refs.push((prefab.name.as_str(), texture));
                }
            }
            Self::Checkpoint(prefab) => refs.push((prefab.name.as_str(), prefab.sprite.as_str())),
        }
        refs
    }

    fn sound_refs(&self) -> Vec<(&str, &str)> {
        match self {
            Self::Pickup(prefab) => prefab
                .sound
                .as_deref()
                .map(|sound| vec![(prefab.name.as_str(), sound)])
                .unwrap_or_default(),
            _ => Vec::new(),
        }
    }

    fn animation_sheet_refs(&self) -> Vec<(&str, &str)> {
        match self {
            Self::Player(prefab) => prefab
                .animation_sheet
                .as_deref()
                .map(|sheet| vec![(prefab.name.as_str(), sheet)])
                .unwrap_or_default(),
            Self::Enemy(prefab) => prefab
                .animation_sheet
                .as_deref()
                .map(|sheet| vec![(prefab.name.as_str(), sheet)])
                .unwrap_or_default(),
            Self::Projectile(prefab) => prefab
                .animation_sheet
                .as_deref()
                .map(|sheet| vec![(prefab.name.as_str(), sheet)])
                .unwrap_or_default(),
            _ => Vec::new(),
        }
    }

    fn prefab_refs(&self) -> Vec<(&str, &str)> {
        match self {
            Self::Enemy(prefab) => prefab
                .drops
                .as_deref()
                .map(|drop| vec![(prefab.name.as_str(), drop)])
                .unwrap_or_default(),
            Self::Spawner(prefab) => vec![(prefab.name.as_str(), prefab.spawn.as_str())],
            _ => Vec::new(),
        }
    }

    fn map_refs(&self) -> Vec<(&str, &str)> {
        match self {
            Self::Door(prefab) => match &prefab.action {
                DoorActionFile::ChangeMap(map) => vec![(prefab.name.as_str(), map.as_str())],
                DoorActionFile::ChangeScene(_) | DoorActionFile::RestartLevel => Vec::new(),
            },
            _ => Vec::new(),
        }
    }

    fn scene_refs(&self) -> Vec<(&str, &str)> {
        match self {
            Self::Door(prefab) => match &prefab.action {
                DoorActionFile::ChangeScene(scene) => vec![(prefab.name.as_str(), scene.as_str())],
                DoorActionFile::ChangeMap(_) | DoorActionFile::RestartLevel => Vec::new(),
            },
            _ => Vec::new(),
        }
    }

    fn tags(&self) -> Vec<&str> {
        match self {
            Self::Player(prefab) => prefab.tags.iter().map(String::as_str).collect(),
            Self::Enemy(prefab) => prefab.tags.iter().map(String::as_str).collect(),
            Self::Pickup(prefab) => prefab.tags.iter().map(String::as_str).collect(),
            Self::Door(prefab) => prefab.tags.iter().map(String::as_str).collect(),
            Self::Projectile(prefab) => prefab.tags.iter().map(String::as_str).collect(),
            Self::Spawner(_) => Vec::new(),
            Self::Trigger(prefab) => prefab.tags.iter().map(String::as_str).collect(),
            Self::Checkpoint(prefab) => prefab.tags.iter().map(String::as_str).collect(),
        }
    }

    fn data_keys(&self) -> Vec<&str> {
        match self {
            Self::Player(prefab) => prefab.data.keys().map(String::as_str).collect(),
            Self::Enemy(prefab) => prefab.data.keys().map(String::as_str).collect(),
            Self::Pickup(prefab) => prefab.data.keys().map(String::as_str).collect(),
            Self::Door(prefab) => prefab.data.keys().map(String::as_str).collect(),
            Self::Projectile(prefab) => prefab.data.keys().map(String::as_str).collect(),
            Self::Spawner(_) => Vec::new(),
            Self::Trigger(prefab) => prefab.data.keys().map(String::as_str).collect(),
            Self::Checkpoint(prefab) => prefab.data.keys().map(String::as_str).collect(),
        }
    }

    fn validate_numbers(&self, label: &str) -> Result<()> {
        match self {
            Self::Projectile(prefab) => {
                if prefab.damage < 0 {
                    anyhow::bail!(
                        "beginner game file '{label}' projectile '{}' has negative damage {}",
                        prefab.name,
                        prefab.damage
                    );
                }
                if !prefab.speed.is_finite() || prefab.speed < 0.0 {
                    anyhow::bail!(
                        "beginner game file '{label}' projectile '{}' has invalid speed {}",
                        prefab.name,
                        prefab.speed
                    );
                }
                if !prefab.lifetime.is_finite() || prefab.lifetime <= 0.0 {
                    anyhow::bail!(
                        "beginner game file '{label}' projectile '{}' has invalid lifetime {}; use a positive number",
                        prefab.name,
                        prefab.lifetime
                    );
                }
            }
            Self::Spawner(prefab) => {
                if !prefab.every_seconds.is_finite() || prefab.every_seconds <= 0.0 {
                    anyhow::bail!(
                        "beginner game file '{label}' spawner '{}' has invalid every_seconds {}; use a positive number",
                        prefab.name,
                        prefab.every_seconds
                    );
                }
                if prefab.max_alive == Some(0) {
                    anyhow::bail!(
                        "beginner game file '{label}' spawner '{}' has max_alive: Some(0); use a positive value or None",
                        prefab.name
                    );
                }
            }
            Self::Trigger(prefab) => validate_size(label, "trigger", &prefab.name, prefab.size)?,
            Self::Checkpoint(prefab) => {
                validate_size(label, "checkpoint", &prefab.name, prefab.size)?;
            }
            _ => {}
        }
        Ok(())
    }
}

impl CustomRuleFile {
    fn name(&self) -> &str {
        match self {
            Self::Countdown(rule) => &rule.name,
        }
    }
}

fn validate_size(label: &str, kind: &str, name: &str, size: (f32, f32)) -> Result<()> {
    if !size.0.is_finite() || !size.1.is_finite() || size.0 <= 0.0 || size.1 <= 0.0 {
        anyhow::bail!(
            "beginner game file '{label}' {kind} '{name}' has invalid size ({}, {}); use positive finite numbers",
            size.0,
            size.1
        );
    }
    Ok(())
}

impl BeginnerMapFile {
    fn name(&self) -> &str {
        match self {
            Self::TextMap(map) => &map.name,
            Self::TextMapAuto(map) => &map.name,
            Self::Tiled(map) => &map.name,
            Self::Ldtk(map) => &map.name,
        }
    }

    fn start(&self) -> bool {
        match self {
            Self::TextMap(map) => map.start,
            Self::TextMapAuto(map) => map.start,
            Self::Tiled(map) => map.start,
            Self::Ldtk(map) => map.start,
        }
    }

    fn texture_refs(&self) -> Vec<(&str, &str)> {
        match self {
            Self::TextMap(map) => vec![
                (map.name.as_str(), map.theme.0.as_str()),
                (map.name.as_str(), map.theme.1.as_str()),
            ],
            Self::TextMapAuto(map) => vec![
                (map.name.as_str(), map.theme.0.as_str()),
                (map.name.as_str(), map.theme.1.as_str()),
            ],
            Self::Tiled(map) => vec![
                (map.name.as_str(), map.theme.0.as_str()),
                (map.name.as_str(), map.theme.1.as_str()),
            ],
            Self::Ldtk(map) => vec![
                (map.name.as_str(), map.theme.0.as_str()),
                (map.name.as_str(), map.theme.1.as_str()),
            ],
        }
    }

    fn prefab_refs(&self) -> Vec<(&str, &str)> {
        match self {
            Self::TextMap(map) => map
                .legend
                .values()
                .map(|prefab| (map.name.as_str(), prefab.as_str()))
                .collect(),
            Self::TextMapAuto(map) => map
                .legend
                .values()
                .map(|prefab| (map.name.as_str(), prefab.as_str()))
                .collect(),
            Self::Tiled(map) => map
                .objects
                .values()
                .map(|prefab| (map.name.as_str(), prefab.as_str()))
                .collect(),
            Self::Ldtk(map) => map
                .entities
                .values()
                .map(|prefab| (map.name.as_str(), prefab.as_str()))
                .collect(),
        }
    }
}

fn default_controls() -> BeginnerControlsFile {
    BeginnerControlsFile::Structured(BeginnerControlsKind::TopDown)
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

const fn default_projectile_damage() -> i32 {
    1
}

const fn default_projectile_speed() -> f32 {
    300.0
}

const fn default_projectile_lifetime() -> f32 {
    1.0
}

const fn default_spawn_every() -> f32 {
    1.0
}

const fn default_area_size() -> (f32, f32) {
    (32.0, 32.0)
}

const fn default_tile_size() -> f32 {
    32.0
}

const fn default_music_volume() -> f32 {
    1.0
}

const fn default_shoot_cooldown() -> f32 {
    0.2
}

const fn default_true() -> bool {
    true
}

const LEGACY_RULES: &[&str] = &[
    "top_down_controls",
    "player_collects_pickups",
    "enemies_damage_player",
    "dead_enemies_despawn",
    "enemy_drops",
    "projectiles",
    "projectiles_move",
    "projectiles_expire_after_lifetime",
    "projectiles_damage_enemies",
    "projectiles_despawn_on_hit",
    "projectile_impact_animation_before_despawn",
    "spawners_spawn_prefabs",
    "doors_change_maps",
    "player_activates_checkpoints",
    "respawn_at_checkpoint",
    "camera_follows_player",
    "pause_and_reset",
    "show_basic_ui",
    "show_score",
    "show_enemy_count",
    "show_player_health",
    "show_menu",
    "show_pause_menu",
    "show_game_over_panel",
    "show_win_panel",
    "win_when_all_pickups_collected",
    "win_when_all_enemies_dead",
    "animate_enemies_by_movement",
    "animate_player_directionally",
    "animate_enemies_directionally",
    "animate_attacks_directionally",
    "dead_enemies_play_death_animation",
    "dead_enemies_despawn_after_animation",
];

#[cfg(test)]
mod tests {
    use super::{
        BeginnerGameFile, load_beginner_game_text, parse_beginner_game_source, validate_file,
        validate_file_with_base,
    };
    use crate::app::{GameApp, GamePlugin};
    use crate::beginner::actors::Enemy;
    use crate::harness::GameTestHarness;
    use anyhow::Result;
    use game_combat::Health;
    use game_core::backend::AudioCommand;
    use game_core::world::Velocity;
    use std::fs;
    use std::path::{Path, PathBuf};

    const GAME: &str = r#"(
    version: 1,
    assets: (
        textures: ["player", "slime", "coin", "bolt", "spawner_debug", "floor", "wall", "door", "checkpoint"],
        sounds: ["hit"],
        music: ["theme"],
    ),
    controls: TopDown,
    prefabs: [
        Player((name: "player", sprite: "player")),
        Enemy((name: "slime", sprite: "slime", chase_player: true, tags: ["enemy"], drops: Some("coin"))),
        Pickup((name: "coin", sprite: "coin", score: 1, heal_player: Some(5), sound: Some("hit"))),
        Projectile((name: "bolt", sprite: "bolt", damage: 2, speed: 260.0, lifetime: 0.8)),
        Spawner((name: "spawner", spawn: "slime", every_seconds: 2.0, max_alive: Some(4), placement: NearPlayer(96.0))),
        Door((name: "exit", sprite: "door", action: ChangeMap("level_2"))),
        Trigger((name: "danger", size: (32.0, 32.0), visible_debug: Some("spawner_debug"), tags: ["danger"], data: {"fuse": 0.01})),
        Checkpoint((name: "checkpoint", sprite: "checkpoint")),
    ],
    maps: [
        TextMap((
            name: "level_1",
            path: "maps/beginner_text_map.txt",
            theme: ("floor", "wall"),
            legend: {'P': "player", 'E': "slime", 'C': "coin"},
            start: true,
        )),
        TextMap((
            name: "level_2",
            path: "maps/level_1.txt",
            theme: ("floor", "wall"),
            legend: {'P': "player", 'E': "slime"},
        )),
    ],
    scene_flow: Some((
        game: Some("level_1"),
        win: Some("level_2"),
        restart_on: Some(Reset),
        win_condition: Some(AllEnemiesDead),
    )),
    audio: (
        music_on_scene: {"level_1": (track: "theme", volume: 0.5)},
    ),
    actions: [
        PlayerShoots((prefab: "bolt", action: Attack, cooldown: 0.2, direction: Right, sound: Some("hit"))),
    ],
    custom_rules: [
        Countdown((
            name: "danger fuse",
            tag: "danger",
            key: "fuse",
            when_zero: [
                DamageTagged(tag: "enemy", amount: 2, radius: 48.0),
                PlaySound("hit"),
                DespawnSelf,
            ],
        )),
    ],
    rules: [
        TopDownControls,
        PlayerCollectsPickups,
        EnemyDrops,
        Projectiles,
        SpawnersSpawnPrefabs,
        EnemiesDamagePlayer,
        DoorsChangeMaps,
        CameraFollowsPlayer,
        ShowBasicUi,
        ShowPlayerHealth,
        WinWhenAllEnemiesDead,
    ],
)"#;

    const SCRIPT_RULE_GAME: &str = r#"(
    version: 1,
    assets: (
        textures: ["player", "slime", "coin", "floor", "wall"],
        sounds: ["hit"],
    ),
    controls: TopDown,
    prefabs: [
        Player((name: "player", sprite: "player")),
        Enemy((name: "slime", sprite: "slime", health: 1)),
        Pickup((name: "coin", sprite: "coin", score: 1)),
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
    rules: [
        TopDownControls,
        OnEnemyDeath(
            prefab: "slime",
            effects: [AddScore(3), PlaySound("hit"), SpawnPrefab("coin"), DespawnSelf],
        ),
        EverySeconds(
            seconds: 0.001,
            effects: [SpawnNearPlayer(prefab: "coin", radius: 32.0)],
        ),
        OnScoreReaches(score: 3, effects: [AddScore(5)]),
    ],
)"#;

    struct DataPlugin;

    impl GamePlugin for DataPlugin {
        fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
            load_beginner_game_text(game, GAME, "inline.ron").map(|_| ())
        }
    }

    struct ScriptRuleDataPlugin;

    impl GamePlugin for ScriptRuleDataPlugin {
        fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
            load_beginner_game_text(game, SCRIPT_RULE_GAME, "script-rules.ron").map(|_| ())
        }
    }

    struct FileDataPlugin;

    impl GamePlugin for FileDataPlugin {
        fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
            game.load_beginner_file("game.ron").map(|_| ())
        }
    }

    struct TempFileDataPlugin {
        path: String,
        debug: bool,
    }

    impl GamePlugin for TempFileDataPlugin {
        fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
            game.load_beginner_file(&self.path)?;
            if self.debug {
                game.enable_debug_overlay();
            }
            Ok(())
        }
    }

    struct FullDemoDataPlugin;

    impl GamePlugin for FullDemoDataPlugin {
        fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../examples/data-driven-full-demo/assets/game.ron");
            game.load_beginner_file(path.to_str().unwrap()).map(|_| ())
        }
    }

    #[test]
    fn compiles_the_small_game_file_through_the_normal_beginner_builders() {
        let game = GameTestHarness::from_plugin(DataPlugin).unwrap();

        assert_eq!(game.current_map_name().as_deref(), Some("level_1"));
        assert_eq!(game.count::<crate::beginner::actors::Player>(), 1);
        assert_eq!(game.count::<crate::beginner::actors::Enemy>(), 1);
        assert_eq!(game.count::<crate::beginner::actors::Pickup>(), 1);
        assert_eq!(game.count::<crate::beginner::actors::Spawner>(), 0);
    }

    #[test]
    fn structured_script_rules_run_from_data_files() {
        let mut game = GameTestHarness::from_plugin(ScriptRuleDataPlugin).unwrap();

        assert_eq!(game.count::<crate::beginner::actors::Pickup>(), 1);
        game.set_enemy_health(0, 0);
        game.step_seconds(0.001);

        game.assert_score(8);
        game.assert_sound_played();
        assert_eq!(game.count::<crate::beginner::actors::Pickup>(), 3);
        assert_eq!(game.enemy_count(), 0);
    }

    #[test]
    fn when_rules_run_conditions_and_game_effects_from_data_files() {
        let source = r#"(
            version: 1,
            assets: (
                textures: ["player", "slime", "floor", "wall"],
                sounds: ["hit"],
                music: ["theme"],
            ),
            controls: TopDown,
            prefabs: [
                Player((name: "player", sprite: "player", health: 10)),
                Enemy((name: "slime", sprite: "slime", tags: ["enemy"])),
            ],
            maps: [
                TextMap((
                    name: "level_1",
                    path: "maps/level_1.txt",
                    theme: ("floor", "wall"),
                    legend: {'P': "player", 'E': "slime"},
                    start: true,
                )),
            ],
            rules: [
                TopDownControls,
                When(
                    condition: ActionPressed(Attack),
                    effects: [AddScore(2)],
                ),
                When(
                    condition: ScoreAtLeast(2),
                    effects: [
                        SetScore(7),
                        DamagePlayer(amount: 5),
                        HealPlayer(2),
                        PlaySound("hit"),
                        PlayMusic("theme"),
                        StopMusic,
                        ShowUiText("Gate open"),
                    ],
                ),
            ],
        )"#;

        struct RichWhenPlugin(&'static str);

        impl GamePlugin for RichWhenPlugin {
            fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
                load_beginner_game_text(game, self.0, "rich-when.ron").map(|_| ())
            }
        }

        let mut game = GameTestHarness::from_plugin(RichWhenPlugin(source)).unwrap();

        game.tap_action("attack");

        game.assert_score(7);
        game.assert_player_health(7);
        assert!(
            game.audio_commands()
                .iter()
                .any(|command| matches!(command, AudioCommand::Play { .. }))
        );
        assert!(
            game.audio_commands()
                .iter()
                .any(|command| matches!(command, AudioCommand::PlayMusic { .. }))
        );
        assert!(
            game.audio_commands()
                .iter()
                .any(|command| matches!(command, AudioCommand::StopMusic))
        );
        game.frame(1.0 / 60.0);
        game.assert_ui_contains("Gate open");
    }

    #[test]
    fn when_rules_support_timers_and_tag_zero_conditions() {
        let source = r#"(
            version: 1,
            assets: (
                textures: ["player", "slime", "floor", "wall"],
            ),
            controls: TopDown,
            prefabs: [
                Player((name: "player", sprite: "player")),
                Enemy((name: "slime", sprite: "slime", health: 1, tags: ["enemy"])),
            ],
            maps: [
                TextMap((
                    name: "level_1",
                    path: "maps/level_1.txt",
                    theme: ("floor", "wall"),
                    legend: {'P': "player", 'E': "slime"},
                    start: true,
                )),
            ],
            rules: [
                TopDownControls,
                When(
                    condition: TimerReached(name: "first_wave", seconds: 0.01),
                    effects: [SpawnNearPlayer(prefab: "slime", radius: 32.0)],
                ),
                When(
                    condition: TagCountZero("enemy"),
                    effects: [AddScore(10)],
                ),
            ],
        )"#;

        struct TimerWhenPlugin(&'static str);

        impl GamePlugin for TimerWhenPlugin {
            fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
                load_beginner_game_text(game, self.0, "timer-when.ron").map(|_| ())
            }
        }

        let mut game = GameTestHarness::from_plugin(TimerWhenPlugin(source)).unwrap();

        game.step_seconds(0.02);
        assert_eq!(game.enemy_count(), 2);
        game.set_enemy_health(0, 0);
        game.set_enemy_health(1, 0);
        game.step_seconds(0.001);

        game.assert_score(10);
    }

    #[test]
    fn f5_reloads_beginner_game_file_map_path_and_respawns_current_map() {
        let dir = temp_data_project("reload-map-path");
        write_map(&dir, "level_a.txt", "#####\n#P..#\n#####\n");
        write_map(&dir, "level_b.txt", "#####\n#PE.#\n#####\n");
        let game_file = dir.join("game.ron");
        fs::write(&game_file, reload_game_ron("level_a.txt", "")).unwrap();

        let mut game = GameTestHarness::from_plugin(TempFileDataPlugin {
            path: game_file.to_string_lossy().into_owned(),
            debug: true,
        })
        .unwrap();
        assert_eq!(game.count::<Enemy>(), 0);

        fs::write(
            &game_file,
            reload_game_ron("level_b.txt", "").replace("health: 30", "health: 77"),
        )
        .unwrap();
        game.tap_action("reload");

        assert_eq!(game.current_map_name().as_deref(), Some("level_1"));
        assert_eq!(game.count::<Enemy>(), 1);
        let enemy = game.world().ids_with::<Enemy>()[0];
        let health = game.world().get::<Health>(enemy).unwrap();
        assert_eq!(health.max, 77);
        assert_eq!(health.current, 77);
        game.frame(1.0 / 60.0);
        game.assert_ui_contains("game.ron reload: partial");
        game.assert_ui_contains("last reload: game.ron ok (level_1)");
    }

    #[test]
    fn f5_rejects_beginner_game_file_asset_identity_changes() {
        let dir = temp_data_project("reload-asset-identity");
        write_map(&dir, "level_a.txt", "#####\n#P..#\n#####\n");
        write_map(&dir, "level_b.txt", "#####\n#PE.#\n#####\n");
        let game_file = dir.join("game.ron");
        fs::write(&game_file, reload_game_ron("level_a.txt", "")).unwrap();

        let mut game = GameTestHarness::from_plugin(TempFileDataPlugin {
            path: game_file.to_string_lossy().into_owned(),
            debug: true,
        })
        .unwrap();
        assert_eq!(game.count::<Enemy>(), 0);

        fs::write(
            &game_file,
            reload_game_ron("level_b.txt", r#", "new_texture""#),
        )
        .unwrap();
        game.tap_action("reload");

        assert_eq!(game.count::<Enemy>(), 0);
        game.frame(1.0 / 60.0);
        game.assert_ui_contains("game.ron reload: partial");
        game.assert_ui_contains("changed its texture assets list");
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
        fs::create_dir_all(dir.join("maps")).unwrap();
        dir
    }

    fn write_map(dir: &Path, name: &str, contents: &str) {
        fs::write(dir.join("maps").join(name), contents).unwrap();
    }

    fn assert_reload_error_contains(initial: String, updated: String, expected: &str) {
        let dir = temp_data_project("reload-identity");
        write_map(&dir, "level.txt", "#####\n#P..#\n#####\n");
        write_map(&dir, "level_2.txt", "#####\n#P..#\n#####\n");
        let game_file = dir.join("game.ron");
        fs::write(&game_file, initial).unwrap();

        let mut game = GameTestHarness::from_plugin(TempFileDataPlugin {
            path: game_file.to_string_lossy().into_owned(),
            debug: true,
        })
        .unwrap();

        fs::write(&game_file, updated).unwrap();
        game.tap_action("reload");

        game.frame(1.0 / 60.0);
        game.assert_ui_contains("game.ron reload: partial");
        game.assert_ui_contains(expected);
    }

    fn reload_game_ron(map_file: &str, extra_textures: &str) -> String {
        format!(
            r#"(
    version: 1,
    assets: (
        textures: ["player", "slime", "floor", "wall"{extra_textures}],
    ),
    controls: TopDown,
    prefabs: [
        Player((name: "player", sprite: "player")),
        Enemy((name: "slime", sprite: "slime", health: 30)),
    ],
    maps: [
        TextMap((
            name: "level_1",
            path: "maps/{map_file}",
            theme: ("floor", "wall"),
            legend: {{'P': "player", 'E': "slime"}},
            start: true,
        )),
    ],
    rules: [
        TopDownControls,
    ],
)"#
        )
    }

    fn spawner_reload_game_ron(map_file: &str, enemy_health: i32) -> String {
        format!(
            r#"(
    version: 1,
    assets: (
        textures: ["player", "slime", "floor", "wall"],
    ),
    controls: TopDown,
    prefabs: [
        Player((name: "player", sprite: "player")),
        Enemy((name: "slime", sprite: "slime", health: {enemy_health})),
        Spawner((name: "spawner", spawn: "slime", every_seconds: 0.01, max_alive: Some(1))),
    ],
    maps: [
        TextMap((
            name: "level_1",
            path: "maps/{map_file}",
            theme: ("floor", "wall"),
            legend: {{'P': "player", 'S': "spawner"}},
            start: true,
        )),
    ],
    rules: [
        TopDownControls,
        SpawnersSpawnPrefabs,
    ],
)"#
        )
    }

    fn countdown_reload_game_ron(map_file: &str, damage: i32) -> String {
        format!(
            r#"(
    version: 1,
    assets: (
        textures: ["player", "floor", "wall"],
    ),
    controls: TopDown,
    prefabs: [
        Player((name: "player", sprite: "player", health: 100)),
        Trigger((name: "danger", size: (32.0, 32.0), tags: ["danger"], data: {{"fuse": 0.01}})),
    ],
    maps: [
        TextMap((
            name: "level_1",
            path: "maps/{map_file}",
            theme: ("floor", "wall"),
            legend: {{'P': "player", 'D': "danger"}},
            start: true,
        )),
    ],
    custom_rules: [
        Countdown((
            name: "danger fuse",
            tag: "danger",
            key: "fuse",
            when_zero: [
                DamagePlayer(amount: {damage}, radius: 128.0),
                DespawnSelf,
            ],
        )),
    ],
    rules: [
        TopDownControls,
    ],
)"#
        )
    }

    fn scene_text_reload_game_ron(map_file: &str, menu_text: &str) -> String {
        format!(
            r#"(
    version: 1,
    assets: (
        textures: ["player", "floor", "wall"],
    ),
    controls: TopDown,
    prefabs: [
        Player((name: "player", sprite: "player")),
    ],
    maps: [
        TextMap((
            name: "level_1",
            path: "maps/{map_file}",
            theme: ("floor", "wall"),
            legend: {{'P': "player"}},
            start: true,
        )),
    ],
    scene_flow: Some((
        menu: Some("menu"),
        game: Some("level_1"),
        menu_text: Some("{menu_text}"),
        menu_button: Some((label: "Start", map: "level_1")),
    )),
    rules: [
        TopDownControls,
    ],
)"#
        )
    }

    fn audio_reload_game_ron(map_file: &str, volume: f32) -> String {
        format!(
            r#"(
    version: 1,
    assets: (
        textures: ["player", "floor", "wall"],
        music: ["theme"],
    ),
    controls: TopDown,
    prefabs: [
        Player((name: "player", sprite: "player")),
    ],
    maps: [
        TextMap((
            name: "level_1",
            path: "maps/{map_file}",
            theme: ("floor", "wall"),
            legend: {{'P': "player"}},
            start: true,
        )),
    ],
    scene_flow: Some((
        menu: Some("menu"),
        game: Some("level_1"),
    )),
    audio: (
        music_on_scene: {{"menu": (track: "theme", volume: {volume})}},
    ),
    rules: [
        TopDownControls,
    ],
)"#
        )
    }

    fn action_reload_game_ron(map_file: &str, direction: &str) -> String {
        format!(
            r#"(
    version: 1,
    assets: (
        textures: ["player", "bolt", "floor", "wall"],
    ),
    controls: TopDown,
    prefabs: [
        Player((name: "player", sprite: "player")),
        Projectile((name: "bolt", sprite: "bolt", speed: 100.0, lifetime: 1.0)),
    ],
    maps: [
        TextMap((
            name: "level_1",
            path: "maps/{map_file}",
            theme: ("floor", "wall"),
            legend: {{'P': "player"}},
            start: true,
        )),
    ],
    actions: [
        PlayerShoots((prefab: "bolt", action: Attack, cooldown: 0.0, direction: {direction})),
    ],
    rules: [
        TopDownControls,
    ],
)"#
        )
    }

    fn last_play_music_volume(game: &GameTestHarness) -> f32 {
        game.audio_commands()
            .iter()
            .rev()
            .find_map(|command| match command {
                AudioCommand::PlayMusic { volume, .. } => Some(*volume),
                _ => None,
            })
            .expect("expected a PlayMusic command")
    }

    #[test]
    fn f5_reloaded_prefabs_are_used_by_command_spawned_rules() {
        let dir = temp_data_project("reload-spawner-prefab");
        write_map(&dir, "level.txt", "#####\n#PS.#\n#####\n");
        let game_file = dir.join("game.ron");
        fs::write(&game_file, spawner_reload_game_ron("level.txt", 30)).unwrap();

        let mut game = GameTestHarness::from_plugin(TempFileDataPlugin {
            path: game_file.to_string_lossy().into_owned(),
            debug: true,
        })
        .unwrap();

        game.fixed_step(0.02);
        assert_eq!(game.count::<Enemy>(), 1);
        let enemy = game.world().ids_with::<Enemy>()[0];
        assert_eq!(game.world().get::<Health>(enemy).unwrap().max, 30);

        fs::write(&game_file, spawner_reload_game_ron("level.txt", 77)).unwrap();
        game.tap_action("reload");
        assert_eq!(game.count::<Enemy>(), 0);

        game.fixed_step(0.02);
        assert_eq!(game.count::<Enemy>(), 1);
        let enemy = game.world().ids_with::<Enemy>()[0];
        let health = game.world().get::<Health>(enemy).unwrap();
        assert_eq!(health.max, 77);
        assert_eq!(health.current, 77);
        game.frame(1.0 / 60.0);
        game.assert_ui_contains("game.ron reload: partial");
        game.assert_ui_contains("last reload: game.ron ok (level_1)");
    }

    #[test]
    fn f5_rejects_added_prefabs_with_restart_required_diagnostic() {
        let initial = reload_game_ron("level.txt", "");
        let updated = initial.replace(
            "        Enemy((name: \"slime\", sprite: \"slime\", health: 30)),",
            "        Enemy((name: \"slime\", sprite: \"slime\", health: 30)),\n        Enemy((name: \"bat\", sprite: \"slime\", health: 15)),",
        );

        assert_reload_error_contains(initial, updated, "changed its prefabs list");
    }

    #[test]
    fn f5_rejects_added_maps_with_restart_required_diagnostic() {
        let initial = reload_game_ron("level.txt", "");
        let updated = initial.replace(
            "            start: true,\n        )),\n    ],",
            "            start: true,\n        )),\n        TextMap((\n            name: \"level_2\",\n            path: \"maps/level_2.txt\",\n            theme: (\"floor\", \"wall\"),\n            legend: {'P': \"player\", 'E': \"slime\"},\n            start: false,\n        )),\n    ],",
        );

        assert_reload_error_contains(initial, updated, "changed its maps list");
    }

    #[test]
    fn f5_rejects_added_scene_flow_with_restart_required_diagnostic() {
        let initial = reload_game_ron("level.txt", "");
        let updated = initial.replace(
            "    rules: [",
            "    scene_flow: Some((menu: Some(\"menu\"), game: Some(\"level_1\"))),\n    rules: [",
        );

        assert_reload_error_contains(initial, updated, "changed its scene flow structure");
    }

    #[test]
    fn f5_rejects_action_identity_changes_with_restart_required_diagnostic() {
        let initial = action_reload_game_ron("level.txt", "Right");
        let updated = initial.replace("action: Attack", "action: Reload");

        assert_reload_error_contains(initial, updated, "changed its actions");
    }

    #[test]
    fn f5_reloads_existing_custom_countdown_rule_values() {
        let dir = temp_data_project("reload-countdown-rule");
        write_map(&dir, "level.txt", "#####\n#PD.#\n#####\n");
        let game_file = dir.join("game.ron");
        fs::write(&game_file, countdown_reload_game_ron("level.txt", 3)).unwrap();

        let mut game = GameTestHarness::from_plugin(TempFileDataPlugin {
            path: game_file.to_string_lossy().into_owned(),
            debug: true,
        })
        .unwrap();

        game.fixed_step(0.02);
        assert_eq!(game.player().health(), 97);
        assert_eq!(game.count::<crate::beginner::actors::TriggerArea>(), 0);

        fs::write(&game_file, countdown_reload_game_ron("level.txt", 11)).unwrap();
        game.tap_action("reload");
        assert_eq!(game.player().health(), 100);

        game.fixed_step(0.02);
        assert_eq!(game.player().health(), 89);
        game.frame(1.0 / 60.0);
        game.assert_ui_contains("game.ron reload: partial");
        game.assert_ui_contains("last reload: game.ron ok (level_1)");
    }

    #[test]
    fn f5_rejects_enabled_rule_list_changes_until_runtime_rules_are_dynamic() {
        let dir = temp_data_project("reload-rule-identity");
        write_map(&dir, "level.txt", "#####\n#P..#\n#####\n");
        let game_file = dir.join("game.ron");
        fs::write(&game_file, reload_game_ron("level.txt", "")).unwrap();

        let mut game = GameTestHarness::from_plugin(TempFileDataPlugin {
            path: game_file.to_string_lossy().into_owned(),
            debug: true,
        })
        .unwrap();

        fs::write(
            &game_file,
            reload_game_ron("level.txt", "").replace(
                "rules: [\n        TopDownControls,\n    ],",
                "rules: [\n        TopDownControls,\n        ShowScore,\n    ],",
            ),
        )
        .unwrap();
        game.tap_action("reload");

        game.frame(1.0 / 60.0);
        game.assert_ui_contains("game.ron reload: partial");
        game.assert_ui_contains("changed its enabled rules");
    }

    #[test]
    fn f5_reloads_existing_scene_flow_text_and_buttons() {
        let dir = temp_data_project("reload-scene-text");
        write_map(&dir, "level.txt", "#####\n#P..#\n#####\n");
        let game_file = dir.join("game.ron");
        fs::write(
            &game_file,
            scene_text_reload_game_ron("level.txt", "First title"),
        )
        .unwrap();

        let mut game = GameTestHarness::from_plugin(TempFileDataPlugin {
            path: game_file.to_string_lossy().into_owned(),
            debug: true,
        })
        .unwrap();
        game.frame(1.0 / 60.0);
        game.assert_ui_contains("First title");

        fs::write(
            &game_file,
            scene_text_reload_game_ron("level.txt", "Second title"),
        )
        .unwrap();
        game.tap_action("reload");
        game.frame(1.0 / 60.0);

        game.assert_ui_contains("Second title");
        game.assert_ui_contains("game.ron reload: partial");
    }

    #[test]
    fn f5_reloads_existing_audio_scene_settings() {
        let dir = temp_data_project("reload-audio");
        write_map(&dir, "level.txt", "#####\n#P..#\n#####\n");
        let game_file = dir.join("game.ron");
        fs::write(&game_file, audio_reload_game_ron("level.txt", 0.25)).unwrap();

        let mut game = GameTestHarness::from_plugin(TempFileDataPlugin {
            path: game_file.to_string_lossy().into_owned(),
            debug: true,
        })
        .unwrap();
        game.frame(1.0 / 60.0);
        assert_eq!(last_play_music_volume(&game), 0.25);

        fs::write(&game_file, audio_reload_game_ron("level.txt", 0.75)).unwrap();
        game.tap_action("reload");
        game.frame(1.0 / 60.0);

        assert_eq!(last_play_music_volume(&game), 0.75);
    }

    #[test]
    fn f5_reloads_existing_player_shoot_action_settings() {
        let dir = temp_data_project("reload-action");
        write_map(&dir, "level.txt", "#####\n#P..#\n#####\n");
        let game_file = dir.join("game.ron");
        fs::write(&game_file, action_reload_game_ron("level.txt", "Right")).unwrap();

        let mut game = GameTestHarness::from_plugin(TempFileDataPlugin {
            path: game_file.to_string_lossy().into_owned(),
            debug: true,
        })
        .unwrap();

        game.tap_action("attack");
        let projectile = game
            .world()
            .ids_with::<crate::beginner::actors::Projectile>()[0];
        assert_eq!(
            game.world().get::<Velocity>(projectile).unwrap().0,
            glam::vec2(100.0, 0.0)
        );

        fs::write(&game_file, action_reload_game_ron("level.txt", "Up")).unwrap();
        game.tap_action("reload");
        game.tap_action("attack");

        let projectile = game
            .world()
            .ids_with::<crate::beginner::actors::Projectile>()[0];
        assert_eq!(
            game.world().get::<Velocity>(projectile).unwrap().0,
            glam::vec2(0.0, -100.0)
        );
    }

    #[test]
    fn validation_names_unknown_legend_prefabs_and_offers_a_suggestion() {
        let source = GAME.replace("'E': \"slime\"", "'E': \"slimee\"");
        let file: BeginnerGameFile = ron::from_str(&source).unwrap();
        let error = validate_file(&file, "game.ron").unwrap_err().to_string();

        assert!(error.contains("references unknown prefab 'slimee'"));
        assert!(error.contains("Did you mean 'slime'?"));
    }

    #[test]
    fn validation_names_bad_map_symbols_with_row_and_column() {
        let dir = temp_data_project("bad-map-symbol");
        write_map(&dir, "beginner_text_map.txt", "#####\n#PZ.#\n#####\n");
        write_map(&dir, "level_1.txt", "#####\n#P.E#\n#####\n");
        let file: BeginnerGameFile = ron::from_str(GAME).unwrap();
        let error = validate_file_with_base(&file, "game.ron", Some(&dir))
            .unwrap_err()
            .to_string();

        assert!(error.contains("map 'level_1' has an invalid symbol"));
        assert!(error.contains("uses symbol 'Z'"));
        assert!(error.contains("At row 2, col 3"));
        assert!(error.contains(".legend('Z', \"some_prefab\")"));
    }

    #[test]
    fn validation_names_unknown_prefab_assets_and_lists_known_keys() {
        let source = GAME.replace("sprite: \"player\"", "sprite: \"plaeyr\"");
        let file: BeginnerGameFile = ron::from_str(&source).unwrap();
        let error = validate_file(&file, "game.ron").unwrap_err().to_string();

        assert!(error.contains("references unknown texture 'plaeyr'"));
        assert!(error.contains("Known textures:"));
        assert!(error.contains("player"));
        assert!(error.contains("Did you mean 'player'?"));
    }

    #[test]
    fn validation_names_unknown_spawner_targets() {
        let source = GAME.replace("spawn: \"slime\"", "spawn: \"slmie\"");
        let file: BeginnerGameFile = ron::from_str(&source).unwrap();
        let error = validate_file(&file, "game.ron").unwrap_err().to_string();

        assert!(error.contains("references unknown prefab 'slmie'"));
        assert!(error.contains("Did you mean 'slime'?"));
    }

    #[test]
    fn validation_names_unknown_tiled_object_prefabs() {
        let dir = temp_data_project("tiled-object-prefab");
        fs::write(
            dir.join("maps/tiled_demo.tmx"),
            r#"<?xml version="1.0" encoding="UTF-8"?>
<map width="5" height="3" tilewidth="32" tileheight="32">
  <layer name="Collision" width="5" height="3"><data encoding="csv">1,1,1,1,1,1,0,0,0,1,1,1,1,1,1</data></layer>
</map>
"#,
        )
        .unwrap();
        let source = r#"(
    version: 1,
    assets: (
        textures: ["player", "slime", "floor", "wall"],
    ),
    controls: TopDown,
    prefabs: [
        Player((name: "player", sprite: "player")),
        Enemy((name: "slime", sprite: "slime")),
    ],
    maps: [
        Tiled((
            name: "level_1",
            path: "maps/tiled_demo.tmx",
            theme: ("floor", "wall"),
            objects: {"Player": "player", "Slime": "slmie"},
            start: true,
        )),
    ],
    rules: [
        TopDownControls,
    ],
)"#;
        let file: BeginnerGameFile = ron::from_str(source).unwrap();
        let error = validate_file_with_base(&file, "game.ron", Some(&dir))
            .unwrap_err()
            .to_string();

        assert!(error.contains("map 'level_1' references unknown prefab 'slmie'"));
        assert!(error.contains("Did you mean 'slime'?"));
    }

    #[test]
    fn validation_names_unknown_door_maps() {
        let source = GAME.replace("ChangeMap(\"level_2\")", "ChangeMap(\"levle_2\")");
        let file: BeginnerGameFile = ron::from_str(&source).unwrap();
        let error = validate_file(&file, "game.ron").unwrap_err().to_string();

        assert!(error.contains("references unknown map 'levle_2'"));
        assert!(error.contains("Did you mean 'level_2'?"));
    }

    #[test]
    fn validation_names_unknown_custom_rule_sounds() {
        let source = GAME.replace("PlaySound(\"hit\")", "PlaySound(\"hti\")");
        let file: BeginnerGameFile = ron::from_str(&source).unwrap();
        let error = validate_file(&file, "game.ron").unwrap_err().to_string();

        assert!(error.contains("references unknown sound 'hti'"));
        assert!(error.contains("Did you mean 'hit'?"));
    }

    #[test]
    fn validation_names_unknown_music_tracks() {
        let source = GAME.replace("track: \"theme\"", "track: \"theem\"");
        let file: BeginnerGameFile = ron::from_str(&source).unwrap();
        let error = validate_file(&file, "game.ron").unwrap_err().to_string();

        assert!(error.contains("references unknown music 'theem'"));
        assert!(error.contains("Known music: theme"));
        assert!(error.contains("Did you mean 'theme'?"));
    }

    #[test]
    fn validation_names_unknown_script_music_tracks() {
        let source = GAME.replace(
            "        WinWhenAllEnemiesDead,\n",
            "        When(condition: ActionPressed(Attack), effects: [PlayMusic(\"theem\")]),\n        WinWhenAllEnemiesDead,\n",
        );
        let file: BeginnerGameFile = ron::from_str(&source).unwrap();
        let error = validate_file(&file, "game.ron").unwrap_err().to_string();

        assert!(error.contains("references unknown music 'theem'"));
        assert!(error.contains("Known music: theme"));
        assert!(error.contains("Did you mean 'theme'?"));
    }

    #[test]
    fn validation_names_unknown_animation_sheets() {
        let source = GAME
            .replace(
                "music: [\"theme\"],",
                "music: [\"theme\"],\n        animation_sheets: [\"hero\"],",
            )
            .replace(
                "Player((name: \"player\", sprite: \"player\"))",
                "Player((name: \"player\", sprite: \"player\", animation_sheet: Some(\"hre\")))",
            );
        let file: BeginnerGameFile = ron::from_str(&source).unwrap();
        let error = validate_file(&file, "game.ron").unwrap_err().to_string();

        assert!(error.contains("references unknown animation sheet 'hre'"));
        assert!(error.contains("Known animation sheets: hero"));
        assert!(error.contains("Did you mean 'hero'?"));
    }

    #[test]
    fn validation_names_unknown_custom_rule_tags() {
        let source = GAME.replace(
            "            tag: \"danger\",\n            key:",
            "            tag: \"dangeer\",\n            key:",
        );
        let file: BeginnerGameFile = ron::from_str(&source).unwrap();
        let error = validate_file(&file, "game.ron").unwrap_err().to_string();

        assert!(error.contains("references unknown tag 'dangeer'"));
        assert!(error.contains("Known tags:"));
        assert!(error.contains("danger"));
        assert!(error.contains("Did you mean 'danger'?"));
    }

    #[test]
    fn custom_countdown_accepts_declared_data_key_for_known_tag() {
        let file: BeginnerGameFile = ron::from_str(GAME).unwrap();
        validate_file(&file, "game.ron").unwrap();
    }

    #[test]
    fn custom_countdown_rejects_unknown_data_key_for_known_tag() {
        let source = GAME.replace("key: \"fuse\"", "key: \"fues\"");
        let file: BeginnerGameFile = ron::from_str(&source).unwrap();
        let error = validate_file(&file, "game.ron").unwrap_err().to_string();

        assert!(error.contains("custom rule 'danger fuse' counts down key 'fues' on tag 'danger'"));
        assert!(error.contains("but no prefab with that tag declares that data key"));
        assert!(error.contains("Prefabs with tag 'danger': danger"));
        assert!(error.contains("Known data keys for tag 'danger': fuse"));
        assert!(error.contains("Did you mean 'fuse'?"));
        assert!(error.contains("Fix: add data: {\"fues\": 3.0}"));
    }

    #[test]
    fn custom_countdown_error_explains_tags_without_data_keys() {
        let source = GAME.replace(", data: {\"fuse\": 0.01}", "");
        let file: BeginnerGameFile = ron::from_str(&source).unwrap();
        let error = validate_file(&file, "game.ron").unwrap_err().to_string();

        assert!(error.contains("custom rule 'danger fuse' counts down key 'fuse' on tag 'danger'"));
        assert!(error.contains("Prefabs with tag 'danger': danger"));
        assert!(error.contains("Known data keys for tag 'danger': (none)"));
        assert!(error.contains("No prefab tagged 'danger' declares any data keys."));
    }

    #[test]
    fn validation_names_unknown_script_condition_tags() {
        let source = GAME.replace(
            "        WinWhenAllEnemiesDead,\n",
            "        When(condition: TagCountZero(\"dangeer\"), effects: [AddScore(1)]),\n        WinWhenAllEnemiesDead,\n",
        );
        let file: BeginnerGameFile = ron::from_str(&source).unwrap();
        let error = validate_file(&file, "game.ron").unwrap_err().to_string();

        assert!(error.contains("references unknown tag 'dangeer'"));
        assert!(error.contains("Known tags:"));
        assert!(error.contains("danger"));
        assert!(error.contains("Did you mean 'danger'?"));
    }

    #[test]
    fn validation_names_unknown_script_effect_tags() {
        let source = GAME.replace(
            "        WinWhenAllEnemiesDead,\n",
            "        When(condition: ActionPressed(Attack), effects: [DespawnTagged(\"dangeer\")]),\n        WinWhenAllEnemiesDead,\n",
        );
        let file: BeginnerGameFile = ron::from_str(&source).unwrap();
        let error = validate_file(&file, "game.ron").unwrap_err().to_string();

        assert!(error.contains("references unknown tag 'dangeer'"));
        assert!(error.contains("Known tags:"));
        assert!(error.contains("danger"));
        assert!(error.contains("Did you mean 'danger'?"));
    }

    #[test]
    fn validation_names_unknown_custom_rule_scenes() {
        let source = GAME.replace("DespawnSelf,", "ChangeScene(\"levle_2\"),");
        let file: BeginnerGameFile = ron::from_str(&source).unwrap();
        let error = validate_file(&file, "game.ron").unwrap_err().to_string();

        assert!(error.contains("references unknown scene 'levle_2'"));
        assert!(error.contains("Known scenes:"));
        assert!(error.contains("level_2"));
        assert!(error.contains("Did you mean 'level_2'?"));
    }

    #[test]
    fn validation_names_unknown_legacy_rules() {
        let source = GAME.replace("        ShowBasicUi,\n", "        \"show_basic_iu\",\n");
        let file: BeginnerGameFile = ron::from_str(&source).unwrap();
        let error = validate_file(&file, "game.ron").unwrap_err().to_string();

        assert!(error.contains("has unknown rule 'show_basic_iu'"));
        assert!(error.contains("Supported legacy rules:"));
        assert!(error.contains("Did you mean 'show_basic_ui'?"));
    }

    #[test]
    fn ron_parse_names_unknown_actions() {
        let source = GAME.replace("action: Attack", "action: Attak");
        let error = parse_beginner_game_source(&source, "game.ron")
            .unwrap_err()
            .to_string();

        assert!(error.contains("unknown action 'Attak'"));
        assert!(error.contains("Known actions: Attack, Pause, Reset, Reload, MenuAccept"));
        assert!(error.contains("Did you mean 'Attack'?"));
    }

    #[test]
    fn validation_explains_projectile_rules_need_projectile_prefabs() {
        let source = GAME
            .replace(
            "        Projectile((name: \"bolt\", sprite: \"bolt\", damage: 2, speed: 260.0, lifetime: 0.8)),\n",
            "",
        )
            .replace(
                r#"    actions: [
        PlayerShoots((prefab: "bolt", action: Attack, cooldown: 0.2, direction: Right, sound: Some("hit"))),
    ],
"#,
                "    actions: [],\n",
            );
        let file: BeginnerGameFile = ron::from_str(&source).unwrap();
        let error = validate_file(&file, "game.ron").unwrap_err().to_string();

        assert!(error.contains("enables projectile rules but defines no Projectile prefab"));
        assert!(error.contains("Projectile((name: \"bolt\""));
    }

    #[test]
    fn validation_explains_projectile_damage_rule_dependency() {
        let source = GAME.replace(
            "        Projectiles,\n",
            "        ProjectilesDamageEnemies,\n",
        );
        let file: BeginnerGameFile = ron::from_str(&source).unwrap();
        let error = validate_file(&file, "game.ron").unwrap_err().to_string();

        assert!(error.contains("Rule `projectiles_damage_enemies` needs the `projectiles` rule"));
        assert!(error.contains("Add `.projectiles()`"));
        assert!(error.contains("`.projectiles_damage_enemies()`"));
    }

    #[test]
    fn public_file_loader_reads_the_checked_in_game_ron() {
        let game = GameTestHarness::from_plugin(FileDataPlugin).unwrap();

        assert_eq!(game.current_map_name().as_deref(), Some("level_1"));
        assert_eq!(game.count::<crate::beginner::actors::Player>(), 1);
    }

    #[test]
    fn template_data_driven_game_file_stays_valid() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../templates/data-driven-demo/assets/game.ron");
        let source = std::fs::read_to_string(path).unwrap();
        let file: BeginnerGameFile = ron::from_str(&source).unwrap();

        validate_file(&file, "templates/data-driven-demo/assets/game.ron").unwrap();
    }

    #[test]
    fn phase12_data_driven_examples_stay_valid() {
        for relative in [
            "../../examples/data-driven-events-demo/assets/game.ron",
            "../../examples/data-driven-waves-demo/assets/game.ron",
            "../../examples/data-driven-projectiles-demo/assets/game.ron",
        ] {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
            let source = std::fs::read_to_string(&path).unwrap();
            let file: BeginnerGameFile = ron::from_str(&source).unwrap();

            validate_file_with_base(&file, relative, path.parent()).unwrap();
        }
    }

    #[test]
    fn full_data_driven_demo_game_file_stays_valid_and_loads() {
        let game = GameTestHarness::from_plugin(FullDemoDataPlugin).unwrap();

        assert_eq!(game.current_map_name().as_deref(), Some("menu"));
    }

    #[test]
    fn missing_version_defaults_to_one_and_validates() {
        let source = r#"(
            assets: (),
            prefabs: [],
            maps: [],
            rules: [],
        )"#;
        let file: BeginnerGameFile = ron::from_str(source).unwrap();
        assert_eq!(file.version, 1);
        validate_file(&file, "test.ron").unwrap();
    }

    #[test]
    fn legacy_string_controls_and_rules_still_load() {
        let source = r#"(
            version: 1,
            controls: "top_down",
            assets: (
                textures: ["player", "floor", "wall"],
            ),
            prefabs: [
                Player((name: "player", sprite: "player")),
            ],
            maps: [
                TextMap((
                    name: "level_1",
                    path: "maps/level_1.txt",
                    theme: ("floor", "wall"),
                    legend: {'P': "player", 'E': "player"},
                    start: true,
                )),
            ],
            rules: ["top_down_controls", "show_score"],
        )"#;
        let file: BeginnerGameFile = ron::from_str(source).unwrap();
        validate_file(&file, "legacy.ron").unwrap();
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

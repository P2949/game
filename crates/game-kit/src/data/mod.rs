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
    let controls = build::build_beginner_game_file(
        game,
        loaded.file,
        &loaded.label,
        loaded.full_path.parent(),
    )?;
    game.startup(move |game: &mut crate::context::StartupGameCtx<'_, '_>| {
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
    build::build_beginner_game_file(
        &mut game,
        loaded.file,
        &loaded.label,
        loaded.full_path.parent(),
    )?;
    let runtime = game.finish_for_reload()?;
    Ok(RebuiltBeginnerContent { runtime, config })
}

mod build;
mod defaults;
mod effects;
mod legacy;
mod schema;
mod validate;

use legacy::legacy_rule_kind;

pub use schema::*;

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
    validate::validate_file_with_base(&file, label, asset_base)?;
    build::build_beginner_game_file(game, file, label, asset_base)
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
    validate::validate_file_with_base(&file, &label, full_path.parent())?;
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
            Self::Trigger(prefab) => {
                validate::validate_size(label, "trigger", &prefab.name, prefab.size)?;
            }
            Self::Checkpoint(prefab) => {
                validate::validate_size(label, "checkpoint", &prefab.name, prefab.size)?;
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

#[cfg(test)]
mod tests;

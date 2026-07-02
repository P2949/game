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
pub enum AuthoringReloadLevel {
    NotSupported,
    Partial,
    Ok,
}

#[doc(hidden)]
pub type BeginnerReloadLevel = AuthoringReloadLevel;

impl AuthoringReloadLevel {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::NotSupported => "not supported",
            Self::Partial => "partial",
            Self::Ok => "ok",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AuthoringFileRuntime {
    pub(crate) path: PathBuf,
    pub(crate) last_loaded_version: u64,
    pub(crate) last_error: Option<String>,
    pub(crate) reload_level: AuthoringReloadLevel,
    identity: AuthoringReloadIdentity,
}

#[doc(hidden)]
pub type BeginnerFileRuntime = AuthoringFileRuntime;

impl AuthoringFileRuntime {
    fn new(path: PathBuf, identity: AuthoringReloadIdentity) -> Self {
        Self {
            path,
            last_loaded_version: 1,
            last_error: None,
            reload_level: AuthoringReloadLevel::Partial,
            identity,
        }
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn identity(&self) -> &AuthoringReloadIdentity {
        &self.identity
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AuthoringReloadIdentity {
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

impl AuthoringReloadIdentity {
    fn from_file(file: &AuthoringGameFile, label: &str) -> Result<Self> {
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
    fn from_file(file: &AuthoringGameFile) -> Self {
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

pub(crate) struct RebuiltAuthoringContent {
    pub(crate) runtime: ContentRuntime,
    pub(crate) config: BeginnerRuntimeConfig,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AuthoringFormat {
    Toml,
    RonLegacy,
}

impl AuthoringFormat {
    fn from_path(path: &Path) -> Result<Self> {
        let label = path.display();
        let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
            anyhow::bail!(
                "authoring file '{label}' has no file extension. Use `game.toml` for primary no-Rust authoring. RON is legacy; use `game-dev migrate-ron` if needed."
            );
        };

        match extension.to_ascii_lowercase().as_str() {
            "toml" => Ok(Self::Toml),
            "ron" => Ok(Self::RonLegacy),
            other => anyhow::bail!(
                "authoring file '{label}' has unsupported extension '.{other}'. Use `game.toml` for primary no-Rust authoring. RON is legacy; use `game-dev migrate-ron` if needed."
            ),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AuthoringLoadContext {
    pub(crate) project_root: PathBuf,
    pub(crate) asset_root: PathBuf,
    pub(crate) source_file: PathBuf,
}

impl AuthoringLoadContext {
    fn for_file(format: AuthoringFormat, source_file: PathBuf) -> Self {
        let source_parent = source_file.parent().unwrap_or_else(|| Path::new("."));
        match format {
            AuthoringFormat::Toml => {
                let project_root = source_parent.to_path_buf();
                let asset_root = project_root.join("assets");
                Self {
                    project_root,
                    asset_root,
                    source_file,
                }
            }
            AuthoringFormat::RonLegacy => {
                let asset_root = source_parent.to_path_buf();
                let project_root = asset_root
                    .parent()
                    .map(Path::to_path_buf)
                    .unwrap_or_else(|| asset_root.clone());
                Self {
                    project_root,
                    asset_root,
                    source_file,
                }
            }
        }
    }

    fn package_relative_label(&self) -> String {
        self.source_file
            .strip_prefix(&self.project_root)
            .unwrap_or(&self.source_file)
            .display()
            .to_string()
    }
}

/// Loads a primary no-Rust `game.toml` file and compiles it through the normal
/// `GameApp` asset, prefab, map, input, action, scene, audio, and rule
/// builders.
///
/// TOML packages resolve assets from an `assets/` directory next to
/// `game.toml`. Legacy `.ron` files remain supported through the old asset-root
/// search path for compatibility.
pub fn load_authoring_file(
    game: &mut GameApp<'_>,
    path: impl AsRef<Path>,
) -> Result<TopDownControls> {
    let loaded = read_authoring_game_file(path)?;
    load_authoring_file_from_loaded(game, loaded)
}

/// Loads a primary authoring file using an explicit asset root.
///
/// Relative asset roots are resolved from the authoring package root. This is
/// intended for prebuilt tools that expose an `--assets` override.
pub fn load_authoring_file_with_asset_root(
    game: &mut GameApp<'_>,
    path: impl AsRef<Path>,
    asset_root: impl AsRef<Path>,
) -> Result<TopDownControls> {
    let loaded = read_authoring_game_file_with_asset_root(path, Some(asset_root.as_ref()))?;
    load_authoring_file_from_loaded(game, loaded)
}

fn load_authoring_file_from_loaded(
    game: &mut GameApp<'_>,
    loaded: LoadedAuthoringGameFile,
) -> Result<TopDownControls> {
    let identity = AuthoringReloadIdentity::from_file(&loaded.file, &loaded.label)?;
    let runtime = AuthoringFileRuntime::new(loaded.context.source_file.clone(), identity);
    let controls = build::build_beginner_game_file(
        game,
        loaded.file,
        &loaded.label,
        Some(&loaded.context.asset_root),
    )?;
    game.startup(move |game: &mut crate::context::StartupGameCtx<'_, '_>| {
        game.insert_resource(runtime.clone());
        Ok(())
    });
    Ok(controls)
}

/// Loads a legacy beginner RON file through the primary authoring loader.
///
/// This compatibility path keeps `game.ron` resolving from the configured asset
/// root. Prefer [`load_authoring_file`] with `game.toml` for primary no-Rust
/// authoring.
pub fn load_beginner_game_file(
    game: &mut GameApp<'_>,
    path: impl AsRef<Path>,
) -> Result<TopDownControls> {
    load_authoring_file(game, path)
}

/// Validates a primary no-Rust `game.toml` file without starting runtime
/// backends.
///
/// This is the same path [`load_authoring_file`] uses, followed by the normal
/// content finalization checks for maps, prefabs, and start-map state. Legacy
/// `.ron` files remain supported.
pub fn validate_authoring_file(path: impl AsRef<Path>) -> Result<()> {
    let mut builder = GameBuilder::new();
    let mut game = GameApp::new(&mut builder);
    load_authoring_file(&mut game, path)?;
    game.finish()
}

/// Validates a primary authoring file with an explicit asset root.
pub fn validate_authoring_file_with_asset_root(
    path: impl AsRef<Path>,
    asset_root: impl AsRef<Path>,
) -> Result<()> {
    let mut builder = GameBuilder::new();
    let mut game = GameApp::new(&mut builder);
    load_authoring_file_with_asset_root(&mut game, path, asset_root)?;
    game.finish()
}

/// Result of converting a legacy `assets/game.ron` file into primary TOML.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RonToTomlMigration {
    pub toml: String,
    pub notes: Vec<String>,
}

/// Converts legacy RON authoring data into canonical primary `game.toml`.
///
/// The generated TOML is parsed before it is returned. Full validation belongs
/// to the caller, because only the caller knows where the output package and
/// asset root will live.
pub fn migrate_legacy_ron_source_to_toml(source: &str, label: &str) -> Result<RonToTomlMigration> {
    let file = parse_beginner_game_source(source, label)?;

    let mut notes =
        vec!["Converted legacy RON authoring data to primary game.toml version 2.".to_owned()];
    let toml = toml_emit::emit_authoring_game_toml(&file, &mut notes);
    toml_parse::parse_toml_authoring_source(&toml, "generated game.toml")?;
    notes.sort();
    notes.dedup();

    Ok(RonToTomlMigration { toml, notes })
}

/// Validates a legacy beginner RON file through the primary authoring validator.
pub fn validate_beginner_game_file(path: impl AsRef<Path>) -> Result<()> {
    validate_authoring_file(path)
}

pub(crate) fn rebuild_authoring_content_runtime(
    path: &Path,
    expected_identity: &AuthoringReloadIdentity,
) -> Result<RebuiltAuthoringContent> {
    let loaded = read_authoring_game_file(path)?;
    let identity = AuthoringReloadIdentity::from_file(&loaded.file, &loaded.label)?;
    expected_identity.ensure_matches(&identity, &loaded.label)?;
    let config = BeginnerRuntimeConfig::from_file(&loaded.file);

    let mut builder = GameBuilder::new();
    let mut game = GameApp::new(&mut builder);
    build::build_beginner_game_file(
        &mut game,
        loaded.file,
        &loaded.label,
        Some(&loaded.context.asset_root),
    )?;
    let runtime = game.finish_for_reload()?;
    Ok(RebuiltAuthoringContent { runtime, config })
}

mod build;
mod defaults;
mod effects;
mod legacy;
mod legacy_ron;
mod model;
mod toml_emit;
mod toml_parse;
mod toml_schema;
mod validate;

use legacy::legacy_rule_kind;
pub use legacy_ron::*;
pub(crate) use model::*;

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

struct LoadedAuthoringGameFile {
    file: AuthoringGameFile,
    label: String,
    context: AuthoringLoadContext,
}

fn read_authoring_game_file(path: impl AsRef<Path>) -> Result<LoadedAuthoringGameFile> {
    read_authoring_game_file_with_asset_root(path, None)
}

fn read_authoring_game_file_with_asset_root(
    path: impl AsRef<Path>,
    asset_root: Option<&Path>,
) -> Result<LoadedAuthoringGameFile> {
    let requested = path.as_ref();
    let format = AuthoringFormat::from_path(requested)?;
    let path_text = requested.to_string_lossy();
    let full_path = resolve_authoring_path(format, requested, &path_text)?;
    let source = std::fs::read_to_string(&full_path)
        .with_context(|| format_authoring_read_error(format, requested, &full_path))?;
    let mut context = AuthoringLoadContext::for_file(format, full_path);
    if let Some(asset_root) = asset_root {
        context.asset_root = resolve_authoring_asset_root(&context.project_root, asset_root);
    }
    let label = match format {
        AuthoringFormat::Toml => context.package_relative_label(),
        AuthoringFormat::RonLegacy => requested.display().to_string(),
    };
    let file = parse_authoring_source(&source, &label, format)?;
    validate::validate_file_with_base(&file, &label, Some(&context.asset_root))?;
    Ok(LoadedAuthoringGameFile {
        file,
        label,
        context,
    })
}

fn resolve_authoring_asset_root(project_root: &Path, asset_root: &Path) -> PathBuf {
    if asset_root.is_absolute() {
        asset_root.to_path_buf()
    } else {
        project_root.join(asset_root)
    }
}

fn resolve_authoring_path(
    format: AuthoringFormat,
    requested: &Path,
    path_text: &str,
) -> Result<PathBuf> {
    match format {
        AuthoringFormat::Toml => {
            if requested.is_absolute() {
                Ok(requested.to_path_buf())
            } else {
                Ok(std::env::current_dir()
                    .context("could not resolve current directory for game.toml")?
                    .join(requested))
            }
        }
        AuthoringFormat::RonLegacy => Ok(beginner_asset_path(path_text)),
    }
}

fn format_authoring_read_error(
    format: AuthoringFormat,
    requested: &Path,
    full_path: &Path,
) -> String {
    match format {
        AuthoringFormat::Toml => format!(
            "could not read game config '{}' (looked for '{}')",
            requested.display(),
            full_path.display()
        ),
        AuthoringFormat::RonLegacy => format!(
            "could not read legacy beginner game file 'assets/{}' (looked for '{}')",
            requested.display(),
            full_path.display()
        ),
    }
}

fn parse_authoring_source(
    source: &str,
    label: &str,
    format: AuthoringFormat,
) -> Result<AuthoringGameFile> {
    match format {
        AuthoringFormat::Toml => toml_parse::parse_toml_authoring_source(source, label),
        AuthoringFormat::RonLegacy => parse_beginner_game_source(source, label),
    }
}

fn parse_beginner_game_source(source: &str, label: &str) -> Result<AuthoringGameFile> {
    let file: BeginnerGameFile = ron::from_str(source).map_err(|error| {
        anyhow!(
            "beginner game file '{label}' is not valid RON: {error}\n\nUse controls like TopDown and rules like TopDownControls, PlayerCollectsPickups, ShowScore. They are case-sensitive."
        )
    })?;
    Ok(file.into())
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

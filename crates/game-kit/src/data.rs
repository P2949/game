//! Data-driven beginner game setup.
//!
//! `assets/game.ron` is a small layer over the public beginner builders, not a
//! second runtime. It covers conventional named assets, common beginner
//! prefabs, text maps, scene/audio hooks, standard top-down controls, and
//! declarative rules. Rust can add custom behavior after
//! [`load_beginner_game_file`] returns.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::{Context, Result, anyhow};
use game_core::input::ActionId;
use glam::vec2;
use serde::Deserialize;

use crate::app::GameApp;
use crate::beginner::rules::RulesAuthor;
use crate::input::TopDownControls;
use crate::map::beginner_asset_path;

/// Loads a beginner RON file from the asset root and compiles it through the
/// normal `GameApp` asset, prefab, map, input, action, scene, audio, and rule
/// builders.
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
    load_beginner_game_text_with_base(
        game,
        &source,
        &requested.display().to_string(),
        full_path.parent(),
    )
}

/// The file-shaped data model used by `assets/game.ron`.
#[derive(Clone, Debug, Deserialize)]
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

#[derive(Clone, Debug, Default, Deserialize)]
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

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum BeginnerControlsFile {
    Structured(BeginnerControlsKind),
    Legacy(String),
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
pub enum BeginnerControlsKind {
    TopDown,
}

#[derive(Clone, Debug, Deserialize)]
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

#[derive(Clone, Debug, Deserialize)]
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

#[derive(Clone, Debug, Deserialize)]
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

#[derive(Clone, Debug, Deserialize)]
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

#[derive(Clone, Debug, Deserialize)]
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

#[derive(Clone, Debug, Deserialize)]
pub enum DoorActionFile {
    ChangeMap(String),
    ChangeScene(String),
    RestartLevel,
}

#[derive(Clone, Debug, Deserialize)]
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

#[derive(Clone, Debug, Deserialize)]
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

#[derive(Clone, Debug, Default, Deserialize)]
pub enum SpawnPlacementFile {
    #[default]
    AtSpawner,
    NearPlayer(f32),
    AtFirstFloor,
}

#[derive(Clone, Debug, Deserialize)]
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

#[derive(Clone, Debug, Deserialize)]
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

#[derive(Clone, Debug, Deserialize)]
pub struct MeleeFile {
    pub range: f32,
    pub damage: i32,
}

#[derive(Clone, Debug, Deserialize)]
pub enum BeginnerMapFile {
    TextMap(TextMapFile),
    TextMapAuto(TextMapAutoFile),
    Tiled(TiledMapFile),
    Ldtk(LdtkMapFile),
}

#[derive(Clone, Debug, Deserialize)]
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

#[derive(Clone, Debug, Deserialize)]
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

#[derive(Clone, Debug, Deserialize)]
pub struct TiledMapFile {
    pub name: String,
    pub path: String,
    pub theme: (String, String),
    #[serde(default)]
    pub objects: BTreeMap<String, String>,
    #[serde(default)]
    pub start: bool,
}

#[derive(Clone, Debug, Deserialize)]
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

#[derive(Clone, Debug, Default, Deserialize)]
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

#[derive(Clone, Debug, Deserialize)]
pub struct SceneButtonFile {
    pub label: String,
    pub map: String,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
pub enum WinConditionFile {
    AllPickupsCollected,
    AllEnemiesDead,
}

#[derive(Clone, Debug, Default, Deserialize)]
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

#[derive(Clone, Debug, Deserialize)]
pub struct MusicPlaybackFile {
    pub track: String,
    #[serde(default = "default_music_volume")]
    pub volume: f32,
    #[serde(default)]
    pub fade_in: Option<f32>,
}

#[derive(Clone, Debug, Deserialize)]
pub enum BeginnerActionFile {
    PlayerShoots(PlayerShootsFile),
}

#[derive(Clone, Debug, Deserialize)]
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

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
pub enum ActionFile {
    #[default]
    Attack,
    Pause,
    Reset,
    Reload,
    MenuAccept,
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

#[derive(Clone, Debug, Deserialize)]
pub enum CustomRuleFile {
    Countdown(CountdownRuleFile),
}

#[derive(Clone, Debug, Deserialize)]
pub struct CountdownRuleFile {
    pub name: String,
    pub tag: String,
    pub key: String,
    pub when_zero: Vec<RuleEffectFile>,
}

#[derive(Clone, Debug, Deserialize)]
pub enum RuleEffectFile {
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
    SpawnPrefab(String),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum BeginnerRuleFile {
    Structured(BeginnerRuleKind),
    Legacy(String),
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

fn load_beginner_game_text_with_base(
    game: &mut GameApp<'_>,
    source: &str,
    label: &str,
    asset_base: Option<&Path>,
) -> Result<TopDownControls> {
    let file: BeginnerGameFile = ron::from_str(source).map_err(|error| {
        anyhow!(
            "beginner game file '{label}' is not valid RON: {error}\n\nUse controls like TopDown and rules like TopDownControls, PlayerCollectsPickups, ShowScore. They are case-sensitive."
        )
    })?;
    validate_file_with_base(&file, label, asset_base)?;

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
    build_actions(game, file.actions, controls);
    build_custom_rules(game, file.custom_rules);

    let mut rules = game.rules();
    for rule in &file.rules {
        rules = apply_rule(rules, rule.kind(label)?, controls);
    }
    rules.build();
    Ok(controls)
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
    if audio.master_volume.is_some() || audio.music_volume.is_some() || audio.sfx_volume.is_some() {
        let mut applied = false;
        let master = audio.master_volume;
        let music = audio.music_volume;
        let sfx = audio.sfx_volume;
        game.every_frame(move |game, _dt| {
            if applied {
                return;
            }
            if let Some(volume) = master {
                game.audio().set_master_volume(volume);
            }
            if let Some(volume) = music {
                game.audio().set_music_volume(volume);
            }
            if let Some(volume) = sfx {
                game.audio().set_sfx_volume(volume);
            }
            applied = true;
        });
    }

    for (scene, playback) in audio.music_on_scene {
        game.on_scene_enter(scene, move |game| {
            let music = game
                .audio()
                .play_music(&playback.track)
                .volume(playback.volume);
            if let Some(fade) = playback.fade_in {
                music.fade_in(fade);
            }
        });
    }
}

fn build_actions(
    game: &mut GameApp<'_>,
    actions: Vec<BeginnerActionFile>,
    controls: TopDownControls,
) {
    for action in actions {
        match action {
            BeginnerActionFile::PlayerShoots(shoot) => {
                let action = shoot.action.resolve(controls);
                let prefab = shoot.prefab;
                let direction = shoot.direction;
                let sound = shoot.sound;
                game.on_action_cooldown(action, shoot.cooldown, move |game| {
                    let fired = match direction {
                        ShotDirectionFile::TowardsMouse => {
                            game.player().shoot(prefab.clone()).towards_mouse()
                        }
                        ShotDirectionFile::Right => game.player().shoot(prefab.clone()).right(),
                        ShotDirectionFile::Left => game.player().shoot(prefab.clone()).left(),
                        ShotDirectionFile::Up => game.player().shoot(prefab.clone()).up(),
                        ShotDirectionFile::Down => game.player().shoot(prefab.clone()).down(),
                    };
                    if let Some(sound) = &sound {
                        fired.play_sound_named(sound);
                    }
                });
            }
        }
    }
}

fn build_custom_rules(game: &mut GameApp<'_>, custom_rules: Vec<CustomRuleFile>) {
    for custom_rule in custom_rules {
        match custom_rule {
            CustomRuleFile::Countdown(rule) => {
                let mut author = game
                    .custom_rule(rule.name)
                    .for_each_tag(rule.tag)
                    .countdown(rule.key)
                    .when_zero();
                for effect in rule.when_zero {
                    author = match effect {
                        RuleEffectFile::DamageTagged {
                            tag,
                            amount,
                            radius,
                        } => author.damage_tag(tag, amount, radius),
                        RuleEffectFile::DamagePlayer { amount, radius } => {
                            author.damage_player(amount, radius)
                        }
                        RuleEffectFile::DespawnSelf => author.despawn_self(),
                        RuleEffectFile::PlaySound(sound) => author.play_sound(sound),
                        RuleEffectFile::SpawnPrefab(prefab) => author.spawn_prefab(prefab),
                    };
                }
                author.build();
            }
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
            require_known(label, "texture", owner, texture, &textures)?;
        }
        for (owner, prefab) in map.prefab_refs() {
            require_known(label, "prefab", owner, prefab, &prefab_names)?;
        }
        validate_map_file(label, map, asset_base)?;
    }

    if let Some(flow) = &file.scene_flow {
        validate_scene_flow(label, flow, &map_names)?;
    }
    validate_audio(label, &file.audio, &music, &scene_name_refs)?;
    validate_actions(label, &file.actions, &prefab_names, &sounds)?;
    validate_custom_rules(label, &file.custom_rules, &prefab_names, &sounds, &tags)?;

    for rule in &file.rules {
        rule.kind(label)?;
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
    let mut listed = known.to_vec();
    listed.sort_unstable();
    listed.dedup();
    let suggestion = suggest(key, listed.iter().copied())
        .map(|candidate| format!(" Did you mean '{candidate}'?"))
        .unwrap_or_default();
    anyhow::bail!(
        "beginner game file '{label}' {owner} references unknown {kind} '{key}'. Known {kind}s: {}.{suggestion}",
        if listed.is_empty() {
            "(none)".to_owned()
        } else {
            listed.join(", ")
        },
    )
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
                "beginner game file '{label}' map '{map_name}' uses symbol {:?} at row {}, col {}, but no legend maps it to a prefab. Known legend symbols: {}.",
                symbol,
                row + 1,
                col + 1,
                if known_symbols.is_empty() {
                    "(none)".to_owned()
                } else {
                    known_symbols
                        .iter()
                        .map(|symbol| format!("{symbol:?}"))
                        .collect::<Vec<_>>()
                        .join(", ")
                }
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
        "beginner game file '{label}' {kind} '{owner}' references missing file 'assets/{path}' (looked for '{}')",
        full_path.display()
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

fn validate_custom_rules(
    label: &str,
    custom_rules: &[CustomRuleFile],
    prefabs: &[&str],
    sounds: &[&str],
    tags: &[&str],
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
                    tags,
                )?;
                for effect in &rule.when_zero {
                    match effect {
                        RuleEffectFile::DamageTagged { tag, radius, .. } => {
                            require_known(
                                label,
                                "tag",
                                &format!("custom rule '{}'", rule.name),
                                tag,
                                tags,
                            )?;
                            validate_radius(label, &rule.name, *radius)?;
                        }
                        RuleEffectFile::DamagePlayer { radius, .. } => {
                            validate_radius(label, &rule.name, *radius)?;
                        }
                        RuleEffectFile::DespawnSelf => {}
                        RuleEffectFile::PlaySound(sound) => {
                            require_known(
                                label,
                                "sound",
                                &format!("custom rule '{}'", rule.name),
                                sound,
                                sounds,
                            )?;
                        }
                        RuleEffectFile::SpawnPrefab(prefab) => {
                            require_known(
                                label,
                                "prefab",
                                &format!("custom rule '{}'", rule.name),
                                prefab,
                                prefabs,
                            )?;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn validate_radius(label: &str, owner: &str, radius: f32) -> Result<()> {
    if !radius.is_finite() || radius < 0.0 {
        anyhow::bail!(
            "beginner game file '{label}' custom rule '{owner}' has invalid radius {radius}; use a finite non-negative number"
        );
    }
    Ok(())
}

fn validate_rule_combinations(
    label: &str,
    rules: &[BeginnerRuleFile],
    prefabs: &[BeginnerPrefabFile],
) -> Result<()> {
    let kinds = rules
        .iter()
        .filter_map(|rule| rule.kind(label).ok())
        .collect::<Vec<_>>();
    let has_checkpoint = prefabs
        .iter()
        .any(|prefab| matches!(prefab, BeginnerPrefabFile::Checkpoint(_)));
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
                let suggestion = suggest(name, supported.into_iter())
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
    fn kind(&self, label: &str) -> Result<BeginnerRuleKind> {
        match self {
            Self::Structured(kind) => Ok(*kind),
            Self::Legacy(name) => legacy_rule_kind(name, label),
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
            let suggestion = suggest(other, LEGACY_RULES.iter().copied())
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
    use super::{BeginnerGameFile, load_beginner_game_text, validate_file};
    use crate::app::{GameApp, GamePlugin};
    use crate::harness::GameTestHarness;
    use anyhow::Result;

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
        Trigger((name: "danger", size: (32.0, 32.0), visible_debug: Some("spawner_debug"), tags: ["danger"])),
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
    fn validation_names_unknown_legend_prefabs_and_offers_a_suggestion() {
        let source = GAME.replace("'E': \"slime\"", "'E': \"slimee\"");
        let file: BeginnerGameFile = ron::from_str(&source).unwrap();
        let error = validate_file(&file, "game.ron").unwrap_err().to_string();

        assert!(error.contains("references unknown prefab 'slimee'"));
        assert!(error.contains("Did you mean 'slime'?"));
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

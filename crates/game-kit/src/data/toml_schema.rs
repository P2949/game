use std::collections::BTreeMap;

use anyhow::{Result, anyhow, bail};
use serde::Deserialize;

use crate::diagnostics::closest_name;

use super::{
    ActionFile, AudioFile, AuthoringGameFile, BeginnerActionFile, BeginnerAssetsFile,
    BeginnerControlsFile, BeginnerControlsKind, BeginnerMapFile, BeginnerPrefabFile,
    BeginnerRuleFile, BeginnerRuleKind, BeginnerScriptRuleFile, CheckpointPrefabFile,
    CountdownRuleFile, CustomRuleFile, DoorActionFile, DoorPrefabFile, EnemyPrefabFile,
    LdtkMapFile, MeleeFile, MusicPlaybackFile, PickupPrefabFile, PlayerPrefabFile,
    PlayerShootsFile, ProjectilePrefabFile, RuleConditionFile, RuleEffectFile, SceneButtonFile,
    SceneFlowFile, ShotDirectionFile, SpawnPlacementFile, SpawnerPrefabFile, TextMapFile,
    TiledMapFile, TriggerPrefabFile, WinConditionFile,
};

#[derive(Debug, Deserialize)]
pub(super) struct GameTomlFile {
    #[serde(default = "default_toml_version")]
    version: u32,
    #[serde(default)]
    game: GameTomlMetadata,
    #[serde(default)]
    assets: AssetsToml,
    #[serde(default)]
    controls: ControlsToml,
    #[serde(default)]
    prefab: Vec<PrefabToml>,
    #[serde(default)]
    map: Vec<MapToml>,
    #[serde(default)]
    scene_flow: Option<SceneFlowToml>,
    #[serde(default)]
    audio: AudioToml,
    #[serde(default)]
    action: Vec<ActionToml>,
    #[serde(default)]
    custom_rule: Vec<CustomRuleToml>,
    #[serde(default)]
    rules: RulesToml,
    #[serde(default)]
    rule: Vec<RuleToml>,
}

impl GameTomlFile {
    pub(super) fn into_authoring(self, label: &str) -> Result<AuthoringGameFile> {
        if self.version != 2 {
            bail!(
                "game config '{label}' uses unsupported game.toml version {}. Supported version: 2",
                self.version
            );
        }

        let mut rules = self
            .rules
            .enabled
            .iter()
            .map(|name| rule_name_from_kebab(name, label).map(BeginnerRuleFile::Structured))
            .collect::<Result<Vec<_>>>()?;
        rules.extend(
            self.rule
                .into_iter()
                .map(|rule| rule.into_authoring(label))
                .collect::<Result<Vec<_>>>()?,
        );

        let scene_flow = match (self.scene_flow, self.game.start_map) {
            (Some(flow), _) => Some(flow.into_authoring(label)?),
            (None, Some(start_map)) => Some(SceneFlowFile {
                game: Some(start_map),
                ..SceneFlowFile::default()
            }),
            (None, None) => None,
        };

        Ok(AuthoringGameFile {
            version: super::defaults::default_beginner_game_version(),
            assets: self.assets.into_authoring(),
            controls: self.controls.into_authoring(label)?,
            prefabs: self
                .prefab
                .into_iter()
                .map(PrefabToml::into_authoring)
                .collect::<Result<Vec<_>>>()?,
            maps: self
                .map
                .into_iter()
                .map(MapToml::into_authoring)
                .collect::<Result<Vec<_>>>()?,
            scene_flow,
            audio: self.audio.into_authoring(),
            actions: self
                .action
                .into_iter()
                .map(|action| action.into_authoring(label))
                .collect::<Result<Vec<_>>>()?,
            custom_rules: self
                .custom_rule
                .into_iter()
                .map(|rule| rule.into_authoring(label))
                .collect::<Result<Vec<_>>>()?,
            rules,
        })
    }
}

fn default_toml_version() -> u32 {
    2
}

#[derive(Debug, Default, Deserialize)]
struct GameTomlMetadata {
    #[serde(default)]
    start_map: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct AssetsToml {
    #[serde(default)]
    textures: Vec<String>,
    #[serde(default)]
    sounds: Vec<String>,
    #[serde(default)]
    music: Vec<String>,
    #[serde(default)]
    animation_sheets: Vec<String>,
}

impl AssetsToml {
    fn into_authoring(self) -> BeginnerAssetsFile {
        BeginnerAssetsFile {
            textures: self.textures,
            sounds: self.sounds,
            music: self.music,
            animation_sheets: self.animation_sheets,
        }
    }
}

#[derive(Debug, Default, Deserialize)]
struct ControlsToml {
    #[serde(default)]
    preset: Option<String>,
}

impl ControlsToml {
    fn into_authoring(self, label: &str) -> Result<BeginnerControlsFile> {
        match self.preset.as_deref().unwrap_or("top-down") {
            "top-down" => Ok(BeginnerControlsFile::Structured(
                BeginnerControlsKind::TopDown,
            )),
            preset => {
                let known = ["top-down"];
                let suggestion = closest_name(preset, known.into_iter())
                    .map(|candidate| format!(" Did you mean \"{candidate}\"?"))
                    .unwrap_or_default();
                bail!(
                    "game config '{label}' has unsupported controls.preset \"{preset}\". Known presets: top-down.{suggestion}"
                )
            }
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
enum PrefabToml {
    Player(ActorPrefabToml),
    Enemy(EnemyToml),
    Pickup(PickupToml),
    Door(DoorToml),
    Projectile(ProjectileToml),
    Spawner(SpawnerToml),
    Trigger(TriggerToml),
    Checkpoint(CheckpointToml),
}

impl PrefabToml {
    fn into_authoring(self) -> Result<BeginnerPrefabFile> {
        Ok(match self {
            Self::Player(prefab) => BeginnerPrefabFile::Player(PlayerPrefabFile {
                name: prefab.name,
                sprite: prefab.sprite,
                animation_sheet: prefab.animation_sheet,
                speed: prefab
                    .speed
                    .unwrap_or_else(super::defaults::default_player_speed),
                health: prefab
                    .health
                    .unwrap_or_else(super::defaults::default_player_health),
                melee: prefab.melee.map(Into::into),
                tags: prefab.tags,
                data: prefab.data,
            }),
            Self::Enemy(prefab) => BeginnerPrefabFile::Enemy(EnemyPrefabFile {
                name: prefab.actor.name,
                sprite: prefab.actor.sprite,
                animation_sheet: prefab.actor.animation_sheet,
                speed: prefab
                    .actor
                    .speed
                    .unwrap_or_else(super::defaults::default_enemy_speed),
                health: prefab
                    .actor
                    .health
                    .unwrap_or_else(super::defaults::default_enemy_health),
                chase_player: prefab.chase_player,
                melee: prefab.actor.melee.map(Into::into),
                drops: prefab.drops,
                drop_chance: prefab.drop_chance,
                despawn_after_death_animation: prefab.despawn_after_death_animation,
                tags: prefab.actor.tags,
                data: prefab.actor.data,
            }),
            Self::Pickup(prefab) => BeginnerPrefabFile::Pickup(PickupPrefabFile {
                name: prefab.name,
                sprite: prefab.sprite,
                score: prefab
                    .score
                    .unwrap_or_else(super::defaults::default_pickup_score),
                heal_player: prefab.heal_player,
                sound: prefab.sound,
                despawn_on_collect: prefab.despawn_on_collect.unwrap_or(true),
                tags: prefab.tags,
                data: prefab.data,
            }),
            Self::Door(prefab) => BeginnerPrefabFile::Door(DoorPrefabFile {
                name: prefab.name,
                sprite: prefab.sprite,
                action: prefab.action.into_authoring()?,
                requires_all_enemies_dead: prefab.requires_all_enemies_dead,
                tags: prefab.tags,
                data: prefab.data,
            }),
            Self::Projectile(prefab) => {
                if prefab.old_lifetime.is_some() {
                    bail!(
                        "projectile '{}' uses lifetime = ...; use duration = ... in primary TOML",
                        prefab.name
                    );
                }
                BeginnerPrefabFile::Projectile(ProjectilePrefabFile {
                    name: prefab.name,
                    sprite: prefab.sprite,
                    animation_sheet: prefab.animation_sheet,
                    damage: prefab
                        .damage
                        .unwrap_or_else(super::defaults::default_projectile_damage),
                    speed: prefab
                        .speed
                        .unwrap_or_else(super::defaults::default_projectile_speed),
                    lifetime: prefab
                        .duration
                        .unwrap_or_else(super::defaults::default_projectile_lifetime),
                    despawn_on_hit: prefab.despawn_on_hit.unwrap_or(true),
                    tags: prefab.tags,
                    data: prefab.data,
                })
            }
            Self::Spawner(prefab) => BeginnerPrefabFile::Spawner(SpawnerPrefabFile {
                name: prefab.name,
                spawn: prefab.spawn,
                every_seconds: prefab
                    .every_seconds
                    .unwrap_or_else(super::defaults::default_spawn_every),
                max_alive: prefab.max_alive,
                placement: prefab.placement.into_authoring(prefab.placement_radius),
            }),
            Self::Trigger(prefab) => BeginnerPrefabFile::Trigger(TriggerPrefabFile {
                name: prefab.name,
                size: tuple2(prefab.size.unwrap_or([32.0, 32.0])),
                visible_debug: prefab.visible_debug,
                tags: prefab.tags,
                data: prefab.data,
            }),
            Self::Checkpoint(prefab) => BeginnerPrefabFile::Checkpoint(CheckpointPrefabFile {
                name: prefab.name,
                sprite: prefab.sprite,
                size: tuple2(prefab.size.unwrap_or([32.0, 32.0])),
                tags: prefab.tags,
                data: prefab.data,
            }),
        })
    }
}

#[derive(Debug, Deserialize)]
struct ActorPrefabToml {
    name: String,
    sprite: String,
    #[serde(default)]
    animation_sheet: Option<String>,
    #[serde(default)]
    speed: Option<f32>,
    #[serde(default)]
    health: Option<i32>,
    #[serde(default)]
    melee: Option<MeleeToml>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    data: BTreeMap<String, f32>,
}

#[derive(Debug, Deserialize)]
struct EnemyToml {
    #[serde(flatten)]
    actor: ActorPrefabToml,
    #[serde(default)]
    chase_player: bool,
    #[serde(default)]
    drops: Option<String>,
    #[serde(default)]
    drop_chance: Option<f32>,
    #[serde(default)]
    despawn_after_death_animation: bool,
}

#[derive(Debug, Deserialize)]
struct PickupToml {
    name: String,
    sprite: String,
    #[serde(default)]
    score: Option<i32>,
    #[serde(default)]
    heal_player: Option<i32>,
    #[serde(default)]
    sound: Option<String>,
    #[serde(default)]
    despawn_on_collect: Option<bool>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    data: BTreeMap<String, f32>,
}

#[derive(Debug, Deserialize)]
struct DoorToml {
    name: String,
    sprite: String,
    #[serde(flatten)]
    action: DoorActionToml,
    #[serde(default)]
    requires_all_enemies_dead: bool,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    data: BTreeMap<String, f32>,
}

#[derive(Debug, Deserialize)]
struct DoorActionToml {
    action: String,
    #[serde(default)]
    map: Option<String>,
    #[serde(default)]
    scene: Option<String>,
}

impl DoorActionToml {
    fn into_authoring(self) -> Result<DoorActionFile> {
        match self.action.as_str() {
            "change-map" => self
                .map
                .map(DoorActionFile::ChangeMap)
                .ok_or_else(|| anyhow!("door action \"change-map\" needs map = \"...\"")),
            "change-scene" => self
                .scene
                .map(DoorActionFile::ChangeScene)
                .ok_or_else(|| anyhow!("door action \"change-scene\" needs scene = \"...\"")),
            "restart-level" => Ok(DoorActionFile::RestartLevel),
            other => bail!(
                "unknown door action \"{other}\". Known actions: change-map, change-scene, restart-level."
            ),
        }
    }
}

#[derive(Debug, Deserialize)]
struct ProjectileToml {
    name: String,
    sprite: String,
    #[serde(default)]
    animation_sheet: Option<String>,
    #[serde(default)]
    damage: Option<i32>,
    #[serde(default)]
    speed: Option<f32>,
    #[serde(default)]
    duration: Option<f32>,
    #[serde(default, rename = "lifetime")]
    old_lifetime: Option<f32>,
    #[serde(default)]
    despawn_on_hit: Option<bool>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    data: BTreeMap<String, f32>,
}

#[derive(Debug, Deserialize)]
struct SpawnerToml {
    name: String,
    spawn: String,
    #[serde(default)]
    every_seconds: Option<f32>,
    #[serde(default)]
    max_alive: Option<usize>,
    #[serde(default)]
    placement: PlacementToml,
    #[serde(default)]
    placement_radius: Option<f32>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum PlacementToml {
    #[default]
    AtSpawner,
    NearPlayer,
    AtFirstFloor,
}

impl PlacementToml {
    fn into_authoring(self, radius: Option<f32>) -> SpawnPlacementFile {
        match self {
            Self::AtSpawner => SpawnPlacementFile::AtSpawner,
            Self::NearPlayer => SpawnPlacementFile::NearPlayer(radius.unwrap_or(96.0)),
            Self::AtFirstFloor => SpawnPlacementFile::AtFirstFloor,
        }
    }
}

#[derive(Debug, Deserialize)]
struct TriggerToml {
    name: String,
    #[serde(default)]
    size: Option<[f32; 2]>,
    #[serde(default)]
    visible_debug: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    data: BTreeMap<String, f32>,
}

#[derive(Debug, Deserialize)]
struct CheckpointToml {
    name: String,
    sprite: String,
    #[serde(default)]
    size: Option<[f32; 2]>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    data: BTreeMap<String, f32>,
}

#[derive(Debug, Deserialize)]
struct MeleeToml {
    range: f32,
    damage: i32,
}

impl From<MeleeToml> for MeleeFile {
    fn from(value: MeleeToml) -> Self {
        Self {
            range: value.range,
            damage: value.damage,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
enum MapToml {
    Text(TextMapToml),
    Tiled(TiledMapToml),
    Ldtk(LdtkMapToml),
}

impl MapToml {
    fn into_authoring(self) -> Result<BeginnerMapFile> {
        Ok(match self {
            Self::Text(map) => BeginnerMapFile::TextMap(TextMapFile {
                name: map.name.clone(),
                path: strip_assets_prefix(
                    map.file
                        .unwrap_or_else(|| format!("assets/maps/{}.txt", map.name)),
                ),
                theme: (map.floor, map.wall),
                tile_size: map
                    .tile_size
                    .unwrap_or_else(super::defaults::default_tile_size),
                legend: char_legend(map.legend)?,
                start: map.start,
            }),
            Self::Tiled(map) => BeginnerMapFile::Tiled(TiledMapFile {
                name: map.name,
                path: strip_assets_prefix(map.file),
                theme: (map.floor, map.wall),
                objects: map.objects,
                start: map.start,
            }),
            Self::Ldtk(map) => BeginnerMapFile::Ldtk(LdtkMapFile {
                name: map.name,
                path: strip_assets_prefix(map.file),
                level: map.level,
                theme: (map.floor, map.wall),
                entities: map.entities,
                start: map.start,
            }),
        })
    }
}

#[derive(Debug, Deserialize)]
struct TextMapToml {
    name: String,
    #[serde(default)]
    file: Option<String>,
    floor: String,
    wall: String,
    #[serde(default)]
    tile_size: Option<f32>,
    #[serde(default)]
    legend: BTreeMap<String, String>,
    #[serde(default)]
    start: bool,
}

#[derive(Debug, Deserialize)]
struct TiledMapToml {
    name: String,
    file: String,
    floor: String,
    wall: String,
    #[serde(default)]
    objects: BTreeMap<String, String>,
    #[serde(default)]
    start: bool,
}

#[derive(Debug, Deserialize)]
struct LdtkMapToml {
    name: String,
    file: String,
    level: String,
    floor: String,
    wall: String,
    #[serde(default)]
    entities: BTreeMap<String, String>,
    #[serde(default)]
    start: bool,
}

#[derive(Debug, Default, Deserialize)]
struct SceneFlowToml {
    #[serde(default)]
    menu: Option<String>,
    #[serde(default)]
    game: Option<String>,
    #[serde(default)]
    game_over: Option<String>,
    #[serde(default)]
    win: Option<String>,
    #[serde(default)]
    menu_text: Option<String>,
    #[serde(default)]
    menu_button: Option<SceneButtonToml>,
    #[serde(default)]
    game_over_text: Option<String>,
    #[serde(default)]
    game_over_button: Option<String>,
    #[serde(default)]
    win_text: Option<String>,
    #[serde(default)]
    win_button: Option<String>,
    #[serde(default)]
    start_on: Option<String>,
    #[serde(default)]
    restart_on: Option<String>,
    #[serde(default)]
    win_condition: Option<String>,
}

impl SceneFlowToml {
    fn into_authoring(self, label: &str) -> Result<SceneFlowFile> {
        Ok(SceneFlowFile {
            menu: self.menu,
            game: self.game,
            game_over: self.game_over,
            win: self.win,
            menu_text: self.menu_text,
            menu_button: self.menu_button.map(Into::into),
            game_over_text: self.game_over_text,
            game_over_button: self.game_over_button,
            win_text: self.win_text,
            win_button: self.win_button,
            start_on: self
                .start_on
                .as_deref()
                .map(|name| action_from_kebab(name, label))
                .transpose()?,
            restart_on: self
                .restart_on
                .as_deref()
                .map(|name| action_from_kebab(name, label))
                .transpose()?,
            win_condition: self
                .win_condition
                .as_deref()
                .map(|name| win_condition_from_kebab(name, label))
                .transpose()?,
        })
    }
}

#[derive(Debug, Deserialize)]
struct SceneButtonToml {
    label: String,
    map: String,
}

impl From<SceneButtonToml> for SceneButtonFile {
    fn from(value: SceneButtonToml) -> Self {
        Self {
            label: value.label,
            map: value.map,
        }
    }
}

#[derive(Debug, Default, Deserialize)]
struct AudioToml {
    #[serde(default)]
    music_on_scene: BTreeMap<String, MusicPlaybackToml>,
    #[serde(default)]
    master_volume: Option<f32>,
    #[serde(default)]
    music_volume: Option<f32>,
    #[serde(default)]
    sfx_volume: Option<f32>,
}

impl AudioToml {
    fn into_authoring(self) -> AudioFile {
        AudioFile {
            music_on_scene: self
                .music_on_scene
                .into_iter()
                .map(|(scene, playback)| (scene, playback.into()))
                .collect(),
            master_volume: self.master_volume,
            music_volume: self.music_volume,
            sfx_volume: self.sfx_volume,
        }
    }
}

#[derive(Debug, Deserialize)]
struct MusicPlaybackToml {
    track: String,
    #[serde(default)]
    volume: Option<f32>,
    #[serde(default)]
    fade_in: Option<f32>,
}

impl From<MusicPlaybackToml> for MusicPlaybackFile {
    fn from(value: MusicPlaybackToml) -> Self {
        Self {
            track: value.track,
            volume: value
                .volume
                .unwrap_or_else(super::defaults::default_music_volume),
            fade_in: value.fade_in,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
enum ActionToml {
    PlayerShoots(PlayerShootsToml),
}

impl ActionToml {
    fn into_authoring(self, label: &str) -> Result<BeginnerActionFile> {
        match self {
            Self::PlayerShoots(action) => Ok(BeginnerActionFile::PlayerShoots(PlayerShootsFile {
                prefab: action.prefab,
                action: action
                    .action
                    .as_deref()
                    .map(|name| action_from_kebab(name, label))
                    .transpose()?
                    .unwrap_or_default(),
                cooldown: action
                    .cooldown
                    .unwrap_or_else(super::defaults::default_shoot_cooldown),
                direction: action
                    .direction
                    .as_deref()
                    .map(|name| shot_direction_from_kebab(name, label))
                    .transpose()?
                    .unwrap_or_default(),
                sound: action.sound,
            })),
        }
    }
}

#[derive(Debug, Deserialize)]
struct PlayerShootsToml {
    prefab: String,
    #[serde(default)]
    action: Option<String>,
    #[serde(default)]
    cooldown: Option<f32>,
    #[serde(default)]
    direction: Option<String>,
    #[serde(default)]
    sound: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
enum CustomRuleToml {
    Countdown(CountdownRuleToml),
}

impl CustomRuleToml {
    fn into_authoring(self, label: &str) -> Result<CustomRuleFile> {
        match self {
            Self::Countdown(rule) => Ok(CustomRuleFile::Countdown(CountdownRuleFile {
                name: rule.name,
                tag: rule.tag,
                key: rule.key,
                when_zero: rule
                    .when_zero
                    .into_iter()
                    .map(|effect| effect.into_authoring(label))
                    .collect::<Result<Vec<_>>>()?,
            })),
        }
    }
}

#[derive(Debug, Deserialize)]
struct CountdownRuleToml {
    name: String,
    tag: String,
    key: String,
    #[serde(default)]
    when_zero: Vec<EffectToml>,
}

#[derive(Debug, Default, Deserialize)]
struct RulesToml {
    #[serde(default)]
    enabled: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct RuleToml {
    #[serde(default)]
    when: String,
    #[serde(default)]
    on: Option<String>,
    #[serde(default)]
    every_seconds: Option<f32>,
    #[serde(default)]
    prefab: Option<String>,
    #[serde(default)]
    score: Option<i32>,
    #[serde(default)]
    health: Option<i32>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    seconds: Option<f32>,
    #[serde(default)]
    map: Option<String>,
    #[serde(default)]
    scene: Option<String>,
    #[serde(default)]
    tag: Option<String>,
    #[serde(default)]
    action: Option<String>,
    #[serde(default)]
    then: Vec<EffectToml>,
}

impl RuleToml {
    fn into_authoring(self, label: &str) -> Result<BeginnerRuleFile> {
        enum PendingScriptRule {
            When(RuleConditionFile),
            OnEnemyDeath(String),
            EverySeconds(f32),
            OnScoreReaches(i32),
        }

        let pending = if let Some(seconds) = self.every_seconds {
            PendingScriptRule::EverySeconds(seconds)
        } else if let Some(event) = self.on.as_deref() {
            match event {
                "enemy-death" => PendingScriptRule::OnEnemyDeath(required_rule_field(
                    self.prefab.clone(),
                    "prefab",
                    "enemy-death",
                )?),
                "score-reaches" => PendingScriptRule::OnScoreReaches(required_rule_field(
                    self.score,
                    "score",
                    "score-reaches",
                )?),
                other => bail!(
                    "game config '{label}' has unknown rule event \"{other}\". Known events: enemy-death, score-reaches."
                ),
            }
        } else {
            if self.when.is_empty() {
                bail!("game config '{label}' has a [[rule]] without when, on, or every_seconds");
            }
            PendingScriptRule::When(condition_from_rule_fields(&self, label)?)
        };

        let effects = self
            .then
            .into_iter()
            .map(|effect| effect.into_authoring(label))
            .collect::<Result<Vec<_>>>()?;
        Ok(BeginnerRuleFile::Script(match pending {
            PendingScriptRule::When(condition) => {
                BeginnerScriptRuleFile::When { condition, effects }
            }
            PendingScriptRule::OnEnemyDeath(prefab) => {
                BeginnerScriptRuleFile::OnEnemyDeath { prefab, effects }
            }
            PendingScriptRule::EverySeconds(seconds) => {
                BeginnerScriptRuleFile::EverySeconds { seconds, effects }
            }
            PendingScriptRule::OnScoreReaches(score) => {
                BeginnerScriptRuleFile::OnScoreReaches { score, effects }
            }
        }))
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "action", rename_all = "kebab-case")]
enum EffectToml {
    AddScore {
        amount: i32,
    },
    SetScore {
        score: i32,
    },
    DamageTagged {
        tag: String,
        amount: i32,
        #[serde(default)]
        radius: f32,
    },
    DamagePlayer {
        amount: i32,
        #[serde(default)]
        radius: f32,
    },
    DespawnSelf,
    PlaySound {
        sound: String,
    },
    PlayMusic {
        music: String,
    },
    StopMusic,
    SpawnPrefab {
        prefab: String,
    },
    SpawnNearPlayer {
        prefab: String,
        radius: f32,
    },
    ChangeScene {
        scene: String,
    },
    ChangeMap {
        map: String,
    },
    RestartCurrentMap,
    ShowUiText {
        text: String,
    },
    HealPlayer {
        amount: i32,
    },
    SetData {
        tag: String,
        key: String,
        value: f32,
    },
    DespawnTagged {
        tag: String,
    },
}

impl EffectToml {
    fn into_authoring(self, _label: &str) -> Result<RuleEffectFile> {
        Ok(match self {
            Self::AddScore { amount } => RuleEffectFile::AddScore(amount),
            Self::SetScore { score } => RuleEffectFile::SetScore(score),
            Self::DamageTagged {
                tag,
                amount,
                radius,
            } => RuleEffectFile::DamageTagged {
                tag,
                amount,
                radius,
            },
            Self::DamagePlayer { amount, radius } => {
                RuleEffectFile::DamagePlayer { amount, radius }
            }
            Self::DespawnSelf => RuleEffectFile::DespawnSelf,
            Self::PlaySound { sound } => RuleEffectFile::PlaySound(sound),
            Self::PlayMusic { music } => RuleEffectFile::PlayMusic(music),
            Self::StopMusic => RuleEffectFile::StopMusic,
            Self::SpawnPrefab { prefab } => RuleEffectFile::SpawnPrefab(prefab),
            Self::SpawnNearPlayer { prefab, radius } => {
                RuleEffectFile::SpawnNearPlayer { prefab, radius }
            }
            Self::ChangeScene { scene } => RuleEffectFile::ChangeScene(scene),
            Self::ChangeMap { map } => RuleEffectFile::ChangeMap(map),
            Self::RestartCurrentMap => RuleEffectFile::RestartCurrentMap,
            Self::ShowUiText { text } => RuleEffectFile::ShowUiText(text),
            Self::HealPlayer { amount } => RuleEffectFile::HealPlayer(amount),
            Self::SetData { tag, key, value } => RuleEffectFile::SetData { tag, key, value },
            Self::DespawnTagged { tag } => RuleEffectFile::DespawnTagged(tag),
        })
    }
}

fn tuple2(value: [f32; 2]) -> (f32, f32) {
    (value[0], value[1])
}

fn char_legend(legend: BTreeMap<String, String>) -> Result<BTreeMap<char, String>> {
    legend
        .into_iter()
        .map(|(symbol, prefab)| {
            let mut chars = symbol.chars();
            let Some(symbol) = chars.next() else {
                bail!("map legend keys must be one visible character");
            };
            if chars.next().is_some() {
                bail!("map legend key \"{symbol}\" must be one character");
            }
            Ok((symbol, prefab))
        })
        .collect()
}

fn strip_assets_prefix(path: String) -> String {
    path.strip_prefix("assets/")
        .map(str::to_owned)
        .unwrap_or(path)
}

fn rule_name_from_kebab(name: &str, label: &str) -> Result<BeginnerRuleKind> {
    let known = RULE_NAME_MAP.iter().map(|(name, _)| *name);
    RULE_NAME_MAP
        .iter()
        .find(|(candidate, _)| *candidate == name)
        .map(|(_, kind)| *kind)
        .ok_or_else(|| {
            let suggestion = closest_name(name, known)
                .map(|candidate| format!(" Did you mean \"{candidate}\"?"))
                .unwrap_or_default();
            anyhow!(
                "game config '{label}' has unknown rule \"{name}\". Known rules: {}.{suggestion}",
                RULE_NAME_MAP
                    .iter()
                    .map(|(name, _)| *name)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })
}

fn action_from_kebab(name: &str, label: &str) -> Result<ActionFile> {
    match name {
        "attack" => Ok(ActionFile::Attack),
        "pause" => Ok(ActionFile::Pause),
        "reset" => Ok(ActionFile::Reset),
        "reload" => Ok(ActionFile::Reload),
        "menu-accept" => Ok(ActionFile::MenuAccept),
        other => bail!(
            "game config '{label}' has unknown action \"{other}\". Known actions: attack, pause, reset, reload, menu-accept."
        ),
    }
}

fn shot_direction_from_kebab(name: &str, label: &str) -> Result<ShotDirectionFile> {
    match name {
        "towards-mouse" => Ok(ShotDirectionFile::TowardsMouse),
        "right" => Ok(ShotDirectionFile::Right),
        "left" => Ok(ShotDirectionFile::Left),
        "up" => Ok(ShotDirectionFile::Up),
        "down" => Ok(ShotDirectionFile::Down),
        other => bail!(
            "game config '{label}' has unknown shot direction \"{other}\". Known directions: towards-mouse, right, left, up, down."
        ),
    }
}

fn win_condition_from_kebab(name: &str, label: &str) -> Result<WinConditionFile> {
    match name {
        "all-pickups-collected" => Ok(WinConditionFile::AllPickupsCollected),
        "all-enemies-dead" => Ok(WinConditionFile::AllEnemiesDead),
        other => bail!(
            "game config '{label}' has unknown win condition \"{other}\". Known win conditions: all-pickups-collected, all-enemies-dead."
        ),
    }
}

fn condition_from_rule_fields(rule: &RuleToml, label: &str) -> Result<RuleConditionFile> {
    match rule.when.as_str() {
        "all-enemies-dead" => Ok(RuleConditionFile::AllEnemiesDead),
        "all-pickups-collected" => Ok(RuleConditionFile::AllPickupsCollected),
        "score-at-least" => Ok(RuleConditionFile::ScoreAtLeast(required_rule_field(
            rule.score,
            "score",
            "score-at-least",
        )?)),
        "player-health-below" => Ok(RuleConditionFile::PlayerHealthBelow(required_rule_field(
            rule.health,
            "health",
            "player-health-below",
        )?)),
        "timer-reached" => Ok(RuleConditionFile::TimerReached {
            name: required_rule_field(rule.name.clone(), "name", "timer-reached")?,
            seconds: required_rule_field(rule.seconds, "seconds", "timer-reached")?,
        }),
        "map-is" => Ok(RuleConditionFile::MapIs(required_rule_field(
            rule.map.clone(),
            "map",
            "map-is",
        )?)),
        "scene-is" => Ok(RuleConditionFile::SceneIs(required_rule_field(
            rule.scene.clone(),
            "scene",
            "scene-is",
        )?)),
        "tag-count-zero" => Ok(RuleConditionFile::TagCountZero(required_rule_field(
            rule.tag.clone(),
            "tag",
            "tag-count-zero",
        )?)),
        "action-pressed" => Ok(RuleConditionFile::ActionPressed(action_from_kebab(
            &required_rule_field(rule.action.clone(), "action", "action-pressed")?,
            label,
        )?)),
        other => bail!(
            "game config '{label}' has unknown rule condition \"{other}\". Known conditions: all-enemies-dead, all-pickups-collected, score-at-least, player-health-below, timer-reached, map-is, scene-is, tag-count-zero, action-pressed."
        ),
    }
}

fn required_rule_field<T>(value: Option<T>, field: &str, kind: &str) -> Result<T> {
    value.ok_or_else(|| anyhow!("rule {kind:?} needs {field} = ..."))
}

const RULE_NAME_MAP: &[(&str, BeginnerRuleKind)] = &[
    ("top-down-controls", BeginnerRuleKind::TopDownControls),
    (
        "player-collects-pickups",
        BeginnerRuleKind::PlayerCollectsPickups,
    ),
    (
        "enemies-damage-player",
        BeginnerRuleKind::EnemiesDamagePlayer,
    ),
    ("dead-enemies-despawn", BeginnerRuleKind::DeadEnemiesDespawn),
    ("enemy-drops", BeginnerRuleKind::EnemyDrops),
    ("projectiles", BeginnerRuleKind::Projectiles),
    ("projectiles-move", BeginnerRuleKind::ProjectilesMove),
    (
        "projectiles-expire-after-duration",
        BeginnerRuleKind::ProjectilesExpireAfterLifetime,
    ),
    (
        "projectiles-expire-after-lifetime",
        BeginnerRuleKind::ProjectilesExpireAfterLifetime,
    ),
    (
        "projectiles-damage-enemies",
        BeginnerRuleKind::ProjectilesDamageEnemies,
    ),
    (
        "projectiles-despawn-on-hit",
        BeginnerRuleKind::ProjectilesDespawnOnHit,
    ),
    (
        "projectile-impact-animation-before-despawn",
        BeginnerRuleKind::ProjectileImpactAnimationBeforeDespawn,
    ),
    (
        "spawners-spawn-prefabs",
        BeginnerRuleKind::SpawnersSpawnPrefabs,
    ),
    ("doors-change-maps", BeginnerRuleKind::DoorsChangeMaps),
    (
        "player-activates-checkpoints",
        BeginnerRuleKind::PlayerActivatesCheckpoints,
    ),
    (
        "respawn-at-checkpoint",
        BeginnerRuleKind::RespawnAtCheckpoint,
    ),
    (
        "camera-follows-player",
        BeginnerRuleKind::CameraFollowsPlayer,
    ),
    ("pause-and-reset", BeginnerRuleKind::PauseAndReset),
    ("show-basic-ui", BeginnerRuleKind::ShowBasicUi),
    ("show-score", BeginnerRuleKind::ShowScore),
    ("show-enemy-count", BeginnerRuleKind::ShowEnemyCount),
    ("show-player-health", BeginnerRuleKind::ShowPlayerHealth),
    ("show-menu", BeginnerRuleKind::ShowMenu),
    ("show-pause-menu", BeginnerRuleKind::ShowPauseMenu),
    ("show-game-over-panel", BeginnerRuleKind::ShowGameOverPanel),
    ("show-win-panel", BeginnerRuleKind::ShowWinPanel),
    (
        "win-when-all-pickups-collected",
        BeginnerRuleKind::WinWhenAllPickupsCollected,
    ),
    (
        "win-when-all-enemies-dead",
        BeginnerRuleKind::WinWhenAllEnemiesDead,
    ),
    (
        "animate-enemies-by-movement",
        BeginnerRuleKind::AnimateEnemiesByMovement,
    ),
    (
        "animate-player-directionally",
        BeginnerRuleKind::AnimatePlayerDirectionally,
    ),
    (
        "animate-enemies-directionally",
        BeginnerRuleKind::AnimateEnemiesDirectionally,
    ),
    (
        "animate-attacks-directionally",
        BeginnerRuleKind::AnimateAttacksDirectionally,
    ),
    (
        "dead-enemies-play-death-animation",
        BeginnerRuleKind::DeadEnemiesPlayDeathAnimation,
    ),
    (
        "dead-enemies-despawn-after-animation",
        BeginnerRuleKind::DeadEnemiesDespawnAfterAnimation,
    ),
];

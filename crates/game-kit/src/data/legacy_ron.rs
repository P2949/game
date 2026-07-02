use super::defaults::*;
use super::*;

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

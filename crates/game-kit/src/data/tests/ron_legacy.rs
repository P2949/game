use super::super::validate::{validate_file, validate_file_with_base};
use super::super::{
    BeginnerGameFile, load_beginner_game_text, migrate_legacy_ron_source_to_toml,
    parse_beginner_game_source,
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
fn migrate_ron_to_toml_converts_checked_in_legacy_examples() {
    for relative in [
        "../../templates/data-driven-demo/assets/game.ron",
        "../../examples/data-driven-events-demo/assets/game.ron",
        "../../examples/data-driven-waves-demo/assets/game.ron",
        "../../examples/data-driven-projectiles-demo/assets/game.ron",
        "../../examples/data-driven-full-demo/assets/game.ron",
        "../../examples/data-driven-tiled-demo/assets/game.ron",
    ] {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
        let source = std::fs::read_to_string(&path).unwrap();
        let migration = migrate_legacy_ron_source_to_toml(&source, relative).unwrap();

        assert!(migration.toml.contains("version = 2"));
        assert!(migration.toml.contains("[controls]"));
        assert!(!migration.toml.contains("Player(("));
        assert!(!migration.toml.contains("Some("));
        assert!(
            migration
                .notes
                .iter()
                .any(|note| note.contains("primary game.toml"))
        );
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

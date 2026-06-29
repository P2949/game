use game_core::commands::AssetReloadStatus;
use game_kit::advanced::prelude::*;
use game_kit::testing::GameTestHarness;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

const PLAYER: &str = "flow/player";

struct MapFlowPlugin;

impl GamePlugin for MapFlowPlugin {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        let texture = game.assets(|assets| assets.texture("flow/player", "textures/test.png"))?;
        let controls = game.input(|input| {
            Ok((
                input.action("start")?.space(),
                input.action("game_over")?.enter(),
                input.action("restart")?.key(Key::R),
            ))
        })?;

        game.prefab(PLAYER, |prefab| {
            prefab
                .spawn(move |at| {
                    (
                        Transform::at(at),
                        Sprite::new(texture, vec2s(16.0)).layer(10),
                    )
                })?
                .require::<Transform>()
                .require::<Sprite>();
            Ok(())
        })?;

        register_map(game, "menu", [".."], false, true);
        register_map(game, "game", ["...", "..."], true, false);
        register_map(game, "game_over", ["...."], false, false);

        game.start_scene("menu").scene("game").scene("game_over");
        game.on_start(|game| game.spawn_start_map());
        game.on_action(controls.0, |game| {
            game.change_map_or_log("game");
            game.change_scene_or_log("game");
        });
        game.on_action(controls.1, |game| {
            game.change_map_or_log("game_over");
            game.change_scene_or_log("game_over");
        });
        game.on_action(controls.2, |game| {
            game.change_map_or_log("game");
            game.change_scene_or_log("game");
        });
        game.fixed_systems_are_pause_guarded();

        Ok(())
    }
}

struct SimpleSceneFlowPlugin;

impl GamePlugin for SimpleSceneFlowPlugin {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        let controls = game.input(|input| input.top_down_controls())?;

        game.player_prefab(PLAYER)
            .sprite(TextureHandle(0))
            .moves_with(controls.movement, 120.0)
            .health(1)
            .build()?;

        register_map(game, "menu", [".."], false, true);
        register_map(game, "game", ["...", "..."], true, false);
        register_map(game, "game_over", ["...."], false, false);

        game.use_simple_scene_flow()
            .menu("menu")
            .menu_text("Start the adventure")
            .menu_button("Play", "game")
            .game("game")
            .game_over("game_over")
            .game_over_text("Defeated - press R")
            .game_over_button("Try again")
            .start_on(controls.attack)
            .restart_on(controls.reset)
            .build();

        Ok(())
    }
}

struct SymbolicMapPlugin;

impl GamePlugin for SymbolicMapPlugin {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        register_marker_prefab(game, "player")?;
        register_marker_prefab(game, "slime")?;

        game.map("legend")
            .tile_size(16.0)
            .tiles(["#####", "#P.E#", "#####"])
            .simple_theme(TextureHandle(0), TextureHandle(0))
            .legend('P', "player")
            .legend('E', "slime")
            .start();

        game.on_start(|game| game.spawn_start_map());

        Ok(())
    }
}

struct TextMapPlugin;

impl GamePlugin for TextMapPlugin {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        let controls = game.input(|input| input.top_down_controls())?;
        game.player_prefab("player")
            .sprite(TextureHandle(1))
            .moves_with(controls.movement, 120.0)
            .build()?;
        game.enemy_prefab("slime")
            .sprite(TextureHandle(2))
            .build()?;
        game.pickup_prefab("coin")
            .sprite(TextureHandle(3))
            .score(1)
            .build()?;
        game.map_from_text_auto("beginner_text_map")
            .simple_theme(TextureHandle(0), TextureHandle(0))
            .legend('P', "player")
            .legend('E', "slime")
            .legend('C', "coin")
            .start();
        game.on_start(|game| game.spawn_start_map());
        Ok(())
    }
}

struct ReloadableTextMapPlugin {
    path: String,
}

struct LdtkMapPlugin {
    map_entities: bool,
}

struct TiledMapPlugin {
    map_objects: bool,
}

impl GamePlugin for LdtkMapPlugin {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        let controls = game.input(|input| input.top_down_controls())?;
        game.player_prefab("player")
            .sprite(TextureHandle(1))
            .moves_with(controls.movement, 120.0)
            .build()?;
        game.enemy_prefab("slime")
            .sprite(TextureHandle(2))
            .build()?;

        let mut map = game
            .map_from_ldtk("ldtk", "maps/ldtk_demo.ldtk")
            .level("Level_1")
            .simple_theme(TextureHandle(0), TextureHandle(0))
            .entity("PlayerStart", "player");
        if self.map_entities {
            map = map.entity("Slime", "slime");
        }
        map.start();

        game.use_top_down_game().controls(controls).build();
        Ok(())
    }
}

impl GamePlugin for TiledMapPlugin {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        let controls = game.input(|input| input.top_down_controls())?;
        game.player_prefab("player")
            .sprite(TextureHandle(1))
            .moves_with(controls.movement, 120.0)
            .build()?;
        game.enemy_prefab("slime")
            .sprite(TextureHandle(2))
            .build()?;

        let mut map = game
            .map_from_tiled("tiled", "maps/tiled_demo.tmx")
            .simple_theme(TextureHandle(0), TextureHandle(0))
            .object("Player", "player");
        if self.map_objects {
            map = map.object("Slime", "slime");
        }
        map.start();

        game.use_top_down_game().controls(controls).build();
        Ok(())
    }
}

impl GamePlugin for ReloadableTextMapPlugin {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        let controls = game.input(|input| input.top_down_controls())?;
        game.player_prefab("player")
            .sprite(TextureHandle(1))
            .moves_with(controls.movement, 120.0)
            .build()?;
        game.enemy_prefab("slime")
            .sprite(TextureHandle(2))
            .build()?;
        game.map_from_text("reloadable", self.path.clone())
            .simple_theme(TextureHandle(0), TextureHandle(0))
            .legend('P', "player")
            .legend('E', "slime")
            .start();
        game.use_top_down_game().controls(controls).build();
        Ok(())
    }
}

#[test]
fn beginner_map_flow_changes_maps_and_respawns_objects() {
    let mut game = GameTestHarness::from_plugin(MapFlowPlugin).unwrap();
    assert_eq!(game.current_map_name(), Some("menu".to_owned()));
    assert_eq!(
        game.world()
            .get_resource::<SceneState>()
            .map(|scene| scene.current().to_owned()),
        Some("menu".to_owned())
    );
    assert_eq!(game.map().tilemap.width(), 2);
    assert_eq!(game.world().ids_with::<Transform>().len(), 0);

    game = game.press_action("start");
    game.fixed_step(1.0 / 120.0);
    game.clear_input();
    assert_eq!(game.current_map_name(), Some("game".to_owned()));
    assert_eq!(
        game.world()
            .get_resource::<SceneState>()
            .map(|scene| scene.current().to_owned()),
        Some("game".to_owned())
    );
    assert_eq!(game.map().tilemap.width(), 3);
    assert_eq!(game.world().ids_with::<Transform>().len(), 1);

    game = game.press_action("game_over");
    game.fixed_step(1.0 / 120.0);
    game.clear_input();
    assert_eq!(game.current_map_name(), Some("game_over".to_owned()));
    assert_eq!(
        game.world()
            .get_resource::<SceneState>()
            .map(|scene| scene.current().to_owned()),
        Some("game_over".to_owned())
    );
    assert_eq!(game.map().tilemap.width(), 4);
    assert_eq!(game.world().ids_with::<Transform>().len(), 0);

    game = game.press_action("restart");
    game.fixed_step(1.0 / 120.0);
    assert_eq!(game.current_map_name(), Some("game".to_owned()));
    assert_eq!(
        game.world()
            .get_resource::<SceneState>()
            .map(|scene| scene.current().to_owned()),
        Some("game".to_owned())
    );
    assert_eq!(game.map().tilemap.width(), 3);
    assert_eq!(game.world().ids_with::<Transform>().len(), 1);
}

#[test]
fn simple_scene_flow_drives_menu_level_game_over_restart() {
    let mut game = GameTestHarness::from_plugin(SimpleSceneFlowPlugin).unwrap();

    assert_eq!(game.current_map_name(), Some("menu".to_owned()));
    assert_eq!(
        game.world()
            .get_resource::<SceneState>()
            .map(|scene| scene.current().to_owned()),
        Some("menu".to_owned())
    );
    game.frame(0.0);
    game.assert_ui_contains("Start the adventure");

    game = game.press_action("attack");
    game.frame(1.0 / 120.0);
    game.clear_input();
    assert_eq!(game.current_map_name(), Some("game".to_owned()));
    assert_eq!(
        game.world()
            .get_resource::<SceneState>()
            .map(|scene| scene.current().to_owned()),
        Some("game".to_owned())
    );
    assert_eq!(game.map().tilemap.width(), 3);
    assert_eq!(game.player_count(), 1);

    let player = game.player();
    game.set_entity_health(player, 0);
    game.frame(1.0 / 120.0);
    assert_eq!(game.current_map_name(), Some("game_over".to_owned()));
    assert_eq!(
        game.world()
            .get_resource::<SceneState>()
            .map(|scene| scene.current().to_owned()),
        Some("game_over".to_owned())
    );
    assert_eq!(game.map().tilemap.width(), 4);
    game.assert_ui_contains("Defeated - press R");

    game = game.press_action("reset");
    game.frame(1.0 / 120.0);
    assert_eq!(game.current_map_name(), Some("game".to_owned()));
    assert_eq!(
        game.world()
            .get_resource::<SceneState>()
            .map(|scene| scene.current().to_owned()),
        Some("game".to_owned())
    );
    assert_eq!(game.player_count(), 1);
}

#[test]
fn simple_scene_flow_buttons_start_and_restart_the_game() {
    let mut game = GameTestHarness::from_plugin(SimpleSceneFlowPlugin)
        .unwrap()
        .click_mouse_left_at(vec2(400.0, 356.0), vec2(800.0, 600.0));

    game.frame(1.0 / 120.0);
    assert_eq!(game.current_map_name(), Some("game".to_owned()));
    assert_eq!(
        game.world()
            .get_resource::<SceneState>()
            .map(|scene| scene.current().to_owned()),
        Some("game".to_owned())
    );

    let player = game.player();
    game.set_entity_health(player, 0);
    game.clear_input();
    game.frame(1.0 / 120.0);
    assert_eq!(game.current_map_name(), Some("game_over".to_owned()));

    game = game.click_mouse_left_at(vec2(400.0, 356.0), vec2(800.0, 600.0));
    game.frame(1.0 / 120.0);
    assert_eq!(game.current_map_name(), Some("game".to_owned()));
    assert_eq!(game.player_count(), 1);
}

#[test]
fn symbolic_map_legends_spawn_prefabs_from_tile_rows() {
    let game = GameTestHarness::from_plugin(SymbolicMapPlugin).unwrap();

    assert_eq!(game.current_map_name(), Some("legend".to_owned()));
    assert_eq!(game.map().tilemap.width(), 5);
    assert_eq!(game.world().ids_with::<Transform>().len(), 2);
}

#[test]
fn beginner_text_map_loads_symbolic_spawns_from_assets_folder() {
    let game = GameTestHarness::from_plugin(TextMapPlugin).unwrap();

    assert_eq!(
        game.current_map_name(),
        Some("beginner_text_map".to_owned())
    );
    assert_eq!(game.player_count(), 1);
    assert_eq!(game.enemy_count(), 1);
}

#[test]
fn f5_reloads_the_current_text_map_without_restarting_rust_content() {
    let path = std::env::temp_dir().join(format!(
        "game-kit-reload-{}-{}.txt",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::write(&path, "###\n#PE\n###\n").unwrap();

    let mut game = GameTestHarness::from_plugin(ReloadableTextMapPlugin {
        path: path.display().to_string(),
    })
    .unwrap();
    assert_eq!(game.map().tilemap.width(), 3);
    assert_eq!(game.player_count(), 1);
    assert_eq!(game.enemy_count(), 1);

    fs::write(&path, "#####\n#P..E\n#####\n").unwrap();
    game = game.press_action("reload");
    game.fixed_step(1.0 / 120.0);

    assert_eq!(game.map().tilemap.width(), 5);
    assert_eq!(game.player_count(), 1);
    assert_eq!(game.enemy_count(), 1);
    assert_eq!(
        game.world()
            .get_resource::<AssetReloadStatus>()
            .unwrap()
            .message,
        "not applied in the headless game-kit harness"
    );
}

#[test]
fn ldtk_level_spawns_entities_from_named_prefab_mappings() {
    let game = GameTestHarness::from_plugin(LdtkMapPlugin { map_entities: true }).unwrap();
    assert_eq!(game.map().tilemap.width(), 8);
    assert_eq!(game.player_count(), 1);
    assert_eq!(game.enemy_count(), 1);
}

#[test]
fn ldtk_level_reports_missing_entity_mappings() {
    let error = match GameTestHarness::from_plugin(LdtkMapPlugin {
        map_entities: false,
    }) {
        Ok(_) => panic!("LDtk maps must reject unmapped entities"),
        Err(error) => error.to_string(),
    };
    assert!(error.contains("entity 'Slime' with no prefab mapping"));
    assert!(error.contains(".entity(\"Slime\", \"some_prefab\")"));
}

#[test]
fn tiled_map_spawns_mapped_objects_from_csv_collision_and_object_layers() {
    let game = GameTestHarness::from_plugin(TiledMapPlugin { map_objects: true }).unwrap();

    assert_eq!(game.map().tilemap.width(), 5);
    assert_eq!(game.map().tilemap.height(), 3);
    assert_eq!(game.count::<Player>(), 1);
    assert_eq!(game.count::<Enemy>(), 1);
}

#[test]
fn tiled_map_reports_missing_object_mappings() {
    let error = GameTestHarness::from_plugin(TiledMapPlugin { map_objects: false })
        .err()
        .expect("missing object mapping must fail")
        .to_string();

    assert!(error.contains("Tiled map 'tiled' has object 'Slime' with no prefab mapping"));
    assert!(error.contains(".object(\"Slime\", \"some_prefab\")"));
}

fn register_marker_prefab(game: &mut GameApp<'_>, name: &str) -> Result<()> {
    game.prefab(name, |prefab| {
        prefab
            .spawn(|at| (Transform::at(at),))?
            .require::<Transform>();
        Ok(())
    })
}

fn register_map<const N: usize>(
    game: &mut GameApp<'_>,
    name: &str,
    rows: [&str; N],
    spawn_player: bool,
    start: bool,
) {
    let mut map = game.map(name).tile_size(16.0).tiles(rows).theme(TileTheme {
        floor: Sprite::new(TextureHandle(0), vec2s(16.0)),
        wall: Sprite::new(TextureHandle(0), vec2s(16.0)).layer(1),
    });

    if spawn_player {
        map = map.spawn("player_start", PLAYER, cell(1, 1));
    }

    if start {
        map.start();
    } else {
        map.finish();
    }
}

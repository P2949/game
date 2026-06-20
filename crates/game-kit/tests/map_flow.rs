use game_kit::prelude::*;
use game_kit::testing::GameTestHarness;

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
        game.on_start(|game: &mut StartupGameCtx<'_, '_>| game.spawn_start_map());
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
            .game("game")
            .game_over("game_over")
            .game_over_text("Defeated - press R")
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

        game.on_start(|game: &mut StartupGameCtx<'_, '_>| game.spawn_start_map());

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
        game.map_from_text("text_level", "maps/beginner_text_map.txt")
            .simple_theme(TextureHandle(0), TextureHandle(0))
            .legend('P', "player")
            .legend('E', "slime")
            .start();
        game.on_start(|game| game.spawn_start_map());
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
fn symbolic_map_legends_spawn_prefabs_from_tile_rows() {
    let game = GameTestHarness::from_plugin(SymbolicMapPlugin).unwrap();

    assert_eq!(game.current_map_name(), Some("legend".to_owned()));
    assert_eq!(game.map().tilemap.width(), 5);
    assert_eq!(game.world().ids_with::<Transform>().len(), 2);
}

#[test]
fn beginner_text_map_loads_symbolic_spawns_from_assets_folder() {
    let game = GameTestHarness::from_plugin(TextMapPlugin).unwrap();

    assert_eq!(game.current_map_name(), Some("text_level".to_owned()));
    assert_eq!(game.player_count(), 1);
    assert_eq!(game.enemy_count(), 1);
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

use std::fs;

use game_kit::advanced::prelude::GameCtx;
use game_kit::beginner::prelude::*;
use game_kit::testing::GameTestHarness;

struct LiveTuningPlugin {
    path: String,
}

struct F5TuningPlugin {
    tuning_path: String,
    map_path: String,
}

impl GamePlugin for LiveTuningPlugin {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        let controls = game.input(|input| input.top_down_controls())?;
        let tuning = game.tuning_from_file(&self.path)?;

        game.player_prefab("player")
            .sprite(TextureHandle(1))
            .moves_with(controls.movement, tuning.float("player.speed"))
            .health(tuning.int("player.health"))
            .build()?;
        game.map("tuned")
            .tiles(["###", "#P#", "###"])
            .simple_theme(TextureHandle(0), TextureHandle(0))
            .legend('P', "player")
            .start();
        game.use_top_down_game().controls(controls).build();
        game.fixed(move |game: &mut GameCtx<'_, '_>, _dt| {
            if game.pressed(controls.attack) {
                game.reload_tuning_or_log();
                game.reset_to_start_map_or_log();
            }
        });
        Ok(())
    }
}

impl GamePlugin for F5TuningPlugin {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        let controls = game.input(|input| input.top_down_controls())?;
        let tuning = game.tuning_from_file(&self.tuning_path)?;

        game.player_prefab("player")
            .sprite(TextureHandle(1))
            .moves_with(controls.movement, tuning.float("player.speed"))
            .health(tuning.int("player.health"))
            .build()?;
        game.map_from_text("tuned", self.map_path.clone())
            .simple_theme(TextureHandle(0), TextureHandle(0))
            .legend('P', "player")
            .start();
        game.use_top_down_game().controls(controls).build();
        Ok(())
    }
}

#[test]
fn reloaded_tuning_is_used_by_entities_spawned_after_a_reset() {
    let path = std::env::temp_dir().join(format!(
        "game-kit-live-tuning-{}-{}.ron",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::write(
        &path,
        "{ \"player.health\": 40.0, \"player.speed\": 100.0 }",
    )
    .unwrap();

    let mut game = GameTestHarness::from_plugin(LiveTuningPlugin {
        path: path.to_string_lossy().into_owned(),
    })
    .unwrap();
    game.assert_player_health(40);

    fs::write(
        &path,
        "{ \"player.health\": 75.0, \"player.speed\": 180.0 }",
    )
    .unwrap();
    game.tap_action("attack");

    game.assert_player_health(75);
    fs::remove_file(path).unwrap();
}

#[test]
fn f5_reloads_configured_tuning_before_respawning_a_text_map() {
    let suffix = format!(
        "{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    let tuning_path = std::env::temp_dir().join(format!("game-kit-f5-tuning-{suffix}.ron"));
    let map_path = std::env::temp_dir().join(format!("game-kit-f5-map-{suffix}.txt"));
    fs::write(
        &tuning_path,
        "{ \"player.health\": 40.0, \"player.speed\": 100.0 }",
    )
    .unwrap();
    fs::write(&map_path, "###\n#P#\n###\n").unwrap();

    let mut game = GameTestHarness::from_plugin(F5TuningPlugin {
        tuning_path: tuning_path.to_string_lossy().into_owned(),
        map_path: map_path.to_string_lossy().into_owned(),
    })
    .unwrap();
    game.assert_player_health(40);

    fs::write(
        &tuning_path,
        "{ \"player.health\": 75.0, \"player.speed\": 180.0 }",
    )
    .unwrap();
    game = game.press_action("reload");
    game.fixed_step(1.0 / 120.0);

    game.assert_player_health(75);
    fs::remove_file(tuning_path).unwrap();
    fs::remove_file(map_path).unwrap();
}

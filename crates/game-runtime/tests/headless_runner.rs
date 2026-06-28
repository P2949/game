use std::time::Duration;

use game_backend_headless::{HeadlessAudio, HeadlessPlatform, HeadlessRenderer};
use game_kit::{plugin_fn, testing::GameTestHarness};
use game_runtime::{Runner, RuntimeConfig};

#[test]
fn headless_backends_drive_the_complete_content_runtime_loop() {
    let content = plugin_fn(|game| {
        game.asset_bag()
            .texture("test", "textures/test.png")?
            .generated_sound("blip")?
            .build();
        game.map("level")
            .tiles(["..."])
            .simple_theme("test", "test")
            .start();
        game.every_tick(|game, _dt| {
            game.audio().play_sound("blip");
        });
        Ok(())
    });
    let mut runner = GameTestHarness::build_runtime(content, |builder| {
        Runner::new(
            RuntimeConfig::default(),
            builder,
            ".",
            HeadlessPlatform::default(),
            HeadlessRenderer::default(),
            Some(HeadlessAudio::default()),
        )
    })
    .unwrap();

    runner
        .step_frame(Duration::from_secs_f32(1.0 / 60.0))
        .unwrap();

    let frame = runner
        .renderer()
        .frames()
        .last()
        .expect("the runtime should submit a frame");
    assert!(
        !frame.world_sprites.is_empty(),
        "tile extraction should run before a headless frame is submitted"
    );
    assert!(
        !runner.audio().unwrap().commands().is_empty(),
        "audio commands should flow through the runtime backend trait"
    );
}

#[test]
fn headless_backends_drive_data_driven_beginner_file() {
    let content = plugin_fn(|game| {
        game.load_beginner_file("game.ron")?;
        Ok(())
    });
    let mut runner = GameTestHarness::build_runtime(content, |builder| {
        Runner::new(
            RuntimeConfig::default(),
            builder,
            ".",
            HeadlessPlatform::default(),
            HeadlessRenderer::default(),
            Some(HeadlessAudio::default()),
        )
    })
    .unwrap();

    runner
        .step_frame(Duration::from_secs_f32(1.0 / 60.0))
        .unwrap();

    let frame = runner
        .renderer()
        .frames()
        .last()
        .expect("the runtime should submit a data-driven frame");
    assert!(
        !frame.world_sprites.is_empty(),
        "data-driven content should produce world sprites through the runtime"
    );
}

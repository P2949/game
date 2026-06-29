use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use game_backend_headless::{HeadlessAudio, HeadlessPlatform, HeadlessRenderer};
use game_core::app::RenderFrame;
use game_core::backend::{
    PlatformBackend, PlatformEvents, RenderBackend, RenderOutcome, TextureHandle,
};
use game_core::input::InputState;
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

#[test]
fn renderer_drops_before_platform_window_owner() {
    let drops = Rc::new(RefCell::new(Vec::new()));
    let content = plugin_fn(|game| {
        game.asset_bag()
            .texture("test", "textures/test.png")?
            .build();
        game.map("level")
            .tiles(["."])
            .simple_theme("test", "test")
            .start();
        Ok(())
    });

    {
        let runner = GameTestHarness::build_runtime(content, |builder| {
            Runner::new(
                RuntimeConfig::default(),
                builder,
                ".",
                DropOrderPlatform::new(Rc::clone(&drops)),
                DropOrderRenderer::new(Rc::clone(&drops)),
                None::<HeadlessAudio>,
            )
        })
        .unwrap();
        drop(runner);
    }

    assert_eq!(
        drops.borrow().as_slice(),
        ["renderer", "platform"],
        "renderer must drop before platform/window teardown"
    );
}

struct DropOrderPlatform {
    drops: Rc<RefCell<Vec<&'static str>>>,
    input: InputState,
}

impl DropOrderPlatform {
    fn new(drops: Rc<RefCell<Vec<&'static str>>>) -> Self {
        let mut input = InputState::default();
        input.set_viewport_size(glam::vec2(1280.0, 720.0));
        Self { drops, input }
    }
}

impl PlatformBackend for DropOrderPlatform {
    fn pump_events(&mut self) -> PlatformEvents {
        PlatformEvents { should_quit: false }
    }

    fn input(&self) -> &InputState {
        &self.input
    }

    fn drawable_size(&self) -> glam::UVec2 {
        glam::uvec2(1280, 720)
    }

    fn take_stable_resize_request(&mut self) -> bool {
        false
    }

    fn should_quit(&self) -> bool {
        false
    }

    fn request_quit(&mut self) {}
}

impl Drop for DropOrderPlatform {
    fn drop(&mut self) {
        self.drops.borrow_mut().push("platform");
    }
}

struct DropOrderRenderer {
    drops: Rc<RefCell<Vec<&'static str>>>,
}

impl DropOrderRenderer {
    fn new(drops: Rc<RefCell<Vec<&'static str>>>) -> Self {
        Self { drops }
    }
}

impl RenderBackend for DropOrderRenderer {
    fn reload_textures(&mut self, textures: &[(TextureHandle, String)]) -> anyhow::Result<usize> {
        Ok(textures.len())
    }

    fn request_resize(&mut self) {}

    fn render(
        &mut self,
        _drawable_size: glam::UVec2,
        _frame: RenderFrame,
    ) -> anyhow::Result<RenderOutcome> {
        Ok(RenderOutcome::Presented)
    }
}

impl Drop for DropOrderRenderer {
    fn drop(&mut self) {
        self.drops.borrow_mut().push("renderer");
    }
}

//! A content test harness (Phase 18).
//!
//! [`GameTestHarness`] builds a [`GamePlugin`] the same way the runtime does,
//! runs its startup, and steps frames headlessly — so content integration tests
//! exercise the real plugin/schedule wiring without hand-constructing `Ctx`,
//! `RenderFrame`, `World`, and friends.

use anyhow::Result;
use game_core::app::{Ctx, MapData, RenderFrame, StartCtx};
use game_core::audio::{Audio, AudioCommands};
use game_core::backend::AudioCommand;
use game_core::builder::{GameBuilder, RuntimeContent};
use game_core::camera::Camera2D;
use game_core::commands::{Command, CommandQueue};
use game_core::gfx::Gfx;
use game_core::input::{ActionId, Axis2dId, Input};
use game_core::plugin::GamePlugin as CoreGamePlugin;
use game_core::schedule::Schedule;
use game_core::world::World;
use glam::Vec2;

use crate::app::{GamePlugin, plugin};

/// Drives a content plugin headlessly for tests: build → startup → step frames,
/// inspecting UI text and the world.
pub struct GameTestHarness {
    schedule: Schedule,
    world: World,
    map: MapData,
    input: Input,
    camera: Camera2D,
    ui_text: Vec<String>,
    audio_commands: Vec<AudioCommand>,
}

impl GameTestHarness {
    /// Builds `plugin` through the full facade pipeline (including map/prefab
    /// validation), then runs its startup systems.
    pub fn from_plugin(content: impl GamePlugin) -> Result<Self> {
        let wrapped = plugin(content);
        let mut builder = GameBuilder::new();
        CoreGamePlugin::build(&wrapped, &mut builder)?;

        let RuntimeContent {
            maps,
            start_map,
            mut schedule,
            ..
        } = builder.into_parts()?;
        let map = maps
            .get(start_map)
            .ok_or_else(|| anyhow::anyhow!("start map not registered"))?
            .data
            .clone();

        let mut world = World::new();
        schedule.run_startup(&mut StartCtx::new(&mut world))?;

        Ok(Self {
            schedule,
            world,
            map,
            input: Input::default(),
            camera: Camera2D::new(Vec2::ZERO, 1.0),
            ui_text: Vec::new(),
            audio_commands: Vec::new(),
        })
    }

    /// Marks `action` pressed (and held) for subsequent frames.
    pub fn press(mut self, action: ActionId) -> Self {
        self.input = self.input.with_pressed(action);
        self
    }

    /// Sets a 2D axis value for subsequent frames.
    pub fn axis(mut self, axis: Axis2dId, value: Vec2) -> Self {
        self.input = self.input.with_axis2d(axis, value);
        self
    }

    /// Resets all input back to neutral.
    pub fn clear_input(&mut self) {
        self.input = Input::default();
    }

    /// Runs one frame (fixed + update + render-extract + ui) and returns the UI
    /// text produced.
    pub fn frame(&mut self, dt: f32) -> Vec<String> {
        let mut frame = RenderFrame::new(self.camera);
        let mut audio_commands = AudioCommands::default();
        {
            let mut ctx = Ctx {
                world: &mut self.world,
                map: &self.map.tilemap,
                nav: &self.map.nav,
                input: &self.input,
                camera: &mut self.camera,
                gfx: Gfx::new(&mut frame),
                audio: Audio::new(&mut audio_commands),
            };
            self.schedule.run_fixed(&mut ctx, dt);
            self.schedule.run_update(&mut ctx, dt);
            self.schedule.run_render_extract(&mut ctx, dt);
            self.schedule.run_ui(&mut ctx, dt);
        }
        self.process_core_commands(&mut audio_commands);
        self.audio_commands.extend(audio_commands.drain());
        self.ui_text = frame.ui_text.into_iter().map(|text| text.text).collect();
        self.ui_text.clone()
    }

    /// Runs a single fixed step (no update/ui), for stepping the simulation.
    pub fn fixed_step(&mut self, dt: f32) {
        let mut frame = RenderFrame::new(self.camera);
        let mut audio_commands = AudioCommands::default();
        let mut ctx = Ctx {
            world: &mut self.world,
            map: &self.map.tilemap,
            nav: &self.map.nav,
            input: &self.input,
            camera: &mut self.camera,
            gfx: Gfx::new(&mut frame),
            audio: Audio::new(&mut audio_commands),
        };
        self.schedule.run_fixed(&mut ctx, dt);
        self.process_core_commands(&mut audio_commands);
        self.audio_commands.extend(audio_commands.drain());
    }

    /// Read-only access to the simulated world.
    pub fn world(&self) -> &World {
        &self.world
    }

    /// Mutable access to the simulated world (e.g. to reposition entities).
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    /// The start map's collision tilemap dimensions and data.
    pub fn map(&self) -> &MapData {
        &self.map
    }

    /// UI text produced by the most recent [`Self::frame`] call.
    pub fn ui_text(&self) -> &[String] {
        &self.ui_text
    }

    /// Audio commands produced by processed content commands so far.
    pub fn audio_commands(&self) -> &[AudioCommand] {
        &self.audio_commands
    }

    fn process_core_commands(&mut self, audio_commands: &mut AudioCommands) {
        let commands = self
            .world
            .get_resource_mut::<CommandQueue>()
            .map(|queue| queue.drain().collect::<Vec<_>>())
            .unwrap_or_default();

        for command in commands {
            match command {
                Command::Despawn(entity) => self.world.despawn(entity),
                Command::PlaySound(sound) => audio_commands.push(AudioCommand::Play {
                    sound,
                    volume: 0.8,
                    looping: false,
                }),
            }
        }
    }
}

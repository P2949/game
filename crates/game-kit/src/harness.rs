//! A content test harness (Phase 18).
//!
//! [`GameTestHarness`] builds a [`GamePlugin`] the same way the runtime does,
//! runs its startup, and steps frames headlessly — so content integration tests
//! exercise the real plugin/schedule wiring without hand-constructing `Ctx`,
//! `RenderFrame`, `World`, and friends.

use std::rc::Rc;

use anyhow::Result;
use game_combat::Health;
use game_core::app::{Ctx, MapData, RenderFrame, StartCtx};
use game_core::audio::{Audio, AudioCommands};
use game_core::backend::{AudioCommand, SoundHandle};
use game_core::builder::{GameBuilder, MapId, MapRegistry, PrefabRegistry, RuntimeContent};
use game_core::camera::Camera2D;
use game_core::commands::{Command, CommandQueue};
use game_core::gfx::Gfx;
use game_core::input::{ActionId, Axis2dId, Input, InputRegistry, MouseButton};
use game_core::plugin::GamePlugin as CoreGamePlugin;
use game_core::schedule::Schedule;
use game_core::world::{Component, EntityId, Transform, World};
use glam::Vec2;

use crate::app::{GamePlugin, plugin};
use crate::beginner::actors::{Enemy, Pickup, Player};
use crate::beginner::collections::Score;
use crate::beginner::scene::SceneState;
use crate::beginner::testing::TestEntity;
use crate::map::{ContentRuntime, reset_to_start_map_world};

/// Drives a content plugin headlessly for tests: build → startup → step frames,
/// inspecting UI text and the world.
pub struct GameTestHarness {
    schedule: Schedule,
    world: World,
    prefabs: Rc<PrefabRegistry>,
    maps: MapRegistry,
    start_map: MapId,
    active_map: MapId,
    map: MapData,
    input_registry: InputRegistry,
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
            input,
            prefabs,
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
            prefabs,
            maps,
            start_map,
            active_map: start_map,
            map,
            input_registry: input,
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

    /// Marks a named action pressed (and held) for subsequent frames.
    pub fn press_action(mut self, name: &str) -> Self {
        let action = self
            .input_registry
            .action_id(name)
            .unwrap_or_else(|| panic!("unknown action '{name}'"));
        self.input = self.input.with_pressed(action);
        self
    }

    /// Sets a 2D axis value for subsequent frames.
    pub fn axis(mut self, axis: Axis2dId, value: Vec2) -> Self {
        self.input = self.input.with_axis2d(axis, value);
        self
    }

    /// Sets a named 2D axis value for subsequent frames.
    pub fn set_axis(mut self, name: &str, value: Vec2) -> Self {
        let axis = self
            .input_registry
            .axis2d_id(name)
            .unwrap_or_else(|| panic!("unknown axis '{name}'"));
        self.input = self.input.with_axis2d(axis, value);
        self
    }

    /// Positions the mouse and sends one left-click edge through the same
    /// screen-space input used by beginner UI buttons.
    pub fn click_mouse_left_at(mut self, position: Vec2, viewport_size: Vec2) -> Self {
        self.input = self
            .input
            .clone()
            .with_mouse_position(position, viewport_size)
            .with_mouse_pressed(MouseButton::Left);
        self
    }

    /// Resets all input back to neutral.
    pub fn clear_input(&mut self) {
        self.input = Input::default();
    }

    pub fn release_input(&mut self) {
        self.clear_input();
    }

    pub fn step(&mut self) {
        self.step_seconds(1.0 / 120.0);
    }

    pub fn step_seconds(&mut self, dt: f32) {
        self.fixed_step(dt);
    }

    pub fn tap_action(&mut self, name: &str) {
        let action = self.action_id(name);
        self.input = Input::default().with_pressed(action);
        self.step();
        self.release_input();
    }

    pub fn hold_action(&mut self, name: &str) {
        let action = self.action_id(name);
        self.input = self.input.clone().with_down(action);
    }

    pub fn player(&self) -> TestEntity {
        let id = self
            .world
            .ids_with::<Player>()
            .into_iter()
            .next()
            .expect("expected a player entity");
        TestEntity::from_world(id, &self.world)
    }

    pub fn enemy(&self, index: usize) -> TestEntity {
        let id = self
            .world
            .ids_with::<Enemy>()
            .get(index)
            .copied()
            .unwrap_or_else(|| panic!("expected enemy at index {index}"));
        TestEntity::from_world(id, &self.world)
    }

    pub fn enemies(&self) -> Vec<TestEntity> {
        self.world
            .ids_with::<Enemy>()
            .into_iter()
            .map(|id| TestEntity::from_world(id, &self.world))
            .collect()
    }

    pub fn player_count(&self) -> usize {
        self.world.ids_with::<Player>().len()
    }

    pub fn enemy_count(&self) -> usize {
        self.world.ids_with::<Enemy>().len()
    }

    pub fn assert_player_health(&self, expected: i32) {
        assert_eq!(self.player().health(), expected);
    }

    pub fn assert_enemy_dead(&self, index: usize) {
        assert!(
            self.enemy(index).is_dead(),
            "expected enemy {index} to be dead"
        );
    }

    pub fn assert_enemy_count(&self, expected: usize) {
        assert_eq!(self.enemy_count(), expected);
    }

    pub fn assert_score(&self, expected: i32) {
        let score = self
            .world
            .get_resource::<Score>()
            .map(|score| score.value)
            .unwrap_or_default();
        assert_eq!(score, expected);
    }

    pub fn entity_count(&self) -> usize {
        self.world.ids().count()
    }

    pub fn count<T: Component>(&self) -> usize {
        self.world.ids_with::<T>().len()
    }

    pub fn has_resource<T: 'static>(&self) -> bool {
        self.world.get_resource::<T>().is_some()
    }

    pub fn move_enemy_next_to_player(&mut self, index: usize) {
        let player_pos = self.player().position();
        let enemy = self.enemy(index);
        self.move_entity_to(enemy, player_pos + glam::vec2(10.0, 0.0));
    }

    pub fn move_player_to_pickup(&mut self, index: usize) {
        let pickup = self
            .world
            .ids_with::<Pickup>()
            .get(index)
            .copied()
            .unwrap_or_else(|| panic!("expected pickup at index {index}"));
        let pickup_pos = self
            .world
            .get::<Transform>(pickup)
            .unwrap_or_else(|| panic!("pickup {:?} has no Transform component", pickup))
            .pos;
        let player = self.player();
        self.move_entity_to(player, pickup_pos);
    }

    pub fn collect_first_pickup(&mut self) {
        self.move_player_to_pickup(0);
        self.step();
    }

    pub fn move_entity_to(&mut self, entity: TestEntity, pos: Vec2) {
        let transform = self
            .world
            .get_mut::<Transform>(entity.id())
            .unwrap_or_else(|| panic!("entity {:?} has no Transform component", entity.id()));
        transform.pos = pos;
    }

    pub fn set_entity_health(&mut self, entity: TestEntity, health: i32) {
        let health_component = self
            .world
            .get_mut::<Health>(entity.id())
            .unwrap_or_else(|| panic!("entity {:?} has no Health component", entity.id()));
        health_component.current = health.clamp(0, health_component.max);
    }

    pub fn set_enemy_health(&mut self, index: usize, health: i32) {
        let enemy = self.enemy(index);
        self.set_entity_health(enemy, health);
    }

    pub fn sound_count(&self) -> usize {
        self.audio_commands.len()
    }

    pub fn assert_sound_played(&self) {
        assert!(
            self.sound_count() > 0,
            "expected at least one sound command to be played"
        );
    }

    pub fn assert_ui_contains(&self, text: &str) {
        assert!(
            self.ui_text.iter().any(|line| line.contains(text)),
            "expected UI text to contain '{text}', got {:?}",
            self.ui_text
        );
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

    /// Resets the simulated world through the same content-runtime path as
    /// [`GameCtx::reset_to_start_map`](crate::GameCtx::reset_to_start_map).
    pub fn reset_to_start_map(&mut self) -> Result<()> {
        reset_to_start_map_world(&mut self.world)?;
        self.switch_active_map(self.start_map);
        Ok(())
    }

    /// Queues a despawn command without running a frame.
    pub fn queue_despawn(&mut self, entity: EntityId) {
        self.world
            .resource_or_insert_with(CommandQueue::new)
            .despawn(entity);
    }

    pub fn queue_despawn_entity(&mut self, entity: TestEntity) {
        self.queue_despawn(entity.id());
    }

    /// Queues a sound command without running a frame.
    pub fn queue_play_sound(&mut self, sound: SoundHandle) {
        self.world
            .resource_or_insert_with(CommandQueue::new)
            .play_sound(sound);
    }

    /// The start map's collision tilemap dimensions and data.
    pub fn map(&self) -> &MapData {
        &self.map
    }

    /// Name of the content-runtime map currently spawned.
    pub fn current_map_name(&self) -> Option<String> {
        self.world
            .get_resource::<ContentRuntime>()
            .map(|runtime| runtime.current_map_name().to_owned())
    }

    pub fn current_scene(&self) -> Option<String> {
        self.world
            .get_resource::<SceneState>()
            .map(|scene| scene.current().to_owned())
    }

    pub fn assert_scene(&self, expected: &str) {
        assert_eq!(self.current_scene().as_deref(), Some(expected));
    }

    pub fn assert_map(&self, expected: &str) {
        assert_eq!(self.current_map_name().as_deref(), Some(expected));
    }

    /// UI text produced by the most recent [`Self::frame`] call.
    pub fn ui_text(&self) -> &[String] {
        &self.ui_text
    }

    /// Audio commands produced by processed content commands so far.
    pub fn audio_commands(&self) -> &[AudioCommand] {
        &self.audio_commands
    }

    fn action_id(&self, name: &str) -> ActionId {
        self.input_registry
            .action_id(name)
            .unwrap_or_else(|| panic!("unknown action '{name}'"))
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
                Command::SpawnPrefab {
                    prefab,
                    position,
                    properties,
                } => {
                    self.prefabs
                        .spawn(prefab, &mut self.world, position, &properties)
                        .expect("test command should spawn prefab");
                }
                Command::ChangeMap(map) => self.switch_active_map(map),
                Command::RestartMap => self.switch_active_map(self.active_map),
                Command::RestartStartMap => self.switch_active_map(self.start_map),
            }
        }
    }

    fn switch_active_map(&mut self, map: MapId) {
        let registered = self
            .maps
            .get(map)
            .unwrap_or_else(|| panic!("test command referenced unknown map {map:?}"));
        self.active_map = map;
        self.map = registered.data.clone();
    }
}

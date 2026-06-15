//! Phase 12 second demo. Proves the engine/content split: this crate defines a
//! distinct map, three prefabs (player, chasing enemy, patrolling enemy), and its
//! own systems while depending ONLY on the engine-neutral gameplay crates
//! (`game-core`, `game-map`, `game-ai`, `game-combat`, `game-physics`). It never
//! touches the runtime, renderer, audio, or platform backends, nor `arena-content`.

pub mod actor;
pub mod ai;
pub mod assets;
pub mod combat;
pub mod input;
pub mod level;
pub mod maps;
pub mod prefabs;
pub mod spawn;
pub mod state;
pub mod systems;

use std::rc::Rc;

use anyhow::{Context, Result};
use game_ai::{AiController, Patrol};
use game_combat::{Faction, Health};
use game_core::app::{Ctx, Game, StartCtx};
use game_core::assets::AssetRegistry;
use game_core::builder::{GameBuilder, PrefabId, PrefabRegistry, PrefabValidator};
use game_core::input::InputRegistry;
use game_core::plugin::GamePlugin;
use game_core::world::{Sprite, Transform};
use game_map::{GameMap, MapValidator};
use game_physics::Collider;

use crate::actor::PlayerController;
use crate::assets::TestbedAssets;
use crate::input::TestbedActions;
use crate::prefabs::TestbedPrefabs;

pub struct TestbedPlugin;

pub fn plugin() -> TestbedPlugin {
    TestbedPlugin
}

pub struct TestbedGame {
    assets: TestbedAssets,
    actions: TestbedActions,
    map: GameMap,
    prefabs: Rc<PrefabRegistry>,
}

impl TestbedGame {
    pub fn new() -> Self {
        let mut assets = AssetRegistry::new();
        let testbed_assets = assets::register(&mut assets);
        let mut input = InputRegistry::new();
        let actions = input::register(&mut input);
        let mut prefabs = PrefabRegistry::new();
        let _ = prefabs::register(&mut prefabs, testbed_assets, actions);
        let map = level::testbed_map_from_ron(&prefabs).expect("embedded testbed RON map is valid");
        Self {
            assets: testbed_assets,
            actions,
            map,
            prefabs: Rc::new(prefabs),
        }
    }
}

impl Default for TestbedGame {
    fn default() -> Self {
        Self::new()
    }
}

impl GamePlugin for TestbedPlugin {
    type Game = TestbedGame;

    fn build(&self, app: &mut GameBuilder) -> Result<Self::Game> {
        let assets = assets::register(app.assets_mut());
        let actions = input::register(app.input_mut());

        // Register prefabs exactly once into a registry shared (via `Rc`) by
        // validation, the schedule's systems, and the returned game.
        let mut prefabs = PrefabRegistry::new();
        let prefab_ids = prefabs::register(&mut prefabs, assets, actions);
        // Load the map from the external RON content file (Phase 13).
        let map = level::testbed_map_from_ron(&prefabs)?;
        let start_map = maps::register(app.maps_mut(), &assets, &map);
        app.set_start_map(start_map);

        validate_testbed_content(&prefabs, prefab_ids, &map)?;

        let prefabs = Rc::new(prefabs);
        systems::register(
            app.schedule_mut(),
            assets,
            actions,
            map.clone(),
            Rc::clone(&prefabs),
        );
        Ok(TestbedGame {
            assets,
            actions,
            map,
            prefabs,
        })
    }
}

impl Game for TestbedGame {
    fn start(&mut self, ctx: &mut StartCtx) -> Result<()> {
        systems::startup_system(ctx, self.assets, &self.map, &self.prefabs)
    }

    fn update(&mut self, ctx: &mut Ctx, dt: f32) {
        systems::state_input_system(ctx, &self.map, &self.prefabs, self.actions);
        systems::player_control_system(ctx, dt);
        systems::chase_system(ctx, dt);
        systems::patrol_system(ctx, dt);
        systems::physics_system(ctx, dt);
        systems::combat_system(ctx, self.actions.attack, self.assets.hit, dt);
        systems::death_state_system(ctx, dt);
        systems::camera_follow_player_system(ctx, self.actions, dt);
        systems::testbed_ui_system(ctx, dt);
    }
}

/// Validates the testbed's map and prefab compositions before the runtime enters
/// the main loop (reusing the Phase 11 validators on a second demo).
fn validate_testbed_content(
    prefabs: &PrefabRegistry,
    prefab_ids: TestbedPrefabs,
    map: &GameMap,
) -> Result<()> {
    let known: [PrefabId; 3] = [prefab_ids.player, prefab_ids.chaser, prefab_ids.patroller];
    MapValidator::new()
        .allow_prefabs(known)
        .require_object("player_start")
        .validate(map)
        .context("testbed map validation failed")?;

    let mut validator = PrefabValidator::new(prefabs);
    validator
        .require_component::<Transform>(prefabs::PLAYER)
        .require_component::<Collider>(prefabs::PLAYER)
        .require_component::<Sprite>(prefabs::PLAYER)
        .require_component::<Health>(prefabs::PLAYER)
        .require_component::<Faction>(prefabs::PLAYER)
        .require_component::<PlayerController>(prefabs::PLAYER)
        .require_component::<Transform>(prefabs::CHASER)
        .require_component::<Collider>(prefabs::CHASER)
        .require_component::<Sprite>(prefabs::CHASER)
        .require_component::<Health>(prefabs::CHASER)
        .require_component::<Faction>(prefabs::CHASER)
        .require_component::<AiController>(prefabs::CHASER)
        .require_component::<Transform>(prefabs::PATROLLER)
        .require_component::<Collider>(prefabs::PATROLLER)
        .require_component::<Sprite>(prefabs::PATROLLER)
        .require_component::<Health>(prefabs::PATROLLER)
        .require_component::<Faction>(prefabs::PATROLLER)
        .require_component::<Patrol>(prefabs::PATROLLER);
    validator
        .validate()
        .context("testbed prefab validation failed")?;
    game_map::validate_map_prefabs(map, prefabs)
        .context("testbed map references unknown prefab")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use game_ai::Patrol;
    use game_core::app::{Ctx, Game, MapData, RenderFrame, StartCtx};
    use game_core::audio::{Audio, AudioCommands};
    use game_core::builder::GameBuilder;
    use game_core::camera::Camera2D;
    use game_core::gfx::Gfx;
    use game_core::input::Input;
    use game_core::plugin::GamePlugin;
    use game_core::schedule::ScheduleValidator;
    use game_core::world::World;

    use super::{TestbedGame, TestbedPlugin};

    const DT: f32 = 1.0 / 120.0;

    fn start_testbed() -> (TestbedGame, World, MapData) {
        let mut game = TestbedGame::new();
        let mut world = World::new();
        let mut map_slot = None;
        game.start(&mut StartCtx::new(&mut world, &mut map_slot))
            .unwrap();
        (game, world, map_slot.unwrap())
    }

    fn update_testbed(
        game: &mut TestbedGame,
        world: &mut World,
        map: &MapData,
        input: Input,
    ) -> Vec<String> {
        let mut camera = Camera2D::new(glam::Vec2::ZERO, 1.0);
        let mut frame = RenderFrame::new(camera);
        let mut audio_commands = AudioCommands::default();
        {
            let mut ctx = Ctx {
                world,
                map: &map.tilemap,
                nav: &map.nav,
                input: &input,
                camera: &mut camera,
                gfx: Gfx::new(&mut frame),
                audio: Audio::new(&mut audio_commands),
            };
            game.update(&mut ctx, DT);
        }
        frame.ui_text.into_iter().map(|text| text.text).collect()
    }

    #[test]
    fn testbed_start_spawns_distinct_world() {
        let (_game, world, map) = start_testbed();
        assert_eq!(map.tilemap.width(), 17);
        assert_eq!(map.tilemap.height(), 11);
        assert_eq!(world.ids().count(), 3);
        assert_eq!(world.ids_with::<Patrol>().len(), 1);
    }

    #[test]
    fn testbed_ui_text_is_distinct_from_arena() {
        let (mut game, mut world, map) = start_testbed();
        let ui = update_testbed(&mut game, &mut world, &map, Input::default());
        assert_eq!(ui, vec!["TESTBED"]);
    }

    #[test]
    fn testbed_plugin_builds_and_validates_clean() {
        let mut builder = GameBuilder::new();
        TestbedPlugin.build(&mut builder).unwrap();

        assert!(builder.start_map().is_some());
        ScheduleValidator::new(builder.schedule())
            .start_map_set(builder.start_map().is_some())
            .builtin_render_extract()
            .validate()
            .unwrap();
    }
}

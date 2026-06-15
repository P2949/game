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

use anyhow::{Context, Result};
use game_ai::{AiController, Patrol};
use game_combat::{Faction, Health};
use game_core::builder::{GameBuilder, PrefabId, PrefabRegistry, PrefabValidator};
use game_core::plugin::GamePlugin;
use game_core::world::{Sprite, Transform};
use game_map::{GameMap, MapValidator};
use game_physics::Collider;

use crate::actor::PlayerController;
use crate::prefabs::TestbedPrefabs;

pub struct TestbedPlugin;

pub fn plugin() -> TestbedPlugin {
    TestbedPlugin
}

impl GamePlugin for TestbedPlugin {
    fn build(&self, app: &mut GameBuilder) -> Result<()> {
        let assets = assets::register(app.assets_mut());
        let actions = input::register(app.input_mut());

        // Register prefabs exactly once into the builder-owned registry shared by
        // validation, the runtime content, and the schedule systems.
        let prefab_ids = prefabs::register(app.prefabs_mut(), assets, actions);
        // Load the map from the external RON content file (Phase 13).
        let map = level::testbed_map_from_ron(app.prefabs())?;
        let start_map = maps::register(app.maps_mut(), &assets, &map);
        app.set_start_map(start_map);

        validate_testbed_content(app.prefabs(), prefab_ids, &map)?;

        let prefabs = app.prefabs_shared();
        systems::register(app.schedule_mut(), assets, actions, map.clone(), prefabs);
        Ok(())
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
    use game_core::app::{Ctx, MapData, RenderFrame, StartCtx};
    use game_core::audio::{Audio, AudioCommands};
    use game_core::builder::{GameBuilder, RuntimeContent};
    use game_core::camera::Camera2D;
    use game_core::gfx::Gfx;
    use game_core::input::Input;
    use game_core::plugin::GamePlugin;
    use game_core::schedule::{Schedule, ScheduleValidator};
    use game_core::world::World;

    use super::TestbedPlugin;

    const DT: f32 = 1.0 / 120.0;

    fn start_testbed() -> (Schedule, World, MapData) {
        let mut builder = GameBuilder::new();
        TestbedPlugin.build(&mut builder).unwrap();
        let RuntimeContent {
            maps,
            start_map,
            mut schedule,
            ..
        } = builder.into_parts().unwrap();
        let map = maps.get(start_map).unwrap().data.clone();
        let mut world = World::new();
        schedule
            .run_startup(&mut StartCtx::new(&mut world))
            .unwrap();
        (schedule, world, map)
    }

    fn update_testbed(
        schedule: &mut Schedule,
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
            schedule.run_fixed(&mut ctx, DT);
            schedule.run_update(&mut ctx, DT);
            schedule.run_render_extract(&mut ctx, DT);
            schedule.run_ui(&mut ctx, DT);
        }
        frame.ui_text.into_iter().map(|text| text.text).collect()
    }

    #[test]
    fn testbed_start_spawns_distinct_world() {
        let (_schedule, world, map) = start_testbed();
        assert_eq!(map.tilemap.width(), 17);
        assert_eq!(map.tilemap.height(), 11);
        assert_eq!(world.ids().count(), 3);
        assert_eq!(world.ids_with::<Patrol>().len(), 1);
    }

    #[test]
    fn testbed_ui_text_is_distinct_from_arena() {
        let (mut schedule, mut world, map) = start_testbed();
        let ui = update_testbed(&mut schedule, &mut world, &map, Input::default());
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

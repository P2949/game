use std::rc::Rc;

use anyhow::Result;
use game_core::app::{Ctx, StartCtx};
use game_core::backend::SoundHandle;
use game_core::builder::PrefabRegistry;
use game_core::commands::CommandQueue;
use game_core::input::ActionId;
use game_core::schedule::Schedule;
use game_core::world::World;
use game_map::GameMap;

use crate::assets::TestbedAssets;
use crate::input::TestbedActions;
use crate::state::GameState;
use crate::{ai, combat, level, spawn};

pub fn register(
    schedule: &mut Schedule,
    assets: TestbedAssets,
    actions: TestbedActions,
    map: GameMap,
    prefabs: Rc<PrefabRegistry>,
) {
    let startup_assets = assets;
    let startup_map = map.clone();
    let startup_prefabs = Rc::clone(&prefabs);
    schedule.add_startup(move |ctx| {
        startup_system(ctx, startup_assets, &startup_map, &startup_prefabs)
    });

    let state_map = map.clone();
    let state_prefabs = Rc::clone(&prefabs);
    schedule
        .add_fixed(move |ctx, _dt| state_input_system(ctx, &state_map, &state_prefabs, actions));
    schedule.add_fixed(player_control_system);
    schedule.add_fixed(chase_system);
    schedule.add_fixed(patrol_system);
    schedule.add_fixed(physics_system);

    let hit_sound = assets.hit;
    schedule.add_fixed(move |ctx, dt| combat_system(ctx, actions.attack, hit_sound, dt));
    schedule.add_fixed(death_state_system);
    schedule.add_update(move |ctx, dt| camera_follow_player_system(ctx, actions, dt));
    schedule.add_ui(testbed_ui_system);

    // Every fixed system self-guards via `simulation_active`/`GameState`.
    schedule.mark_fixed_pause_guarded();
}

pub fn startup_system(
    ctx: &mut StartCtx<'_>,
    assets: TestbedAssets,
    map: &GameMap,
    prefabs: &PrefabRegistry,
) -> Result<()> {
    initialize_resources(ctx.world);
    reset_world(ctx.world, map, prefabs)?;
    ctx.set_map(map.collision_tilemap(), level::theme(&assets));
    Ok(())
}

pub fn initialize_resources(world: &mut World) {
    world.resource_or_insert_with(GameState::default);
    world.resource_or_insert_with(CommandQueue::new);
}

pub fn reset_world(world: &mut World, map: &GameMap, prefabs: &PrefabRegistry) -> Result<()> {
    world.clear();
    // `World::clear` preserves resources (including the command queue), so drop
    // any commands queued against the pre-reset world before respawning.
    if let Some(commands) = world.get_resource_mut::<CommandQueue>() {
        commands.clear();
    }
    spawn::spawn_map_objects(world, map, prefabs)
}

pub fn state_input_system(
    ctx: &mut Ctx<'_>,
    map: &GameMap,
    prefabs: &PrefabRegistry,
    actions: TestbedActions,
) {
    initialize_resources(ctx.world);
    let mut state = ctx
        .world
        .get_resource::<GameState>()
        .copied()
        .unwrap_or_default();
    let was_dead = state.player_dead;

    if ctx.input.pressed(actions.pause) {
        state.paused = !state.paused;
    }

    if ctx.input.pressed(actions.reset) {
        reset_world(ctx.world, map, prefabs)
            .expect("testbed map objects reference registered prefabs");
        state = GameState::default();
    }

    if ctx.input.pressed(actions.debug_die) {
        combat::kill_player(ctx.world);
    }

    state.player_dead = combat::player_is_dead(ctx.world);
    if state.player_dead && (ctx.input.pressed(actions.attack) || ctx.input.pressed(actions.reset))
    {
        reset_world(ctx.world, map, prefabs)
            .expect("testbed map objects reference registered prefabs");
        state = GameState::default();
    }

    if state.player_dead && !was_dead {
        combat::emit_player_death(ctx.world);
    }

    if state.player_dead || state.paused {
        ai::stop_all(ctx.world);
    }

    ctx.world.insert_resource(state);
}

pub fn player_control_system(ctx: &mut Ctx<'_>, _dt: f32) {
    if simulation_active(ctx.world) {
        ai::drive_player(ctx.world, ctx.input);
    }
}

pub fn chase_system(ctx: &mut Ctx<'_>, dt: f32) {
    if simulation_active(ctx.world) {
        let target = ai::player_pos(ctx.world);
        game_ai::chase_system(ctx.world, ctx.nav, target, dt);
    }
}

pub fn patrol_system(ctx: &mut Ctx<'_>, dt: f32) {
    if simulation_active(ctx.world) {
        game_ai::patrol_system(ctx.world, dt);
    }
}

pub fn physics_system(ctx: &mut Ctx<'_>, dt: f32) {
    if simulation_active(ctx.world) {
        game_physics::movement_system(ctx.world, ctx.map, dt);
    }
}

pub fn combat_system(ctx: &mut Ctx<'_>, attack: ActionId, hit_sound: SoundHandle, dt: f32) {
    if simulation_active(ctx.world) {
        combat::tick_commands(ctx.world, ctx.input, attack, hit_sound, dt);
    }
}

pub fn death_state_system(ctx: &mut Ctx<'_>, _dt: f32) {
    let mut state = ctx
        .world
        .get_resource::<GameState>()
        .copied()
        .unwrap_or_default();
    let was_dead = state.player_dead;
    state.player_dead = combat::player_is_dead(ctx.world);
    if state.player_dead && !was_dead {
        combat::emit_player_death(ctx.world);
    }
    ctx.world.insert_resource(state);
}

pub fn camera_follow_player_system(ctx: &mut Ctx<'_>, actions: TestbedActions, dt: f32) {
    let zoom_in = ctx.input.down(actions.zoom_in);
    let zoom_out = ctx.input.down(actions.zoom_out);
    if zoom_in != zoom_out {
        let zoom_step = 1.0 + 2.0 * dt;
        let mut zoom = ctx.camera.zoom();
        if zoom_in {
            zoom *= zoom_step;
        } else {
            zoom /= zoom_step;
        }
        ctx.camera.set_zoom(zoom.clamp(0.25, 6.0));
    }

    if let Some(pos) = ai::player_pos(ctx.world) {
        ctx.camera.set_center(pos);
    }
}

pub fn testbed_ui_system(ctx: &mut Ctx<'_>, _dt: f32) {
    let state = ctx
        .world
        .get_resource::<GameState>()
        .copied()
        .unwrap_or_default();
    let label = if state.player_dead {
        "TESTBED - you died"
    } else if state.paused {
        "TESTBED - paused"
    } else {
        "TESTBED"
    };
    ctx.gfx.text(
        label,
        glam::vec2(24.0, 24.0),
        glam::vec4(0.7, 1.0, 0.8, 1.0),
    );
}

fn simulation_active(world: &World) -> bool {
    let state = world
        .get_resource::<GameState>()
        .copied()
        .unwrap_or_default();
    !state.paused && !state.player_dead
}

#[cfg(test)]
mod tests {
    use game_ai::Patrol;
    use game_core::app::{Ctx, RenderFrame, StartCtx};
    use game_core::audio::{Audio, AudioCommands};
    use game_core::camera::Camera2D;
    use game_core::gfx::Gfx;
    use game_core::input::Input;
    use game_core::nav::NavGrid;
    use game_core::world::Velocity;

    use crate::state::GameState;
    use crate::{assets, input, level, prefabs};

    use super::{initialize_resources, startup_system};

    fn build_world_and_map() -> (
        game_core::world::World,
        game_map::GameMap,
        game_core::builder::PrefabRegistry,
        assets::TestbedAssets,
    ) {
        let assets = assets::TestbedAssets::load();
        let mut input_registry = game_core::input::InputRegistry::new();
        let actions = input::register(&mut input_registry);
        let mut prefab_registry = game_core::builder::PrefabRegistry::new();
        let testbed_prefabs = prefabs::register(&mut prefab_registry, assets, actions);
        let map = level::testbed_map(testbed_prefabs);
        (game_core::world::World::new(), map, prefab_registry, assets)
    }

    #[test]
    fn startup_spawns_player_and_two_enemies() {
        let (mut world, map, prefab_registry, assets) = build_world_and_map();
        let mut map_slot = None;

        startup_system(
            &mut StartCtx::new(&mut world, &mut map_slot),
            assets,
            &map,
            &prefab_registry,
        )
        .unwrap();

        assert!(world.get_resource::<GameState>().is_some());
        assert!(map_slot.is_some());
        assert_eq!(world.ids().count(), 3);
        assert_eq!(world.ids_with::<Patrol>().len(), 1);
    }

    #[test]
    fn patrol_enemy_moves_when_simulation_active() {
        let (mut world, map, prefab_registry, assets) = build_world_and_map();
        initialize_resources(&mut world);
        crate::spawn::spawn_map_objects(&mut world, &map, &prefab_registry).unwrap();

        let collision = map.collision_tilemap();
        let nav = NavGrid::from_tilemap(&collision);
        let mut camera = Camera2D::new(glam::Vec2::ZERO, 1.0);
        let mut frame = RenderFrame::new(camera);
        let mut audio_commands = AudioCommands::default();
        let input = Input::default();
        let mut ctx = Ctx {
            world: &mut world,
            map: &collision,
            nav: &nav,
            input: &input,
            camera: &mut camera,
            gfx: Gfx::new(&mut frame),
            audio: Audio::new(&mut audio_commands),
        };

        super::patrol_system(&mut ctx, 1.0 / 120.0);

        let patroller = ctx.world.ids_with::<Patrol>()[0];
        let velocity = ctx.world.get::<Velocity>(patroller).unwrap().0;
        assert!(velocity.length() > 0.0, "patroller should be moving");
        let _ = assets;
    }

    #[test]
    fn ui_renders_distinct_testbed_label() {
        let (mut world, _map, _prefabs, _assets) = build_world_and_map();
        initialize_resources(&mut world);

        let collision = game_core::tilemap::TileMap::from_rows(&["."], 32.0);
        let nav = NavGrid::from_tilemap(&collision);
        let mut camera = Camera2D::new(glam::Vec2::ZERO, 1.0);
        let mut frame = RenderFrame::new(camera);
        let mut audio_commands = AudioCommands::default();
        let input = Input::default();
        {
            let mut ctx = Ctx {
                world: &mut world,
                map: &collision,
                nav: &nav,
                input: &input,
                camera: &mut camera,
                gfx: Gfx::new(&mut frame),
                audio: Audio::new(&mut audio_commands),
            };
            super::testbed_ui_system(&mut ctx, 1.0 / 120.0);
        }

        assert_eq!(
            frame
                .ui_text
                .into_iter()
                .map(|t| t.text)
                .collect::<Vec<_>>(),
            vec!["TESTBED".to_owned()]
        );
    }
}

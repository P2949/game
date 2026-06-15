use std::rc::Rc;

use anyhow::Result;
use game_map::GameMap;

use crate::assets::ArenaAssets;
use crate::combat;
use crate::engine::app::{Ctx, StartCtx};
use crate::engine::backend::SoundHandle;
use crate::engine::builder::PrefabRegistry;
use crate::engine::commands::CommandQueue;
use crate::engine::input::ActionId;
use crate::engine::schedule::Schedule;
use crate::game::World;
use crate::input::ArenaActions;
use crate::state::GameState;
use crate::{ai, level, spawn};

pub fn register(
    schedule: &mut Schedule,
    assets: ArenaAssets,
    actions: ArenaActions,
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
    schedule.add_fixed(chase_player_system);
    schedule.add_fixed(physics_system);

    let hit_sound = assets.hit;
    schedule.add_fixed(move |ctx, dt| combat_system(ctx, actions.attack, hit_sound, dt));
    schedule.add_fixed(death_state_system);
    schedule.add_update(move |ctx, dt| camera_follow_player_system(ctx, actions, dt));
    schedule.add_ui(pause_death_ui_system);

    // Every arena fixed system self-guards via `simulation_active`/`GameState`,
    // so the simulation is safe to step while paused or dead. Declare that to the
    // schedule validator (Phase 11.4).
    schedule.mark_fixed_pause_guarded();
}

pub fn startup_system(
    ctx: &mut StartCtx<'_>,
    assets: ArenaAssets,
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
    actions: ArenaActions,
) {
    initialize_resources(ctx.world);
    let mut state = ctx
        .world
        .get_resource::<GameState>()
        .copied()
        .unwrap_or_default();
    if ctx.input.pressed(actions.pause) {
        state.paused = !state.paused;
    }

    if ctx.input.pressed(actions.reset) {
        reset_world(ctx.world, map, prefabs)
            .expect("arena map objects reference registered prefabs");
        state = GameState::default();
    }

    if ctx.input.pressed(actions.debug_die) {
        combat::kill_player(ctx.world);
    }

    state.player_dead = combat::player_is_dead(ctx.world);
    if state.player_dead && (ctx.input.pressed(actions.attack) || ctx.input.pressed(actions.reset))
    {
        reset_world(ctx.world, map, prefabs)
            .expect("arena map objects reference registered prefabs");
        state = GameState::default();
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

pub fn chase_player_system(ctx: &mut Ctx<'_>, dt: f32) {
    if simulation_active(ctx.world) {
        ai::chase_player(ctx.world, ctx.nav, dt);
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
    state.player_dead = combat::player_is_dead(ctx.world);
    ctx.world.insert_resource(state);
}

pub fn camera_follow_player_system(ctx: &mut Ctx<'_>, actions: ArenaActions, dt: f32) {
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

pub fn pause_death_ui_system(ctx: &mut Ctx<'_>, _dt: f32) {
    let state = ctx
        .world
        .get_resource::<GameState>()
        .copied()
        .unwrap_or_default();
    if state.player_dead {
        ctx.gfx.text(
            "You died",
            glam::vec2(24.0, 24.0),
            glam::vec4(1.0, 0.35, 0.25, 1.0),
        );
    } else if state.paused {
        ctx.gfx.text(
            "Paused",
            glam::vec2(24.0, 24.0),
            glam::vec4(1.0, 0.95, 0.75, 1.0),
        );
    }
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
    use crate::engine::app::{Ctx, RenderFrame, StartCtx};
    use crate::engine::audio::{Audio, AudioCommands};
    use crate::engine::camera::Camera2D;
    use crate::engine::gfx::Gfx;
    use crate::engine::input::Input;
    use crate::engine::world::Velocity;
    use crate::state::GameState;
    use crate::{assets, input, level, prefabs};

    use super::{initialize_resources, startup_system, state_input_system};

    #[test]
    fn startup_system_sets_map_and_game_state_resource() {
        let assets = assets::ArenaAssets::load();
        let mut input_registry = crate::engine::input::InputRegistry::new();
        let actions = input::register(&mut input_registry);
        let mut prefab_registry = crate::engine::builder::PrefabRegistry::new();
        let arena_prefabs = prefabs::register(&mut prefab_registry, assets, actions);
        let map = level::arena_map(arena_prefabs);
        let mut world = crate::game::World::new();
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
        assert_eq!(world.ids().count(), 2);
    }

    #[test]
    fn state_input_pause_stops_existing_velocity() {
        let assets = assets::ArenaAssets::load();
        let mut input_registry = crate::engine::input::InputRegistry::new();
        let actions = input::register(&mut input_registry);
        let mut prefab_registry = crate::engine::builder::PrefabRegistry::new();
        let arena_prefabs = prefabs::register(&mut prefab_registry, assets, actions);
        let map = level::arena_map(arena_prefabs);
        let mut world = crate::game::World::new();
        initialize_resources(&mut world);
        crate::spawn::spawn_map_objects(&mut world, &map, &prefab_registry).unwrap();
        for id in world.ids_with::<Velocity>() {
            world.get_mut::<Velocity>(id).unwrap().0 = glam::Vec2::ONE;
        }

        let collision = map.collision_tilemap();
        let nav = crate::engine::nav::NavGrid::from_tilemap(&collision);
        let mut camera = Camera2D::new(glam::Vec2::ZERO, 1.0);
        let mut frame = RenderFrame::new(camera);
        let mut audio_commands = AudioCommands::default();
        let input = Input::default().with_pressed(actions.pause);
        let mut ctx = Ctx {
            world: &mut world,
            map: &collision,
            nav: &nav,
            input: &input,
            camera: &mut camera,
            gfx: Gfx::new(&mut frame),
            audio: Audio::new(&mut audio_commands),
        };

        state_input_system(&mut ctx, &map, &prefab_registry, actions);

        assert!(world.get_resource::<GameState>().unwrap().paused);
        for id in world.ids_with::<Velocity>() {
            assert_eq!(world.get::<Velocity>(id).unwrap().0, glam::Vec2::ZERO);
        }
    }
}

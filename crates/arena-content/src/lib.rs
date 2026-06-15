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

// TEMP: compatibility modules for the Phase 2 physical split. Arena content
// still uses old single-crate paths internally, but only depends on game-core.
pub mod engine {
    pub use game_core::*;
}

pub mod game {
    pub use crate::{
        ArenaGame, ArenaPlugin, World, actor, ai, assets, combat, input, level, maps, prefabs,
        spawn, state, systems,
    };
}

use anyhow::{Context, Result};
use std::rc::Rc;

use crate::actor::PlayerController;
use crate::assets::ArenaAssets;
use crate::engine::app::{Ctx, Game, StartCtx};
use crate::engine::assets::AssetRegistry;
use crate::engine::builder::{GameBuilder, PrefabId, PrefabRegistry, PrefabValidator};
use crate::engine::plugin::GamePlugin;
use crate::engine::world::{Sprite, Transform};
use crate::prefabs::ArenaPrefabs;
use game_ai::AiController;
use game_combat::{Faction, Health};
use game_map::{GameMap, MapValidator};
use game_physics::Collider;

pub type World = crate::engine::world::World;

pub struct ArenaPlugin;

pub fn plugin() -> ArenaPlugin {
    ArenaPlugin
}

pub struct ArenaGame {
    assets: ArenaAssets,
    map: GameMap,
    prefabs: Rc<PrefabRegistry>,
}

impl ArenaGame {
    pub fn new() -> Self {
        let mut assets = AssetRegistry::new();
        let arena_assets = assets::register(&mut assets);
        let mut input = crate::engine::input::InputRegistry::new();
        let actions = input::register(&mut input);
        let mut prefabs = PrefabRegistry::new();
        let arena_prefabs = prefabs::register(&mut prefabs, arena_assets, actions);
        let map = level::arena_map(arena_prefabs);
        Self::with_content(arena_assets, map, prefabs)
    }

    pub fn with_content(assets: ArenaAssets, map: GameMap, prefabs: PrefabRegistry) -> Self {
        Self::with_shared_content(assets, map, Rc::new(prefabs))
    }

    pub fn with_shared_content(
        assets: ArenaAssets,
        map: GameMap,
        prefabs: Rc<PrefabRegistry>,
    ) -> Self {
        Self {
            assets,
            map,
            prefabs,
        }
    }
}

impl GamePlugin for ArenaPlugin {
    type Game = ArenaGame;

    fn build(&self, app: &mut GameBuilder) -> Result<Self::Game> {
        let assets = assets::register(app.assets_mut());
        let actions = input::register(app.input_mut());
        let _builder_prefabs = prefabs::register(app.prefabs_mut(), assets, actions);
        let mut runtime_prefabs = PrefabRegistry::new();
        let runtime_prefab_ids = prefabs::register(&mut runtime_prefabs, assets, actions);
        let (start_map, map) = maps::register(app.maps_mut(), &assets, runtime_prefab_ids);
        app.set_start_map(start_map);

        // Phase 11: fail before the runtime enters the main loop if the arena's
        // map, prefab compositions, or object references are malformed.
        validate_arena_content(&runtime_prefabs, runtime_prefab_ids, &map)?;

        let runtime_prefabs = Rc::new(runtime_prefabs);
        systems::register(
            app.schedule_mut(),
            assets,
            map.clone(),
            Rc::clone(&runtime_prefabs),
        );
        Ok(ArenaGame::with_shared_content(assets, map, runtime_prefabs))
    }
}

/// Validates arena maps and prefabs (Phase 11.1/11.2). Runs during plugin build,
/// which the runtime performs before creating any backend or entering the loop.
fn validate_arena_content(
    prefabs: &PrefabRegistry,
    prefab_ids: ArenaPrefabs,
    map: &GameMap,
) -> Result<()> {
    let known: [PrefabId; 2] = [prefab_ids.player, prefab_ids.slime];
    MapValidator::new()
        .allow_prefabs(known)
        .require_object("player_start")
        .validate(map)
        .context("arena map validation failed")?;

    let mut validator = PrefabValidator::new(prefabs);
    validator
        .require_component::<Transform>(prefabs::PLAYER)
        .require_component::<Collider>(prefabs::PLAYER)
        .require_component::<Sprite>(prefabs::PLAYER)
        .require_component::<Health>(prefabs::PLAYER)
        .require_component::<PlayerController>(prefabs::PLAYER)
        .require_component::<Transform>(prefabs::SLIME)
        .require_component::<Collider>(prefabs::SLIME)
        .require_component::<Sprite>(prefabs::SLIME)
        .require_component::<Health>(prefabs::SLIME)
        .require_component::<Faction>(prefabs::SLIME)
        .require_component::<AiController>(prefabs::SLIME);
    validator
        .validate()
        .context("arena prefab validation failed")?;
    validator
        .validate_map_references(map)
        .context("arena map references unknown prefab")?;
    Ok(())
}

impl Default for ArenaGame {
    fn default() -> Self {
        Self::new()
    }
}

impl Game for ArenaGame {
    fn start(&mut self, ctx: &mut StartCtx) -> Result<()> {
        systems::startup_system(ctx, self.assets, &self.map, &self.prefabs)
    }

    fn update(&mut self, ctx: &mut Ctx, dt: f32) {
        systems::state_input_system(ctx, &self.map, &self.prefabs);
        systems::player_control_system(ctx, dt);
        systems::chase_player_system(ctx, dt);
        systems::physics_system(ctx, dt);
        combat::tick(ctx.world, ctx.input, &mut ctx.audio, self.assets.hit, dt);
        systems::death_state_system(ctx, dt);
        systems::camera_follow_player_system(ctx, dt);
        systems::pause_death_ui_system(ctx, dt);
    }
}

#[cfg(test)]
mod tests {
    use game_combat::Health;
    use game_core::app::{Ctx, Game, MapData, RenderFrame, StartCtx};
    use game_core::audio::{Audio, AudioCommands};
    use game_core::camera::Camera2D;
    use game_core::gfx::Gfx;
    use game_core::input::{FrameActions, Input};
    use game_core::world::{EntityId, Transform, Velocity};

    use crate::actor::{EnemyTag, PlayerController};
    use crate::{ArenaGame, World, spawn};

    const DT: f32 = 1.0 / 120.0;

    fn start_arena() -> (ArenaGame, World, MapData) {
        let mut game = ArenaGame::new();
        let mut world = World::new();
        let mut map_slot = None;
        let mut start_ctx = StartCtx::new(&mut world, &mut map_slot);

        game.start(&mut start_ctx).unwrap();

        (game, world, map_slot.unwrap())
    }

    fn update_arena(
        game: &mut ArenaGame,
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

    fn player_id(world: &World) -> EntityId {
        world.ids_with::<PlayerController>()[0]
    }

    fn enemy_ids(world: &World) -> Vec<EntityId> {
        world.ids_with::<EnemyTag>()
    }

    fn pos(world: &World, id: EntityId) -> glam::Vec2 {
        world.get::<Transform>(id).unwrap().pos
    }

    fn set_pos(world: &mut World, id: EntityId, pos: glam::Vec2) {
        world.get_mut::<Transform>(id).unwrap().pos = pos;
    }

    fn velocity(world: &World, id: EntityId) -> glam::Vec2 {
        world.get::<Velocity>(id).unwrap().0
    }

    fn set_all_velocities(world: &mut World, velocity: glam::Vec2) {
        for id in world.ids_with::<Velocity>() {
            world.get_mut::<Velocity>(id).unwrap().0 = velocity;
        }
    }

    #[test]
    fn arena_start_snapshot_sets_map_and_spawn_positions() {
        let (_game, world, map) = start_arena();

        assert_eq!(map.tilemap.width(), 15);
        assert_eq!(map.tilemap.height(), 9);
        assert_eq!(world.ids_with::<PlayerController>().len(), 1);
        assert_eq!(world.ids_with::<EnemyTag>().len(), 1);

        let player = player_id(&world);
        assert_eq!(pos(&world, player), map.tilemap.cell_center(3, 4));

        let enemy = enemy_ids(&world).pop().unwrap();
        assert_eq!(pos(&world, enemy), map.tilemap.cell_center(9, 4));
    }

    #[test]
    fn arena_update_snapshot_player_input_sets_velocity() {
        let (mut game, mut world, map) = start_arena();

        update_arena(
            &mut game,
            &mut world,
            &map,
            Input::new(glam::vec2(1.0, 0.0), 0.0, FrameActions::default()),
        );

        assert_eq!(velocity(&world, player_id(&world)), glam::vec2(130.0, 0.0));
    }

    #[test]
    fn arena_update_snapshot_enemy_chases_player_through_nav_grid() {
        let (mut game, mut world, map) = start_arena();
        let player_pos = pos(&world, player_id(&world));
        for id in enemy_ids(&world) {
            set_pos(&mut world, id, player_pos + glam::vec2(96.0, 0.0));
        }

        update_arena(
            &mut game,
            &mut world,
            &map,
            Input::new(glam::Vec2::ZERO, 0.0, FrameActions::default()),
        );

        let enemy = enemy_ids(&world).pop().unwrap();
        assert!(velocity(&world, enemy).x < 0.0);
    }

    #[test]
    fn arena_update_snapshot_player_attack_damages_nearest_enemy() {
        let (mut game, mut world, map) = start_arena();
        let player_pos = pos(&world, player_id(&world));
        for id in enemy_ids(&world) {
            set_pos(&mut world, id, player_pos + glam::vec2(29.0, 0.0));
        }
        let near_enemy_id = spawn::spawn_enemy(
            &mut world,
            player_pos + glam::vec2(20.0, 0.0),
            &crate::assets::ArenaAssets::load(),
        );

        update_arena(
            &mut game,
            &mut world,
            &map,
            Input::new(
                glam::Vec2::ZERO,
                0.0,
                FrameActions {
                    action_pressed: true,
                    ..Default::default()
                },
            ),
        );

        assert_eq!(world.get::<Health>(near_enemy_id).unwrap().current, 15);

        let far_enemy = enemy_ids(&world)
            .into_iter()
            .find(|id| pos(&world, *id).x > player_pos.x + 25.0)
            .unwrap();
        assert_eq!(world.get::<Health>(far_enemy).unwrap().current, 40);
    }

    #[test]
    fn arena_update_snapshot_enemy_attack_damages_player() {
        let (mut game, mut world, map) = start_arena();
        let player_pos = pos(&world, player_id(&world));
        for id in enemy_ids(&world) {
            set_pos(&mut world, id, player_pos + glam::vec2(10.0, 0.0));
        }

        update_arena(
            &mut game,
            &mut world,
            &map,
            Input::new(glam::Vec2::ZERO, 0.0, FrameActions::default()),
        );

        assert_eq!(world.get::<Health>(player_id(&world)).unwrap().current, 94);
    }

    #[test]
    fn arena_update_snapshot_reset_clears_and_respawns_world() {
        let (mut game, mut world, map) = start_arena();
        spawn::spawn_enemy(
            &mut world,
            glam::vec2(64.0, 64.0),
            &crate::assets::ArenaAssets::load(),
        );
        let player = player_id(&world);
        set_pos(&mut world, player, glam::vec2(64.0, 64.0));

        update_arena(
            &mut game,
            &mut world,
            &map,
            Input::new(
                glam::Vec2::ZERO,
                0.0,
                FrameActions {
                    reset_pressed: true,
                    ..Default::default()
                },
            ),
        );

        assert_eq!(world.ids().count(), 2);
        assert_eq!(
            pos(&world, player_id(&world)),
            map.tilemap.cell_center(3, 4)
        );
        assert_eq!(
            pos(&world, enemy_ids(&world).pop().unwrap()),
            map.tilemap.cell_center(9, 4)
        );
    }

    #[test]
    fn arena_update_snapshot_pause_stops_simulation_velocity() {
        let (mut game, mut world, map) = start_arena();
        set_all_velocities(&mut world, glam::vec2(5.0, 7.0));

        let ui_text = update_arena(
            &mut game,
            &mut world,
            &map,
            Input::new(
                glam::Vec2::ZERO,
                0.0,
                FrameActions {
                    pause_pressed: true,
                    ..Default::default()
                },
            ),
        );

        assert!(
            world
                .ids_with::<Velocity>()
                .into_iter()
                .all(|id| velocity(&world, id) == glam::Vec2::ZERO)
        );
        assert_eq!(ui_text, vec!["Paused"]);
    }

    #[test]
    fn arena_update_snapshot_death_screen_path_stops_entities() {
        let (mut game, mut world, map) = start_arena();
        set_all_velocities(&mut world, glam::vec2(5.0, 7.0));
        let player = player_id(&world);
        let health = world.get_mut::<Health>(player).unwrap();
        health.damage(health.current);

        let ui_text = update_arena(
            &mut game,
            &mut world,
            &map,
            Input::new(glam::Vec2::ZERO, 0.0, FrameActions::default()),
        );

        assert!(
            world
                .ids_with::<Velocity>()
                .into_iter()
                .all(|id| velocity(&world, id) == glam::Vec2::ZERO)
        );
        assert_eq!(ui_text, vec!["You died"]);
    }

    #[test]
    fn arena_plugin_builds_and_validates_clean() {
        use game_core::builder::GameBuilder;
        use game_core::plugin::GamePlugin;
        use game_core::schedule::ScheduleValidator;

        let mut builder = GameBuilder::new();
        crate::ArenaPlugin.build(&mut builder).unwrap();

        assert!(builder.start_map().is_some());
        // The arena schedule satisfies validation given runtime-provided extraction.
        ScheduleValidator::new(builder.schedule())
            .start_map_set(builder.start_map().is_some())
            .builtin_render_extract()
            .validate()
            .unwrap();
    }

    #[test]
    fn arena_content_validation_rejects_missing_player_spawn() {
        use crate::engine::builder::PrefabRegistry;
        use crate::engine::input::InputRegistry;
        use game_map::{MapBuilder, cell};

        let assets = crate::assets::ArenaAssets::load();
        let mut input = InputRegistry::new();
        let actions = crate::input::register(&mut input);
        let mut prefabs = PrefabRegistry::new();
        let ids = crate::prefabs::register(&mut prefabs, assets, actions);

        let map = MapBuilder::new("arena", crate::level::TILE)
            .tile_layer("collision", &["###", "#.#", "###"])
            .object("enemy_01", ids.slime, cell(1, 1))
            .finish();

        let err = crate::validate_arena_content(&prefabs, ids, &map).unwrap_err();
        assert!(format!("{err:#}").contains("player_start"));
    }
}

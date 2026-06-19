//! Declarative beginner rules builder.

use glam::{vec2, vec4};

use crate::app::GameApp;
use crate::beginner::actors::{Door, DoorAction, DoorTarget};
use crate::beginner::events::DEFAULT_PICKUP_COLLECT_RANGE;
use crate::context::GameCtx;
use crate::input::TopDownControls;

const DEFAULT_DOOR_TRIGGER_RANGE: f32 = 28.0;

pub struct RulesAuthor<'a, 'app> {
    app: &'a mut GameApp<'app>,
    top_down: Option<TopDownControls>,
    collect_pickups: bool,
    doors_change_maps: bool,
    enemies_damage_player: bool,
    dead_enemies_despawn: bool,
    camera_follows_player: bool,
    pause_and_reset: bool,
    basic_ui: bool,
}

impl<'a, 'app> RulesAuthor<'a, 'app> {
    pub(crate) fn new(app: &'a mut GameApp<'app>) -> Self {
        Self {
            app,
            top_down: None,
            collect_pickups: false,
            doors_change_maps: false,
            enemies_damage_player: false,
            dead_enemies_despawn: false,
            camera_follows_player: false,
            pause_and_reset: false,
            basic_ui: false,
        }
    }

    pub fn top_down_controls(mut self, controls: TopDownControls) -> Self {
        self.top_down = Some(controls);
        self
    }

    pub fn controls(self, controls: TopDownControls) -> Self {
        self.top_down_controls(controls)
    }

    pub fn player_collects_pickups(mut self) -> Self {
        self.collect_pickups = true;
        self
    }

    pub fn doors_change_maps(mut self) -> Self {
        self.doors_change_maps = true;
        self
    }

    pub fn enemies_damage_player(mut self) -> Self {
        self.enemies_damage_player = true;
        self
    }

    pub fn dead_enemies_despawn(mut self) -> Self {
        self.dead_enemies_despawn = true;
        self
    }

    pub fn camera_follows_player(mut self) -> Self {
        self.camera_follows_player = true;
        self
    }

    pub fn pause_and_reset(mut self) -> Self {
        self.pause_and_reset = true;
        self
    }

    pub fn show_basic_ui(mut self) -> Self {
        self.basic_ui = true;
        self
    }

    pub fn build(self) {
        let app = self.app;

        if self.top_down.is_some()
            || self.enemies_damage_player
            || self.camera_follows_player
            || self.pause_and_reset
        {
            let mut top_down = app.use_top_down_game();
            if let Some(controls) = self.top_down {
                top_down = top_down.controls(controls);
            }
            if self.enemies_damage_player {
                top_down = top_down.with_melee_combat().with_enemy_chase();
            }
            if self.camera_follows_player {
                top_down = top_down.with_camera_follow();
            }
            if self.pause_and_reset {
                top_down = top_down.with_pause_death_ui();
            }
            top_down.build();
        }

        if self.collect_pickups {
            app.every_tick(|game: &mut GameCtx<'_, '_>, _dt| {
                game.collect_pickups_near_player(DEFAULT_PICKUP_COLLECT_RANGE);
            });
        }

        if self.doors_change_maps {
            app.every_tick(|game: &mut GameCtx<'_, '_>, _dt| {
                doors_change_maps_system(game);
            });
        }

        if self.dead_enemies_despawn {
            app.every_tick(|game: &mut GameCtx<'_, '_>, _dt| {
                game.enemies().dead().despawn();
            });
        }

        if self.basic_ui {
            app.draw_ui(basic_ui_system);
        }
    }
}

fn basic_ui_system(game: &mut GameCtx<'_, '_>, _dt: f32) {
    let score = game.score().value();
    game.text(
        &format!("Score: {score}"),
        vec2(24.0, 72.0),
        vec4(1.0, 0.95, 0.35, 1.0),
    );
}

fn doors_change_maps_system(game: &mut GameCtx<'_, '_>) {
    let Some(player_pos) = game.player_position() else {
        return;
    };

    let actions = game
        .entities_with::<Door>()
        .into_iter()
        .filter_map(|id| {
            let door_pos = game.position(id)?;
            if door_pos.distance(player_pos) > DEFAULT_DOOR_TRIGGER_RANGE {
                return None;
            }

            let target = game.component::<DoorTarget>(id)?.clone();
            if target.requires_all_enemies_dead && game.enemies().alive().count() > 0 {
                return None;
            }
            Some(target.action)
        })
        .collect::<Vec<_>>();

    for action in actions {
        match action {
            DoorAction::ChangeMap(map) => game.change_map_or_log(&map),
            DoorAction::ChangeScene(scene) => game.change_scene_or_log(&scene),
            DoorAction::RestartLevel => game.restart_level(),
        }
    }
}

#[cfg(test)]
mod tests {
    use game_core::backend::TextureHandle;

    use crate::app::{GameApp, GamePlugin};
    use crate::beginner::actors::Pickup;
    use crate::beginner::collections::Score;
    use crate::harness::GameTestHarness;

    struct RulesPlugin;

    impl GamePlugin for RulesPlugin {
        fn build(&self, game: &mut GameApp<'_>) -> anyhow::Result<()> {
            let controls = game.input(|input| input.top_down_controls())?;

            game.player_prefab("player")
                .sprite(TextureHandle(1))
                .moves_with(controls.movement, 130.0)
                .build()?;

            game.enemy_prefab("slime")
                .sprite(TextureHandle(2))
                .health(10)
                .build()?;

            game.pickup_prefab("coin")
                .sprite(TextureHandle(3))
                .score(1)
                .despawn_on_collect()
                .build()?;

            game.map("rules")
                .tile_size(16.0)
                .tiles(["#####", "#PCE#", "#####"])
                .simple_theme(TextureHandle(10), TextureHandle(11))
                .legend('P', "player")
                .legend('C', "coin")
                .legend('E', "slime")
                .start();

            game.every_tick(|game: &mut GameCtx<'_, '_>, _dt| {
                game.enemies().alive().damage(100);
            });

            game.rules()
                .top_down_controls(controls)
                .player_collects_pickups()
                .dead_enemies_despawn()
                .camera_follows_player()
                .pause_and_reset()
                .show_basic_ui()
                .build();

            Ok(())
        }
    }

    use crate::context::GameCtx;

    #[test]
    fn rules_builder_registers_common_beginner_rules() {
        let mut game = GameTestHarness::from_plugin(RulesPlugin).unwrap();

        game.step();

        assert_eq!(game.enemy_count(), 0);
        assert_eq!(game.count::<Pickup>(), 0);
        assert_eq!(game.world().get_resource::<Score>().unwrap().value, 1);
        game.frame(0.0);
        game.assert_ui_contains("Score: 1");
    }
}

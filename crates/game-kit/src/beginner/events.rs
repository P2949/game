//! Beginner event helpers.

pub const DEFAULT_PICKUP_COLLECT_RANGE: f32 = 24.0;

#[cfg(test)]
mod tests {
    use game_core::backend::{SoundHandle, TextureHandle};

    use crate::app::{GameApp, GamePlugin};
    use crate::beginner::camera::CameraShake;
    use crate::beginner::collections::Score;
    use crate::beginner::state::SimpleGameState;
    use crate::context::{GameCtx, StartupGameCtx};
    use crate::harness::GameTestHarness;

    struct EventPlugin;

    impl GamePlugin for EventPlugin {
        fn build(&self, game: &mut GameApp<'_>) -> anyhow::Result<()> {
            let controls = game.input(|input| input.top_down_controls())?;

            game.scene("menu").start_scene("menu");

            game.player_prefab("player")
                .sprite(TextureHandle(1))
                .moves_with(controls.movement, 130.0)
                .build()?;

            game.pickup_prefab("coin")
                .sprite(TextureHandle(2))
                .score(3)
                .play_sound(SoundHandle(1))
                .despawn_on_collect()
                .build()?;

            game.map("events")
                .tiles(["#####", "#PC.#", "#####"])
                .simple_theme(TextureHandle(10), TextureHandle(11))
                .legend('P', "player")
                .legend('C', "coin")
                .start();

            game.on_start(|game: &mut StartupGameCtx<'_, '_>| {
                game.init_resource::<SimpleGameState>();
                game.spawn_start_map()
            });

            game.on_action(controls.attack, |game: &mut GameCtx<'_, '_>| {
                game.score().add(1);
            });
            game.on_action_when_playing(controls.attack, |game: &mut GameCtx<'_, '_>| {
                game.score().add(10);
            });
            game.on_action_cooldown(controls.attack, 1.0, |game: &mut GameCtx<'_, '_>| {
                game.camera2d().shake(0.25);
                game.score().add(100_000);
            });
            game.every_seconds(0.5, |game: &mut GameCtx<'_, '_>| {
                game.score().add(100);
            });
            game.after_seconds(0.25, |game: &mut GameCtx<'_, '_>| {
                game.score().add(1000);
            });
            game.on_scene_enter("menu", |game: &mut GameCtx<'_, '_>| {
                game.score().add(10_000);
            });
            game.on_scene("menu", |game: &mut GameCtx<'_, '_>, _dt| {
                game.score().add(5);
            });
            game.on_player_collect_pickup_within(40.0, |game: &mut GameCtx<'_, '_>| {
                game.score().add(20);
            });

            Ok(())
        }
    }

    #[test]
    fn event_helpers_register_beginner_systems() {
        let mut game = GameTestHarness::from_plugin(EventPlugin).unwrap();

        game.frame(0.0);
        game.tap_action("attack");
        game.tap_action("attack");
        game.step_seconds(0.5);

        assert_eq!(game.world().get_resource::<Score>().unwrap().value, 111_150);
        assert!(game.world().get_resource::<CameraShake>().is_some());
        game.assert_sound_played();
    }
}

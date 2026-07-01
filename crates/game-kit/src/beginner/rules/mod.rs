//! Declarative beginner rules builder.

mod animation;
mod author;
mod checkpoints;
mod combat;
mod doors;
mod pickups;
mod projectiles;
mod shared;
mod spawners;
mod ui;
mod win_conditions;

pub use animation::{
    DeathAnimationBehavior, DeathAnimationDespawnBehavior, RulesAnimationUpdateBehavior,
    RulesEnemyAnimationByMovementBehavior, RulesEnemyDirectionalAnimationBehavior,
    RulesPlayerDirectionalAnimationBehavior,
};
pub use author::RulesAuthor;
pub use checkpoints::{CheckpointActivationBehavior, CheckpointRespawnBehavior};
pub use combat::{DeadEnemiesDespawnBehavior, EnemyDropsBehavior};
pub use doors::DoorsChangeMapsBehavior;
pub use pickups::CollectPickupsBehavior;
pub use projectiles::{
    ProjectileDamageBehavior, ProjectileImpactDespawnBehavior, ProjectileLifetimeBehavior,
    ProjectileMovementBehavior,
};
pub use spawners::SpawnerBehavior;
pub use ui::HighLevelUiBehavior;
pub use win_conditions::WinConditionBehavior;

#[cfg(test)]
mod tests {
    use game_core::backend::TextureHandle;
    use game_core::world::{Transform, Velocity};

    use crate::app::{GameApp, GamePlugin};
    use crate::beginner::actors::{
        Checkpoint, CheckpointState, Enemy, FacingDirection, Pickup, PlayerProjectile, Projectile,
        Spawner,
    };
    use crate::beginner::animation::{Animation, AnimationSet, SpriteSheet};
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

            game.every_tick(|game, _dt| {
                game.enemies().alive().damage(100);
            });

            game.rules()
                .top_down_controls(controls)
                .player_collects_pickups()
                .dead_enemies_despawn()
                .camera_follows_player()
                .pause_and_reset()
                .show_basic_ui()
                .show_enemy_count()
                .show_player_health()
                .show_game_over_text()
                .show_pause_menu()
                .build();

            Ok(())
        }
    }

    struct InvalidRuleComboPlugin;

    impl GamePlugin for InvalidRuleComboPlugin {
        fn build(&self, game: &mut GameApp<'_>) -> anyhow::Result<()> {
            game.rules().projectiles_damage_enemies().build();
            Ok(())
        }
    }

    #[test]
    fn rules_builder_reports_missing_rule_dependencies() {
        let panic = std::panic::catch_unwind(|| {
            let _ = GameTestHarness::from_plugin(InvalidRuleComboPlugin);
        })
        .expect_err("invalid rule combination should panic with a beginner diagnostic");
        let message = panic
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| panic.downcast_ref::<&str>().copied())
            .unwrap_or("<non-string panic>");

        assert!(message.contains("Rule `projectiles_damage_enemies` needs the `projectiles` rule"));
        assert!(message.contains("Add `.projectiles()`"));
    }

    #[test]
    fn rules_builder_registers_common_beginner_rules() {
        let mut game = GameTestHarness::from_plugin(RulesPlugin).unwrap();

        game.step();

        assert_eq!(game.enemy_count(), 0);
        assert_eq!(game.count::<Pickup>(), 0);
        assert_eq!(game.world().get_resource::<Score>().unwrap().value, 1);
        game.frame(0.0);
        game.assert_ui_contains("Score: 1");
        game.assert_ui_contains("Enemies: 0");
        game.assert_ui_contains("Health:");

        let player = game.player();
        game.set_entity_health(player, 0);
        game.frame(0.0);
        game.assert_ui_contains("Game Over");

        game.tap_action("pause");
        game.frame(0.0);
        game.assert_ui_contains("Paused");
    }

    struct ProjectileRulesPlugin;

    impl GamePlugin for ProjectileRulesPlugin {
        fn build(&self, game: &mut GameApp<'_>) -> anyhow::Result<()> {
            game.asset_bag()
                .texture("player", "textures/player.png")?
                .texture("slime", "textures/slime.png")?
                .texture("bolt", "textures/bolt.png")?
                .texture("floor", "textures/floor.png")?
                .texture("wall", "textures/wall.png")?
                .build();
            let controls = game.input(|input| input.top_down_controls())?;

            game.player_prefab("player")
                .sprite("player")
                .moves_with(controls.movement, 120.0)
                .build()?;
            game.enemy_prefab("slime")
                .sprite("slime")
                .health(20)
                .build()?;
            game.projectile_prefab("bolt")
                .sprite("bolt")
                .speed(120.0)
                .damage(20)
                .lifetime(0.25)
                .despawn_on_hit()
                .build()?;
            game.map("projectiles")
                .tile_size(16.0)
                .tiles(["#######", "#P..E.#", "#######"])
                .simple_theme("floor", "wall")
                .legend('P', "player")
                .legend('E', "slime")
                .start();
            game.on_start(|game| game.spawn_start_map());

            game.rules().projectiles().build();
            game.on_action(controls.attack, |game| {
                game.player().shoot("bolt").right();
            });
            game.on_action(controls.reset, |game| {
                game.player().shoot("bolt").left();
            });
            Ok(())
        }
    }

    #[test]
    fn projectile_rules_move_damage_expire_and_despawn_player_shots() {
        let mut hit_game = GameTestHarness::from_plugin(ProjectileRulesPlugin).unwrap();
        hit_game.tap_action("attack");
        assert_eq!(hit_game.count::<Projectile>(), 1);
        let projectile = hit_game.world().ids_with::<Projectile>()[0];
        assert_eq!(hit_game.count::<PlayerProjectile>(), 1);
        assert_eq!(
            hit_game.world().get::<Velocity>(projectile).unwrap().0,
            glam::vec2(120.0, 0.0)
        );
        let enemy = hit_game.world().ids_with::<Enemy>()[0];
        assert_eq!(
            hit_game.world().get::<Transform>(projectile).unwrap().pos,
            glam::vec2(24.0, 24.0)
        );
        assert_eq!(
            hit_game.world().get::<Transform>(enemy).unwrap().pos,
            glam::vec2(72.0, 24.0)
        );
        hit_game.fixed_step(0.5);
        assert_eq!(hit_game.count::<Projectile>(), 0);
        hit_game.assert_enemy_dead(0);

        let mut expiry_game = GameTestHarness::from_plugin(ProjectileRulesPlugin).unwrap();
        expiry_game.tap_action("reset");
        assert_eq!(expiry_game.count::<Projectile>(), 1);
        expiry_game.fixed_step(0.3);
        assert_eq!(expiry_game.count::<Projectile>(), 0);
        assert_eq!(expiry_game.count::<Enemy>(), 1);
    }

    struct SpawnerRulesPlugin;

    impl GamePlugin for SpawnerRulesPlugin {
        fn build(&self, game: &mut GameApp<'_>) -> anyhow::Result<()> {
            game.asset_bag()
                .texture("player", "textures/player.png")?
                .texture("slime", "textures/slime.png")?
                .texture("floor", "textures/floor.png")?
                .texture("wall", "textures/wall.png")?
                .build();
            let controls = game.input(|input| input.top_down_controls())?;
            game.player_prefab("player")
                .sprite("player")
                .moves_with(controls.movement, 120.0)
                .build()?;
            game.enemy_prefab("slime")
                .sprite("slime")
                .health(10)
                .build()?;
            game.spawner_prefab("spawner")
                .spawn("slime")
                .every_seconds(0.2)
                .max_alive(2)
                .near_player(32.0)
                .build()?;
            game.map("waves")
                .tile_size(16.0)
                .tiles(["#######", "#P...S#", "#######"])
                .simple_theme("floor", "wall")
                .legend('P', "player")
                .legend('S', "spawner")
                .start();
            game.on_start(|game| game.spawn_start_map());
            game.rules().spawners_spawn_prefabs().build();
            Ok(())
        }
    }

    #[test]
    fn spawner_rule_places_prefabs_near_player_and_respects_max_alive() {
        let mut game = GameTestHarness::from_plugin(SpawnerRulesPlugin).unwrap();
        assert_eq!(game.count::<Spawner>(), 1);

        game.fixed_step(0.2);
        assert_eq!(game.enemy_count(), 1);
        assert_eq!(
            game.enemy(0).position(),
            game.player().position() + glam::vec2(32.0, 0.0)
        );

        game.fixed_step(0.2);
        assert_eq!(game.enemy_count(), 2);
        game.fixed_step(0.2);
        assert_eq!(game.enemy_count(), 2);
    }

    struct AnimatedEnemyRulesPlugin;

    impl GamePlugin for AnimatedEnemyRulesPlugin {
        fn build(&self, game: &mut GameApp<'_>) -> anyhow::Result<()> {
            let sheet = SpriteSheet::new(TextureHandle(2), 4, 1);
            game.enemy_prefab("slime")
                .spritesheet(sheet)
                .idle(0..1)
                .walk(1..2)
                .die(2..4)
                .despawn_after_death_animation()
                .build()?;
            game.map("animated-enemy")
                .tiles(["###", "#E#", "###"])
                .simple_theme(TextureHandle(10), TextureHandle(11))
                .legend('E', "slime")
                .start();
            game.on_start(|game| game.spawn_start_map());
            game.rules()
                .animate_enemies_by_movement()
                .dead_enemies_play_death_animation()
                .dead_enemies_despawn_after_animation()
                .build();
            Ok(())
        }
    }

    #[test]
    fn animation_rules_drive_enemy_movement_and_despawn_after_death_clip() {
        let mut game = GameTestHarness::from_plugin(AnimatedEnemyRulesPlugin).unwrap();
        let enemy = game.enemy(0);
        game.world_mut().get_mut::<Velocity>(enemy.id()).unwrap().0 = glam::Vec2::X;
        game.frame(0.0);
        assert_eq!(
            game.world().get::<Animation>(enemy.id()).unwrap().current,
            "walk"
        );

        game.set_enemy_health(0, 0);
        game.step();
        assert_eq!(
            game.world().get::<Animation>(enemy.id()).unwrap().current,
            "die"
        );
        assert!(game.world().get::<AnimationSet>(enemy.id()).is_some());

        game.frame(0.4);
        game.step();
        game.assert_enemy_count(0);
    }

    struct RecipeRulesPlugin;

    impl GamePlugin for RecipeRulesPlugin {
        fn build(&self, game: &mut GameApp<'_>) -> anyhow::Result<()> {
            game.start_scene("game").scene("win");
            let controls = game.input(|input| input.top_down_controls())?;

            game.player_prefab("player")
                .sprite(TextureHandle(1))
                .health(100)
                .moves_with(controls.movement, 130.0)
                .build()?;
            game.pickup_prefab("heart")
                .sprite(TextureHandle(2))
                .heal_player(25)
                .despawn_on_collect()
                .build()?;
            game.enemy_prefab("slime")
                .sprite(TextureHandle(3))
                .health(10)
                .drops("heart")
                .build()?;
            game.checkpoint_prefab("checkpoint")
                .sprite(TextureHandle(4))
                .build()?;

            game.map("recipes")
                .tile_size(16.0)
                .tiles(["#########", "#PHEK...#", "#########"])
                .simple_theme(TextureHandle(10), TextureHandle(11))
                .legend('P', "player")
                .legend('H', "heart")
                .legend('E', "slime")
                .legend('K', "checkpoint")
                .start();
            game.on_start(|game| game.spawn_start_map());

            game.rules()
                .player_collects_pickups()
                .enemy_drops()
                .win_when_all_pickups_collected()
                .win_when_all_enemies_dead()
                .player_activates_checkpoints()
                .respawn_at_checkpoint()
                .build();
            Ok(())
        }
    }

    #[test]
    fn recipe_rules_heal_drop_win_and_respawn_at_checkpoints() {
        let mut game = GameTestHarness::from_plugin(RecipeRulesPlugin).unwrap();
        game.assert_scene("game");

        let player = game.player();
        game.set_entity_health(player, 50);
        game.collect_first_pickup();
        game.assert_player_health(75);
        assert_eq!(game.count::<Pickup>(), 0);

        game.set_enemy_health(0, 0);
        game.step();
        assert_eq!(
            game.count::<Pickup>(),
            1,
            "a defeated enemy drops its prefab"
        );
        game.assert_scene("win");

        let checkpoint = game.world().ids_with::<Checkpoint>()[0];
        let checkpoint_position = game.world().get::<Transform>(checkpoint).unwrap().pos;
        let player = game.player();
        game.move_entity_to(player, checkpoint_position);
        game.step();
        assert_eq!(
            game.world()
                .get_resource::<CheckpointState>()
                .unwrap()
                .position,
            Some(checkpoint_position)
        );

        let player = game.player();
        game.set_entity_health(player, 0);
        game.step();
        game.step();
        game.assert_player_health(100);
        assert_eq!(game.player().position(), checkpoint_position);
    }

    struct DirectionalAttackPlugin;

    impl GamePlugin for DirectionalAttackPlugin {
        fn build(&self, game: &mut GameApp<'_>) -> anyhow::Result<()> {
            let controls = game.input(|input| input.top_down_controls())?;
            let sheet = SpriteSheet::new(TextureHandle(1), 4, 1);
            game.player_prefab("player")
                .spritesheet(sheet)
                .idle(0..1)
                .attack(0..1)
                .attack_up(0..1)
                .attack_down(1..2)
                .attack_left(2..3)
                .attack_right(3..4)
                .moves_with(controls.movement, 130.0)
                .build()?;
            game.map("directional-attack")
                .tiles(["###", "#P#", "###"])
                .simple_theme(TextureHandle(10), TextureHandle(11))
                .legend('P', "player")
                .start();
            game.rules()
                .top_down_controls(controls)
                .animate_attacks_directionally()
                .build();
            Ok(())
        }
    }

    #[test]
    fn directional_attack_rule_uses_remembered_facing_and_one_shot_clips() {
        let mut game = GameTestHarness::from_plugin(DirectionalAttackPlugin).unwrap();

        game.tap_action("attack");
        let player = game.player();
        assert_eq!(
            game.world().get::<Animation>(player.id()).unwrap().current,
            "attack_down"
        );

        *game
            .world_mut()
            .get_mut::<FacingDirection>(player.id())
            .unwrap() = FacingDirection::Left;
        game.tap_action("attack");
        assert_eq!(
            game.world().get::<Animation>(player.id()).unwrap().current,
            "attack_left"
        );
    }
}

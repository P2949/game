//! Beginner prefab builders.

mod area;
mod door;
mod enemy;
mod pickup;
mod player;
mod projectile;
mod shared;
mod spawner;

pub use area::AreaPrefabAuthor;
pub use door::DoorPrefabAuthor;
pub use enemy::EnemyPrefabAuthor;
pub use pickup::PickupPrefabAuthor;
pub use player::PlayerPrefabAuthor;
pub use projectile::ProjectilePrefabAuthor;
pub use spawner::SpawnerPrefabAuthor;

#[cfg(test)]
mod tests {
    use game_ai::Patrol;
    use game_core::backend::{AudioCommand, SoundHandle, TextureHandle};

    use super::shared::*;
    use crate::app::{GameApp, GamePlugin};
    use crate::harness::GameTestHarness;

    struct ObjectPrefabPlugin;

    impl GamePlugin for ObjectPrefabPlugin {
        fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
            game.pickup_prefab("coin")
                .sprite(TextureHandle(1))
                .score(1)
                .play_sound(SoundHandle(1))
                .despawn_on_collect()
                .build()?;

            game.door_prefab("exit")
                .sprite(TextureHandle(2))
                .change_map("level_2")
                .requires_all_enemies_dead()
                .build()?;

            game.projectile_prefab("fireball")
                .sprite(TextureHandle(3))
                .speed(320.0)
                .damage(10)
                .lifetime(2.0)
                .despawn_on_hit()
                .build()?;

            game.spawner_prefab("spawner")
                .spawn("coin")
                .every_seconds(3.0)
                .max_alive(5)
                .build()?;

            game.map("objects")
                .tiles(["#####", "#CDS#", "#F..#", "#####"])
                .simple_theme(TextureHandle(10), TextureHandle(11))
                .legend('C', "coin")
                .legend('D', "exit")
                .legend('F', "fireball")
                .legend('S', "spawner")
                .start();

            game.on_start(|game| game.spawn_start_map());
            Ok(())
        }
    }

    #[test]
    fn object_prefab_builders_spawn_expected_components() {
        let game = GameTestHarness::from_plugin(ObjectPrefabPlugin).unwrap();

        assert_eq!(game.count::<Pickup>(), 1);
        assert_eq!(game.count::<Collectible>(), 1);
        assert_eq!(game.count::<ScoreValue>(), 1);
        assert_eq!(game.count::<CollectSound>(), 1);
        assert_eq!(game.count::<DespawnOnCollect>(), 1);
        assert_eq!(game.count::<Door>(), 1);
        assert_eq!(game.count::<ExitDoor>(), 1);
        assert_eq!(game.count::<DoorTarget>(), 1);
        assert_eq!(game.count::<Projectile>(), 1);
        assert_eq!(game.count::<ProjectileDamage>(), 1);
        assert_eq!(game.count::<Lifetime>(), 1);
        assert_eq!(game.count::<DespawnOnHit>(), 1);
        assert_eq!(game.count::<Spawner>(), 1);
    }

    struct PatrolPrefabPlugin;

    impl GamePlugin for PatrolPrefabPlugin {
        fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
            game.enemy_prefab("patroller")
                .sprite(TextureHandle(1))
                .patrol_horizontal(64.0)
                .patrol_speed(40.0)
                .build()?;

            game.map("patrol")
                .tiles(["###", "#P#", "###"])
                .simple_theme(TextureHandle(10), TextureHandle(11))
                .legend('P', "patroller")
                .start();

            game.on_start(|game| game.spawn_start_map());
            Ok(())
        }
    }

    #[test]
    fn enemy_prefab_can_spawn_patrol_component() {
        let game = GameTestHarness::from_plugin(PatrolPrefabPlugin).unwrap();

        assert_eq!(game.count::<Enemy>(), 1);
        assert_eq!(game.count::<Patrol>(), 1);
    }

    struct NamedAssetPrefabPlugin;

    impl GamePlugin for NamedAssetPrefabPlugin {
        fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
            game.asset_bag()
                .texture("player", "textures/player.png")?
                .texture("slime", "textures/slime.png")?
                .texture("coin", "textures/coin.png")?
                .texture("door", "textures/door.png")?
                .texture("bolt", "textures/bolt.png")?
                .texture("floor", "textures/floor.png")?
                .texture("wall", "textures/wall.png")?
                .generated_sound("coin")?
                .build();
            let controls = game.input(|input| input.top_down_controls())?;

            game.player_prefab("player")
                .sprite("player")
                .moves_with(controls.movement, 120.0)
                .build()?;
            game.enemy_prefab("slime").sprite("slime").build()?;
            game.pickup_prefab("coin")
                .sprite("coin")
                .play_sound("coin")
                .build()?;
            game.door_prefab("door")
                .sprite("door")
                .restart_level()
                .build()?;
            game.projectile_prefab("bolt").sprite("bolt").build()?;

            game.map("named-assets")
                .tiles(["#####", "#P..#", "#####"])
                .simple_theme("floor", "wall")
                .legend('P', "player")
                .start();
            game.on_start(|game| game.spawn_start_map());
            game.on_action(controls.attack, |game| game.play_sound_named("coin"));
            Ok(())
        }
    }

    #[test]
    fn beginner_prefab_and_map_builders_resolve_named_assets() {
        let mut game = GameTestHarness::from_plugin(NamedAssetPrefabPlugin).unwrap();

        assert_eq!(game.count::<Player>(), 1);
        assert_eq!(game.count::<Sprite>(), 1);

        game = game.press_action("attack");
        game.fixed_step(1.0 / 120.0);
        assert_eq!(
            game.audio_commands(),
            &[AudioCommand::Play {
                sound: SoundHandle(0),
                volume: 1.0,
                looping: false,
                bus: None,
            }]
        );
    }

    struct MissingNamedAssetPlugin;

    impl GamePlugin for MissingNamedAssetPlugin {
        fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
            game.asset_bag()
                .texture("player", "textures/player.png")?
                .build();
            let controls = game.input(|input| input.top_down_controls())?;
            game.player_prefab("player")
                .sprite("plaeyr")
                .moves_with(controls.movement, 120.0)
                .build()
        }
    }

    #[test]
    fn named_prefab_assets_report_friendly_missing_key_diagnostics() {
        let error = match GameTestHarness::from_plugin(MissingNamedAssetPlugin) {
            Ok(_) => panic!("missing named asset should reject the prefab"),
            Err(error) => error,
        };
        let message = error.to_string();
        assert!(message.contains("Unknown texture asset 'plaeyr'"));
        assert!(message.contains("- player"));
        assert!(message.contains("Did you mean 'player'?"));
    }
}

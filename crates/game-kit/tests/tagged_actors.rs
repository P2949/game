use game_kit::advanced::prelude::*;
use game_kit::testing::GameTestHarness;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct BomberObservation {
    explosive_count: usize,
    at_explosion_range: usize,
    just_outside_range: usize,
    fuse_after_tick: f32,
    has_explosive_tag: bool,
    damaged_actors: usize,
}

struct TaggedActorsPlugin;

impl GamePlugin for TaggedActorsPlugin {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        let controls = game.input(|input| input.top_down_controls())?;

        game.player_prefab("player")
            .sprite(TextureHandle(1))
            .moves_with(controls.movement, 120.0)
            .health(100)
            .build()?;
        game.enemy_prefab("bomber")
            .sprite(TextureHandle(2))
            .tag("enemy")
            .tag("explosive")
            .data("fuse", 3.0)
            .health(100)
            .build()?;
        game.enemy_prefab("slime")
            .sprite(TextureHandle(3))
            .tag("enemy")
            .health(100)
            .build()?;

        game.map("tagged")
            .tile_size(32.0)
            .tiles(["#####", "#PBE#", "#####"])
            .simple_theme(TextureHandle(0), TextureHandle(0))
            .legend('P', "player")
            .legend('B', "bomber")
            .legend('E', "slime")
            .start();
        game.on_start(|game| game.spawn_start_map());

        game.fixed(|game: &mut GameCtx<'_, '_>, dt| {
            let mut explosions = Vec::new();
            let mut observation = BomberObservation {
                explosive_count: game.actors_tagged("explosive").count(),
                ..BomberObservation::default()
            };

            game.actors_tagged("explosive").each(|actor| {
                observation.has_explosive_tag = actor.has_tag("explosive");
                let fuse = actor.data("fuse").unwrap_or_default() - dt;
                actor.set_data("fuse", fuse);
                observation.fuse_after_tick = actor.data("fuse").unwrap_or_default();
                if fuse <= 0.0 {
                    explosions.push(actor.position());
                }
            });

            for position in explosions.into_iter().flatten() {
                observation.at_explosion_range =
                    game.actors_tagged("enemy").near(position, 32.0).count();
                observation.just_outside_range =
                    game.actors_tagged("enemy").near(position, 31.9).count();
                observation.damaged_actors +=
                    game.actors_tagged("enemy").near(position, 48.0).damage(20);
                game.player().damage_if_near(position, 48.0, 20);
            }

            game.insert_resource(observation);
        });

        Ok(())
    }
}

#[test]
fn tagged_actors_find_filter_mutate_and_damage_without_ecs_surface() {
    let mut game = GameTestHarness::from_plugin(TaggedActorsPlugin).unwrap();

    game.fixed_step(3.0);

    assert_eq!(
        game.world().get_resource::<BomberObservation>(),
        Some(&BomberObservation {
            explosive_count: 1,
            at_explosion_range: 2,
            just_outside_range: 1,
            fuse_after_tick: 0.0,
            has_explosive_tag: true,
            damaged_actors: 2,
        })
    );
    game.assert_player_health(80);
    assert_eq!(game.enemy(0).health(), 80);
    assert_eq!(game.enemy(1).health(), 80);
}

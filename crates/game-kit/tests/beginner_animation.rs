use game_kit::prelude::*;
use game_kit::testing::GameTestHarness;

const PLAYER: &str = "animation/player";

struct AnimationPlugin;

impl GamePlugin for AnimationPlugin {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        let sheet = game
            .assets(|assets| assets.spritesheet("animation/player", "textures/test.png", 4, 1))?;
        let movement = game.input(|input| Ok(input.axis2d("move")?.wasd().arrows()))?;

        game.player_prefab(PLAYER)
            .named("Player")
            .spritesheet(sheet)
            .animation("idle", AnimationClip::frames(0..2).fps(2.0).looping())
            .animation("walk", AnimationClip::frames(2..4).fps(4.0).looping())
            .play("idle")
            .moves_with(movement, 130.0)
            .build()?;

        game.map("animation")
            .tile_size(32.0)
            .tiles(["...", "...", "..."])
            .theme(TileTheme {
                floor: Sprite::new(sheet.texture, vec2s(32.0)).tint(vec4(0.1, 0.1, 0.1, 1.0)),
                wall: Sprite::new(sheet.texture, vec2s(32.0)).tint(vec4(0.4, 0.4, 0.4, 1.0)),
            })
            .spawn("player_start", PLAYER, cell(1, 1))
            .require_object("player_start")
            .start();

        game.use_top_down_game().movement(movement).build();

        Ok(())
    }
}

#[test]
fn spritesheet_prefab_spawns_animation_components() {
    let game = GameTestHarness::from_plugin(AnimationPlugin).unwrap();
    let player = game
        .world()
        .ids_with::<Player>()
        .into_iter()
        .next()
        .unwrap();

    assert_eq!(
        game.world().get::<Animation>(player).unwrap().current,
        "idle"
    );
    assert!(game.world().get::<AnimationSet>(player).is_some());
    assert_eq!(
        game.world().get::<Sprite>(player).unwrap().uv_max,
        vec2(0.25, 1.0)
    );
}

#[test]
fn top_down_preset_advances_sprite_animation() {
    let mut game = GameTestHarness::from_plugin(AnimationPlugin).unwrap();
    let player = game
        .world()
        .ids_with::<Player>()
        .into_iter()
        .next()
        .unwrap();

    game.frame(0.51);

    assert_eq!(game.world().get::<Animation>(player).unwrap().frame, 1);
    assert_eq!(
        game.world().get::<Sprite>(player).unwrap().uv_min,
        vec2(0.25, 0.0)
    );
}

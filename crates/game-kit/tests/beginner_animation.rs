use game_kit::advanced::prelude::*;
use game_kit::testing::GameTestHarness;

const PLAYER: &str = "animation/player";

struct AnimationPlugin;

impl GamePlugin for AnimationPlugin {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        let sheet = game
            .assets(|assets| assets.spritesheet("animation/player", "textures/test.png", 4, 1))?;
        let controls = game.input(|input| input.top_down_controls())?;

        game.player_prefab(PLAYER)
            .named("Player")
            .spritesheet(sheet)
            .idle(0..2)
            .walk(2..4)
            .attack(1..2)
            .moves_with(controls.movement, 130.0)
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

        game.use_top_down_game()
            .controls(controls)
            .with_melee_combat()
            .with_player_animation_by_movement()
            .with_attack_animation("attack")
            .build();

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
    let set = game.world().get::<AnimationSet>(player).unwrap();
    assert_eq!(set.get("idle").unwrap().fps, 6.0);
    assert!(set.get("idle").unwrap().looping);
    assert_eq!(set.get("walk").unwrap().fps, 10.0);
    assert!(set.get("walk").unwrap().looping);
    assert_eq!(set.get("attack").unwrap().fps, 12.0);
    assert!(!set.get("attack").unwrap().looping);
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

#[test]
fn top_down_preset_switches_animation_by_movement_and_attack() {
    let mut game = GameTestHarness::from_plugin(AnimationPlugin).unwrap();
    let player = game
        .world()
        .ids_with::<Player>()
        .into_iter()
        .next()
        .unwrap();

    game = game.set_axis("move", vec2(1.0, 0.0));
    game.frame(1.0 / 120.0);

    assert_eq!(
        game.world().get::<Animation>(player).unwrap().current,
        "walk"
    );

    game.clear_input();
    game.tap_action("attack");

    assert_eq!(
        game.world().get::<Animation>(player).unwrap().current,
        "attack"
    );

    game.frame(0.1);
    game.frame(0.0);

    assert_eq!(
        game.world().get::<Animation>(player).unwrap().current,
        "idle"
    );
}

struct DirectionalAndProjectilePlugin;

impl GamePlugin for DirectionalAndProjectilePlugin {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        let sheet =
            game.assets(|assets| assets.spritesheet("animation/sheet", "textures/test.png", 4, 1))?;
        let controls = game.input(|input| input.top_down_controls())?;

        game.player_prefab("player")
            .spritesheet(sheet)
            .idle(0..1)
            .walk(0..1)
            .walk_up(0..1)
            .walk_down(1..2)
            .walk_left(2..3)
            .walk_right(3..4)
            .moves_with(controls.movement, 120.0)
            .build()?;
        game.enemy_prefab("slime")
            .spritesheet(sheet)
            .idle(0..1)
            .walk(0..1)
            .walk_up(0..1)
            .walk_down(1..2)
            .walk_left(2..3)
            .walk_right(3..4)
            .build()?;
        game.projectile_prefab("bolt")
            .spritesheet(sheet)
            .flight(0..2)
            .impact(2..4)
            .speed(240.0)
            .damage(20)
            .lifetime(2.0)
            .despawn_on_hit()
            .build()?;

        game.map("directional")
            .tile_size(32.0)
            .tiles(["########", "#P.E...#", "########"])
            .simple_theme(sheet.texture, sheet.texture)
            .legend('P', "player")
            .legend('E', "slime")
            .start();

        game.rules()
            .top_down_controls(controls)
            .animate_player_directionally()
            .animate_enemies_directionally()
            .projectiles()
            .projectile_impact_animation_before_despawn()
            .build();
        game.on_action(controls.attack, |game| {
            game.player().shoot("bolt").right();
        });
        Ok(())
    }
}

#[test]
fn directional_walk_and_projectile_impact_helpers_drive_animation_state() {
    let mut game = GameTestHarness::from_plugin(DirectionalAndProjectilePlugin).unwrap();
    let player = game.world().ids_with::<Player>()[0];
    let enemy = game.world().ids_with::<Enemy>()[0];

    game = game.set_axis("move", vec2(1.0, 0.0));
    game.frame(1.0 / 120.0);
    assert_eq!(
        game.world().get::<Animation>(player).unwrap().current,
        "walk_right"
    );

    game.world_mut().get_mut::<Velocity>(enemy).unwrap().0 = vec2(0.0, -1.0);
    game.frame(1.0 / 120.0);
    assert_eq!(
        game.world().get::<Animation>(enemy).unwrap().current,
        "walk_up"
    );

    game.clear_input();
    game.tap_action("attack");
    let projectile = game.world().ids_with::<Projectile>()[0];
    assert_eq!(
        game.world().get::<Animation>(projectile).unwrap().current,
        "flight"
    );

    let enemy_position = game.world().get::<Transform>(enemy).unwrap().pos;
    game.world_mut()
        .get_mut::<Transform>(projectile)
        .unwrap()
        .pos = enemy_position;
    game.fixed_step(1.0 / 120.0);
    assert!(game.world().has::<ProjectileImpact>(projectile));
    assert_eq!(
        game.world().get::<Animation>(projectile).unwrap().current,
        "impact"
    );

    game.frame(0.25);
    game.fixed_step(1.0 / 120.0);
    assert!(game.world().ids_with::<Projectile>().is_empty());
}

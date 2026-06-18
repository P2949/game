use game_kit::beginner::prelude::*;

const PLAYER: &str = "simple/player";
const SLIME: &str = "simple/slime";

pub struct SimplePlugin;

pub fn plugin() -> game_kit::Plugin<SimplePlugin> {
    game_kit::plugin(SimplePlugin)
}

impl GamePlugin for SimplePlugin {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        let assets = game.assets(register_assets)?;
        let controls = game.input(|input| input.top_down_controls())?;

        game.player_prefab(PLAYER)
            .named("Player")
            .sprite(assets.player)
            .size(20.0)
            .tint(vec4(0.35, 0.70, 1.0, 1.0))
            .health(100)
            .moves_with(controls.movement, 130.0)
            .melee(30.0, 25)
            .build()?;

        game.enemy_prefab(SLIME)
            .named("Slime")
            .sprite(assets.slime)
            .size(22.0)
            .tint(vec4(1.0, 0.40, 0.35, 1.0))
            .health(40)
            .speed(80.0)
            .chases_player()
            .melee(26.0, 6)
            .build()?;

        game.map("simple")
            .tile_size(32.0)
            .tiles([
                "#########",
                "#.......#",
                "#.......#",
                "#.......#",
                "#########",
            ])
            .simple_theme(assets.floor, assets.wall)
            .spawn("player_start", PLAYER, cell(3, 2))
            .spawn("enemy_01", SLIME, cell(5, 2))
            .require_object("player_start")
            .start();

        game.use_top_down_game()
            .controls(controls)
            .hit_sound(assets.hit)
            .with_melee_combat()
            .with_enemy_chase()
            .with_collision()
            .with_camera_follow()
            .with_pause_death_ui()
            .build();

        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
struct SimpleAssets {
    floor: TextureHandle,
    wall: TextureHandle,
    player: TextureHandle,
    slime: TextureHandle,
    hit: SoundHandle,
}

fn register_assets(assets: &mut AssetAuthor<'_>) -> Result<SimpleAssets> {
    Ok(SimpleAssets {
        floor: assets.texture("simple/floor", "textures/test.png")?,
        wall: assets.texture("simple/wall", "textures/test.png")?,
        player: assets.texture("simple/player", "textures/test.png")?,
        slime: assets.texture("simple/slime", "textures/test.png")?,
        hit: assets.sound("simple/hit", "sounds/hit.wav")?,
    })
}

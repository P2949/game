pub mod assets;
pub mod level;

use game_kit::prelude::*;

const PLAYER: &str = "arena/player";
const SLIME: &str = "arena/slime";

pub struct ArenaPlugin;

pub fn plugin() -> game_kit::Plugin<ArenaPlugin> {
    game_kit::plugin(ArenaPlugin)
}

impl GamePlugin for ArenaPlugin {
    fn build(&self, game: &mut GameApp) -> anyhow::Result<()> {
        let assets = game.assets(crate::assets::register)?;
        let controls = game.input(|input| input.top_down_controls())?;

        game.player_prefab(PLAYER)
            .named("Player")
            .sprite(assets.player)
            .size(20.0)
            .tint(vec4(0.4, 0.7, 1.0, 1.0))
            .health(100)
            .moves_with(controls.movement, 130.0)
            .melee(30.0, 25)
            .build()?;

        game.enemy_prefab(SLIME)
            .named("Enemy")
            .sprite(assets.enemy)
            .size(22.0)
            .tint(vec4(1.0, 0.4, 0.4, 1.0))
            .health(40)
            .speed(80.0)
            .chases_player()
            .melee(26.0, 6)
            .build()?;

        crate::level::register(game, assets);

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

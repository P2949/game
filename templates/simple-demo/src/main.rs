use game_kit::beginner::prelude::*;
use game_runtime::RuntimeConfig;

#[derive(Clone, Copy)]
struct Assets {
    player: TextureHandle,
    slime: TextureHandle,
    floor: TextureHandle,
    wall: TextureHandle,
    hit: SoundHandle,
}

fn main() -> Result<()> {
    run_game("{{title}}", |game| {
        let assets = game.assets(|assets| {
            Ok(Assets {
                player: assets.texture("player", "textures/test.png")?,
                slime: assets.texture("slime", "textures/test.png")?,
                floor: assets.texture("floor", "textures/test.png")?,
                wall: assets.texture("wall", "textures/test.png")?,
                hit: assets.sound("hit", "sounds/hit.wav")?,
            })
        })?;

        let controls = game.input(|input| input.top_down_controls())?;

        game.player_prefab("player")
            .sprite(assets.player)
            .moves_with(controls.movement, 130.0)
            .health(100)
            .melee(30.0, 25)
            .build()?;

        game.enemy_prefab("slime")
            .sprite(assets.slime)
            .chases_player()
            .health(40)
            .melee(26.0, 6)
            .build()?;

        game.map("level_1")
            .tiles([
                "########",
                "#......#",
                "#..P.E.#",
                "#......#",
                "########",
            ])
            .simple_theme(assets.floor, assets.wall)
            .legend('P', "player")
            .legend('E', "slime")
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
    })
}

fn run_game<F>(title: &str, build: F) -> Result<()>
where
    F: for<'app> Fn(&mut GameApp<'app>) -> Result<()>,
{
    game_runtime::run(
        RuntimeConfig::default().title(title),
        game_kit::plugin_fn(build),
    )
}

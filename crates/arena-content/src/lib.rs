//! Beginner-style structured content crate.
//!
//! This crate uses the same high-level beginner gameplay APIs as
//! `simple-content`, but splits assets, maps, and plugin setup into separate
//! files and uses a typed asset struct. Beginners should start with
//! `simple-content` or `examples/one-file-demo`; this crate shows the next
//! organization step.

pub mod assets;
pub mod level;

use game_kit::beginner::prelude::*;

const PLAYER: &str = "arena/player";
const SLIME: &str = "arena/slime";

content_plugin!(ArenaPlugin, plugin, |game| {
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
});

# 08 - Add Sound And Music

## Goal

Play a short sound effect, loop music, fade it out, and keep audio named rather
than passing sound handles through your game code.

## Files to edit

Edit `src/main.rs`. Add `assets/sounds/coin.wav` and `assets/music/theme.wav`.

## Full code

```rust
use game_starter::prelude::*;

fn main() -> Result<()> {
    run_game("Sound And Music", |game| {
        game.asset_bag()
            .texture("floor", "textures/test.png")?
            .texture("wall", "textures/test.png")?
            .sound("coin", "sounds/coin.wav")?
            .music("theme", "music/theme.wav")?
            .build();

        let controls = game.input(|input| input.top_down_controls())?;

        game.map("audio")
            .tiles([".....", "....."])
            .simple_theme("floor", "wall")
            .start();

        game.rules().top_down_controls(controls).build();

        game.on_action(controls.attack, |game| {
            game.audio().play_sound("coin");
            game.audio().play_music("theme").volume(0.4).fade_in(1.0);
        });
        game.on_action(controls.reset, |game| {
            game.audio().fade_music_to(0.0, 1.0);
        });
        game.on_action(controls.pause, |game| {
            game.audio().pause_music();
        });
        game.on_action(controls.debug_overlay, |game| {
            game.audio().resume_music();
        });

        game.draw_ui(|game, _dt| {
            game.ui()
                .panel("Sound And Music")
                .line("Space: play sound and music")
                .line("R: fade | P: pause | F1: resume")
                .center();
        });

        Ok(())
    })
}
```

## What changed

The asset key is the friendly name used later: `"coin"` and `"theme"`.
Music loops until it is stopped or replaced. WAV is available by default. OGG
Vorbis also works after enabling the `ogg` feature on `game-starter` in your
`Cargo.toml`:

```toml
game-starter = { path = "../game/crates/game-starter", features = ["ogg"] }
```

## Common errors

Use paths relative to `assets/`, so write `music/theme.wav`, not
`assets/music/theme.wav`. If OGG reports that its feature is disabled, either
enable `ogg` as shown above or convert it with
`ffmpeg -i theme.ogg -ac 2 -ar 48000 assets/music/theme.wav`.

## Next step

Give the game a clickable menu in [09 - Add UI and menu](09-add-ui-and-menu.md).

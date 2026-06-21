# Audio

Copy [examples/audio-demo/src/main.rs](../../examples/audio-demo/src/main.rs)
for named sound effects, looping music, fades, bus volumes, and crossfades.

Register assets once, then use their friendly names anywhere in a gameplay
callback:

```rust
game.asset_bag()
    .sound("coin", "sounds/coin.wav")?
    .music("theme", "music/theme.ogg")?
    .build();

game.audio().bus("ambience").volume(0.5);
game.audio().play_sound("wind").bus("ambience");
game.audio().play_music("theme").volume(0.4).fade_in(1.0);
game.audio().fade_music_to(0.0, 1.0);
game.audio().set_master_volume(0.8);
game.audio().set_sfx_volume(0.8);
game.audio().set_music_volume(0.5);

game.on_scene_enter("game", |game| {
    game.audio().crossfade_music("battle", 1.0);
});
```

WAV files (mono/stereo PCM16 or float32) are the safest default. OGG Vorbis is
recommended for music and is optional so small builds do not need its decoder:
enable it through the runtime entry point, for example
`game-starter = { ..., features = ["ogg"] }`. MP3 is also optional:
`game-starter = { ..., features = ["mp3"] }`. That MP3 feature uses the
installed `ffmpeg` program while assets load, so use WAV or OGG when you want a
self-contained game binary.

If an optional feature is disabled, startup reports exactly how to enable it or
convert the file:

```bash
ffmpeg -i theme.ogg -ac 2 -ar 48000 assets/music/theme.wav
ffmpeg -i source.mp3 -ac 2 -ar 48000 assets/music/theme.ogg
ffmpeg -i source.wav -ac 2 -ar 48000 assets/sounds/hit.wav
```

Use 48 kHz mono or stereo exports when possible. The mixer can normalize normal
input sample rates. Music is currently loaded into memory and looped; streaming
long tracks is deliberately deferred, so prefer reasonably sized OGG files for
small games.

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

## Long music tracks

Small demos should keep using `.sound(...)` for effects and `.music(...)` for
ordinary WAV/OGG/MP3 music. Those files are decoded at startup, which keeps the
beginner path delightfully boring.

For a long looping track, opt into the bounded reader instead:

```rust
game.asset_bag()
    .streamed_music("long_theme", "music/long_theme.wav")?
    .build();

// Streaming is selected when the asset is registered; playback, fades, and
// crossfades use the same friendly name API.
game.audio().play_music("long_theme").volume(0.45).fade_in(1.0);
```

The current streamed path reads a **48 kHz stereo PCM16 WAV** on a background
worker into a bounded four-second buffer. It never decodes the whole track at
startup. Export it with:

```bash
ffmpeg -i long-theme.ogg -ac 2 -ar 48000 -c:a pcm_s16le assets/music/long_theme.wav
```

Use static `.music(...)` for OGG or MP3 today; its existing format diagnostics
will name a disabled feature and show the appropriate conversion command.

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
input sample rates. Press F5 in development to reload registered sound files:
existing voices using a replaced static sound stop, and later plays use the new
data. A streamed track is validated and restarted from the updated file.

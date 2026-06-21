# Audio

Copy [examples/audio-demo/src/main.rs](../../examples/audio-demo/src/main.rs)
for named sound effects, looping music, fades, and volume groups.

Register assets once, then use their friendly names anywhere in a gameplay
callback:

```rust
game.asset_bag()
    .sound("coin", "sounds/coin.wav")?
    .music("theme", "music/theme.ogg")?
    .build();

game.audio().play_sound("coin");
game.audio().play_music("theme").volume(0.4).fade_in(1.0);
game.audio().fade_music_to(0.0, 1.0);
game.audio().set_master_volume(0.8);
game.audio().set_sfx_volume(0.8);
game.audio().set_music_volume(0.5);
```

WAV files (mono/stereo PCM16 or float32) are supported by default. OGG Vorbis is
optional so small builds do not need its decoder: enable it through the runtime
entry point, for example `game-starter = { ..., features = ["ogg"] }`. If the
feature is disabled, OGG registration succeeds but startup reports exactly how
to enable it or convert the file:

```bash
ffmpeg -i theme.ogg -ac 2 -ar 48000 assets/music/theme.wav
```

Use 48 kHz mono or stereo exports when possible. The mixer can normalize normal
input sample rates, but streaming long music tracks is a future improvement;
today music is memory-loaded and looped.

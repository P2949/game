# Common Errors

## Goal

Fix beginner authoring mistakes by looking for the builder call that is missing.

## What you will build

Nothing new. This page is a checklist for when the demo fails to build, validate,
or start.

## Files you will edit

Usually `crates/simple-content/src/game.rs`.

## Final code

A healthy beginner demo has this shape:

```rust
use game_kit::beginner::prelude::*;

game.assets_from_folders()
    .texture("player")?
    .texture("slime")?
    .texture("floor")?
    .texture("wall")?
    .sound("hit")?
    .build();
let controls = game.input(|input| input.top_down_controls())?;

game.player_prefab("player")
    .sprite("player")
    .moves_with(controls.movement, 130.0)
    .build()?;

game.enemy_prefab("slime")
    .sprite("slime")
    .chases_player()
    .melee(26.0, 6)
    .build()?;

game.map("level_1")
    .tiles(["#####", "#P.E#", "#####"])
    .simple_theme("floor", "wall")
    .legend('P', "player")
    .legend('E', "slime")
    .start();

game.rules()
    .top_down_controls(controls)
    .enemies_damage_player()
    .dead_enemies_despawn()
    .camera_follows_player()
    .build();
```

## Explanation

Most validation errors are intentional teaching messages. They usually name the
prefab, map, asset, or input action and then show the call to add.

Read the first error from the top, fix that one, and run again. Later errors can
disappear once the first missing builder call is restored.

## Common errors

`player prefab 'player' has no sprite`

Add:

```rust
.sprite("player")
```

`player prefab 'player' has no movement axis`

Add:

```rust
.moves_with(controls.movement, 130.0)
```

`Map 'level_1' has no tile theme`

Add:

```rust
.simple_theme("floor", "wall")
```

`references unknown prefab`

Make the map spawn name match the prefab name:

```rust
game.player_prefab("player")
.spawn("player_start", "player", cell(1, 1))
```

`multiple start maps`

Only one map should call `.start()`. Other maps should call `.finish()`.

`asset validation failed`

Use paths relative to `assets/`:

```rust
.texture("player", "textures/test.png")?
.sound("hit", "sounds/hit.wav")?
```

For the standard folders, avoid repeating those paths. PNG is the current
texture convention; audio chooses `hit.wav` first and then `hit.ogg` when that
is the file present:

```rust
game.assets_from_folders()
    .texture("player")?
    .sound("hit")?
    .build();
```

`Map 'level_1' uses symbol 'X' but no legend was registered`

Add a legend for every non-`.`/`#` character in a text map:

```rust
.legend('X', "some_prefab")
```

`Map rows have inconsistent widths`

Every row in a text map must have the same number of symbols. The diagnostic
names the first row and the mismatched row; add floor `.` cells or remove extras
until their widths match.

Text maps live under `assets/` and use the same symbolic legend format:

```rust
game.map_from_text("level_1", "maps/level_1.txt")
    .simple_theme("floor", "wall")
    .legend('P', "player")
    .start();
```

`Unknown texture asset 'plaeyr'`

You probably wrote:

```rust
.sprite("plaeyr")
```

Registered textures include `player`, `slime`, `floor`, and `wall`. Fix the
key or register it before using the prefab:

```rust
game.asset_bag().texture_auto("player")?.build();
```

`Map 'level_1' has no player spawn`

Put a `P` in the map and connect it to the player prefab:

```rust
.legend('P', "player")
```

Then put a `P` in the text-map file itself.

`Sound file '...' uses unsupported format`

WAV accepts mono or stereo PCM16/float32 audio and is always available. OGG
Vorbis also works when the runtime package enables its `ogg` feature. Both are
converted to 48 kHz automatically. If the error says `OGG audio requires the
ogg feature`, add this to the game package that depends on `game-starter`:

```toml
game-starter = { path = "../game/crates/game-starter", features = ["ogg"] }
```

For unusual files, convert them first:

```bash
ffmpeg -i input.wav -ac 2 -ar 48000 assets/sounds/hit.wav
```

Named playback stays inside your gameplay callback:

```rust
game.audio().play_sound("hit");
game.audio().play_music("theme").volume(0.4);
```

Use `set_master_volume`, `set_sfx_volume`, and `set_music_volume` for global
mix levels. `fade_music_to`, `pause_music`, and `resume_music` control the
current music track without touching raw audio handles.

## Controller input

`input.top_down_controls()` supports the first connected controller as well as
keyboard and mouse: use its left stick to move, the south face button to attack,
Start to pause, and Select to reset. No SDL-specific code is needed in a game.

For a custom controller-only binding, use the same input builder:

```rust
let move_axis = game.input(|input| input.axis2d("move")?.gamepad_left_stick())?;
let attack = game.input(|input| input.action("attack")?.gamepad_south())?;
```

`No controller detected`

Keyboard and mouse keep working. Connect a controller before starting the game;
`top_down_controls()` uses the first detected controller automatically.

`Projectile does not move`

Enable the projectile behavior after defining the prefab:

```rust
game.rules().projectiles().build();
```

`Spawner does not spawn`

Enable the spawner behavior:

```rust
game.rules().spawners_spawn_prefabs().build();
```

`Door does not work`

Enable map-changing door behavior:

```rust
game.rules().doors_change_maps().build();
```

## Rust compiler errors from advanced code

**Populate after the first round of outside playtesting.** This guide will add
two or three real compiler diagnostics beside plain-English translations once
new players have shown which errors they actually encounter—most likely a
borrow-checker error after copying advanced code, or a missing `?` on a
fallible builder call.

We intentionally do not invent raw `rustc` output here. Compiler wording and
the mistakes beginners actually make are both evidence-sensitive; until that
playtest data exists, the tailored compiler messages for common sprite, name,
and movement-type mistakes are the trustworthy guidance.

## Next step

Return to the tutorial you were following and rerun the demo after each small
change.

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

let assets = game
    .asset_bag()
    .texture("player", "textures/test.png")?
    .texture("slime", "textures/test.png")?
    .texture("floor", "textures/test.png")?
    .texture("wall", "textures/test.png")?
    .sound("hit", "sounds/hit.wav")?
    .build();
let controls = game.input(|input| input.top_down_controls())?;

game.player_prefab("player")
    .sprite(assets.texture("player"))
    .moves_with(controls.movement, 130.0)
    .build()?;

game.enemy_prefab("slime")
    .sprite(assets.texture("slime"))
    .chases_player()
    .melee(26.0, 6)
    .build()?;

game.map("level_1")
    .tiles(["#####", "#P.E#", "#####"])
    .simple_theme(assets.texture("floor"), assets.texture("wall"))
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
.sprite(assets.texture("player"))
```

`player prefab 'player' has no movement axis`

Add:

```rust
.moves_with(controls.movement, 130.0)
```

`Map 'level_1' has no tile theme`

Add:

```rust
.simple_theme(assets.texture("floor"), assets.texture("wall"))
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

`Sound file '...' uses unsupported format`

Use a WAV file with mono or stereo audio. The runtime accepts PCM16 and float32
WAV data and converts normal sample rates to 48 kHz automatically. For unusual
files, convert them first:

```bash
ffmpeg -i input.wav -ac 2 -ar 48000 assets/sounds/hit.wav
```

## Next step

Return to the tutorial you were following and rerun the demo after each small
change.

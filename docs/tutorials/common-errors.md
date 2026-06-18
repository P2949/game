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
let assets = game.assets(register_assets)?;
let controls = game.input(|input| input.top_down_controls())?;

game.player_prefab("player")
    .sprite(assets.player)
    .moves_with(controls.movement, 130.0)
    .build()?;

game.map("level_1")
    .tiles(["###", "#.#", "###"])
    .simple_theme(assets.floor, assets.wall)
    .spawn("player_start", "player", cell(1, 1))
    .start();

game.use_top_down_game()
    .movement(controls.movement)
    .attack(controls.attack)
    .with_melee_combat()
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
.sprite(assets.player)
```

`player prefab 'player' has no movement axis`

Add:

```rust
.moves_with(controls.movement, 130.0)
```

`Map 'level_1' has no tile theme`

Add:

```rust
.simple_theme(assets.floor, assets.wall)
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
assets.texture("simple/player", "textures/test.png")?
assets.sound("simple/hit", "sounds/hit.wav")?
```

## Next step

Return to the tutorial you were following and rerun the demo after each small
change.

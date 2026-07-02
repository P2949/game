# {{title}}

This is a legacy/transitional RON data-driven Rust-wrapper template. It remains
useful for existing projects and migration, but the primary no-Rust authoring
surface is a `game.toml` package run by the prebuilt player.

## Legacy Workflow

1. Run `cargo run` or `game-dev run`.
2. Edit legacy `assets/game.ron` to change player/enemy/pickup numbers and
   rules.
3. Edit `assets/maps/level_1.txt` to change the level.
4. Replace files in `assets/textures/` and `assets/sounds/` with your own art
   and sound effects.

The editable legacy RON file is intentionally small:

- `assets.textures`, `sounds`, and `music` can register conventional asset names.
- `controls: TopDown` selects the standard beginner controls.
- `prefabs` define players, enemies, pickups, and other supported objects.
- `maps` connects a text map and its `P`/`E`/`C` legend to prefabs.
- `rules` selects common first-game behaviors with names like
  `TopDownControls`, `PlayerCollectsPickups`, and `ShowScore`.

For larger legacy RON examples, copy `examples/data-driven-events-demo`,
`examples/data-driven-waves-demo`, or `examples/data-driven-projectiles-demo`.
To move one of these projects to the primary no-Rust package shape, run
`game-dev migrate-ron assets/game.ron --out game.toml`, then check the result
with `game-dev check`.

The first build makes small starter assets if they do not already exist:

```text
assets/textures/player.png -> assets.textures: ["player"]
assets/textures/slime.png  -> assets.textures: ["slime"]
assets/textures/coin.png   -> assets.textures: ["coin"]
assets/textures/floor.png  -> theme: ("floor", "wall")
assets/textures/wall.png   -> theme: ("floor", "wall")
assets/textures/door.png   -> assets.textures: ["door"]
assets/textures/bolt.png   -> assets.textures: ["bolt"]
assets/sounds/hit.wav      -> assets.sounds: ["hit"]
assets/sounds/coin.wav     -> assets.sounds: ["coin"]
assets/sounds/shoot.wav    -> assets.sounds: ["shoot"]
```

The map symbols are:

- `#` wall
- `.` floor
- `P` player start (use one)
- `E` enemy
- `C` coin

Press <kbd>F5</kbd> in a debug build after changing the map or existing values
in legacy `assets/game.ron`. F5 validates and partially reloads the data file,
then respawns the current map. Future spawns from beginner rules use the
updated prefab values too, and existing custom countdown rule details, scene
text, and audio scene settings reload. Existing action settings reload when
their input binding stays the same. Adding, removing, or reordering asset,
prefab, map, or custom rule names requires a restart, as do changes to scene
names, adding/removing/reordering actions, action input bindings, or the
enabled rule list.

## Project tools

Install the beginner helper once:

```bash
cargo install --git https://github.com/P2949/game game-cli
```

Useful commands:

```bash
game-dev doctor
game-dev check
game-dev run
game-dev asset-check
game-dev validate-data
game-dev package --release --out dist/my-game --zip
```

## Need a fresh copy?

From anywhere, generate the template with:

```bash
cargo install cargo-generate
cargo generate --git https://github.com/P2949/game templates/data-driven-demo --name my-game
```

From a local checkout, run:

```bash
cargo xtask new-demo my-game --template data-driven
```

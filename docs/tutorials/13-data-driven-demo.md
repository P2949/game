# Data-driven first game

## Goal

Make a small playable demo by editing `assets/game.ron`, your text map, and
your art instead of writing the usual setup code.

## Start a data-driven demo

From a local checkout:

```bash
cargo xtask new-demo my-game --template data-driven
cd my-game
cargo run
```

The generated `src/main.rs` is deliberately tiny:

```rust
run_game("My Game", |game| {
    let _controls = game.load_beginner_file("game.ron")?;
    Ok(())
})
```

## Files to edit

- `assets/game.ron`: asset names, actors, maps, and standard rules
- `assets/maps/level_1.txt`: walls and `P`/`E`/`C` spawns
- `assets/textures/*.png`: your art

The first section of `game.ron` registers conventional files. For example,
`"player"` means `assets/textures/player.png`; `"hit"` means
`assets/sounds/hit.wav`.

```ron
assets: (
    textures: ["player", "slime", "coin", "floor", "wall"],
    sounds: ["hit"],
),
```

`prefabs` defines the object kinds, while `maps` connects each map symbol to
one of those names. The standard `rules` list supplies movement, combat,
camera, pickup, and UI behavior.

Use structured names in new files:

```ron
controls: TopDown,
rules: [
    TopDownControls,
    PlayerCollectsPickups,
    EnemiesDamagePlayer,
    CameraFollowsPlayer,
    ShowBasicUi,
]
```

Old string rules such as `"top_down_controls"` still load for compatibility,
but new examples use the structured names because they can grow to cover
projectiles, spawners, drops, checkpoints, scene flow, UI, and win rules.

For a larger no-Rust reference, see `examples/data-driven-full-demo`. Its
`assets/game.ron` includes doors, projectiles, spawners, checkpoints, music,
player shooting, enemy drops, and a countdown custom rule.

## Add small Rust behavior later

The loader returns the standard controls, so the data file does not trap you in
a dead end. Add a small custom rule after loading it:

```rust
run_game("My Game", |game| {
    let controls = game.load_beginner_file("game.ron")?;

    game.on_action(controls.attack, |game| {
        game.play_sound_named("hit");
    });

    Ok(())
})
```

This is the hybrid path: keep ordinary content in RON, then use Rust only for
the rules that make your game unusual.

## Helpful errors

The loader checks cross-references before it builds the game. A legend typo
such as `"slimee"` names the map symbol and suggests `"slime"`; an unknown
sprite or pickup sound lists the registered asset names. Fix the file named in
the message, then run again.

`game.ron` is read at startup. F5 reloads text maps and reloadable assets, not
the RON setup itself, so restart after changing prefabs, rules, or the map
list.

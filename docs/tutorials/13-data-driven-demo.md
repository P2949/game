# Legacy Data-Driven First Game

## Goal

Make a small playable demo by editing legacy `assets/game.ron`, your text map,
and your art instead of writing the usual setup code. This is the transitional
RON path; the primary no-Rust target is `game.toml` plus a prebuilt player.

## Start a Legacy Data-Driven Demo

From anywhere:

```bash
cargo install cargo-generate
cargo generate --git https://github.com/P2949/game templates/data-driven-demo --name my-game
cd my-game
cargo install --git https://github.com/P2949/game game-cli
game-dev validate-data
game-dev run
```

From a local checkout, `cargo xtask new-demo my-game --template data-driven`
creates the same project with a local path dependency.

The generated `src/main.rs` is deliberately tiny:

```rust
run_game("My Game", |game| {
    let _controls = game.load_beginner_file("game.ron")?;
    Ok(())
})
```

## Files to edit

- legacy `assets/game.ron`: asset names, actors, maps, and standard rules
- `assets/maps/level_1.txt`: walls and `P`/`E`/`C` spawns
- `assets/textures/*.png`: your art
- `assets/sounds/*.wav`: your sound effects

The first section of legacy `game.ron` registers conventional files. For
example, `"player"` means `assets/textures/player.png`; `"hit"` means
`assets/sounds/hit.wav`.

```ron
assets: (
    textures: ["player", "slime", "coin", "floor", "wall", "door", "bolt"],
    sounds: ["hit", "coin", "shoot"],
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

For a larger legacy RON reference, see `examples/data-driven-full-demo`. Its
`assets/game.ron` includes doors, projectiles, spawners, checkpoints, music,
player shooting, enemy drops, and a countdown custom rule.

For visual TMX maps through the transitional RON wrapper, use:

```bash
GAME_ASSET_DIR=examples/data-driven-tiled-demo/assets cargo run -p data-driven-tiled-demo --locked
```

For the equivalent beginner Rust API, use Tiled Rust:

```bash
GAME_ASSET_DIR=examples/tiled-demo/assets cargo run -p tiled-demo --locked
```

Read the [Tiled cookbook](../cookbook/tiled.md) for both paths.

For smaller focused references, copy:

- `examples/data-driven-events-demo` for `When` conditions, score gates, and
  scene changes
- `examples/data-driven-waves-demo` for timed spawns and tag-count rules
- `examples/data-driven-projectiles-demo` for player shooting, projectile
  rules, and enemy-death effects
- `examples/data-driven-tiled-demo` for legacy Tiled TMX object mapping through
  `assets/game.ron`

## Script rules

Use script rules inside the `rules` list when you want small reactions without
writing Rust:

```ron
When(condition: AllEnemiesDead, effects: [ChangeScene("win")])
When(condition: ScoreAtLeast(10), effects: [ShowUiText("Gate open")])
EverySeconds(seconds: 5.0, effects: [SpawnNearPlayer(prefab: "slime", radius: 128.0)])
```

Supported conditions are:

- `AllEnemiesDead`
- `AllPickupsCollected`
- `ScoreAtLeast(10)`
- `PlayerHealthBelow(3)`
- `TimerReached(name: "first_wave", seconds: 2.0)`
- `MapIs("level_1")`
- `SceneIs("win")`
- `TagCountZero("enemy")`
- `ActionPressed(Attack)`

Supported game-level effects are:

- `PlaySound("hit")`
- `PlayMusic("theme")`
- `StopMusic`
- `AddScore(1)`
- `SetScore(0)`
- `SpawnPrefab("coin")`
- `SpawnNearPlayer(prefab: "slime", radius: 128.0)`
- `ChangeMap("level_2")`
- `ChangeScene("win")`
- `RestartCurrentMap`
- `ShowUiText("Wave incoming")`
- `DamagePlayer(amount: 1)`
- `HealPlayer(1)`
- `SetData(tag: "enemy", key: "fuse", value: 3.0)`
- `DespawnTagged("hazard")`

`OnEnemyDeath(prefab: "slime", effects: [...])` supports the event-shaped
effects: score changes, `DespawnSelf`, sounds, prefab spawns, and map or scene
changes.

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

This is the hybrid path: keep ordinary content in legacy RON, then use Rust
only for the rules that make your game unusual.

## Helpful errors

The loader checks cross-references before it builds the game. A legend typo
such as `"slimee"` names the map symbol and suggests `"slime"`; an unknown
sprite or pickup sound lists the registered asset names. Fix the file named in
the message, then run again. You can also check the file without starting the
renderer:

```bash
game-dev validate-data assets/game.ron
```

In a debug build, F5 reparses legacy `assets/game.ron`, validates it, reloads
the current map data, respawns the current map, and reloads existing textures
and sounds. This is a partial data reload: changing existing numbers, prefab
settings, and a map's text-file path is supported, including future spawns from
beginner rules. Existing custom countdown rule details, scene text/buttons, and
audio scene settings also reload. Existing action settings such as prefab,
cooldown, direction, and sound reload when the input binding stays the same.
Adding, removing, or reordering asset names, prefab names, map names, or custom
rule names still requires a restart. Changing scene names, scene input
bindings, adding/removing/reordering actions, action input bindings, or the
enabled rule list also still requires a restart.
Because this legacy tutorial loads `game.ron`, the F1 debug overlay reports
`game.ron reload: partial` and shows the latest error if validation fails.
Primary `game.toml` packages report `game.toml` in the same overlay slot.

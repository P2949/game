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

`game-dev doctor` reports a failed setup check

Run the longer explanation:

```bash
game-dev doctor --explain
```

The doctor checks Rust/Cargo, shader compiler, Vulkan loader and Vulkan 1.3
device, SDL3, audio development files, validation layers, and the generated
project's `assets/` folder. Fix the first failed setup check before changing
game code.

`game-dev check` fails

Run:

```bash
game-dev doctor --explain
game-dev asset-check
game-dev validate-data assets/game.ron
```

`game-dev check` is a quick generated-project check: it runs the setup doctor,
validates assets, validates `assets/game.ron` when the file exists, and then
runs `cargo check`. If the final compiler error mentions SDL3, Vulkan, or
`glslc`, fix the first setup problem reported by the doctor before changing
game code.

`game-dev doctor` is advisory: it prints setup guidance and returns success
unless the command itself crashes. `game-dev check` is the hard gate. It fails
when the current directory cannot be read, `assets/` is missing, assets contain
invalid or unknown files, optional `assets/game.ron` is invalid, or
`cargo check` fails. Vulkan, audio, validation-layer, and optional-tool problems
are setup warnings for this command unless the requested cargo check needs them.

`game-dev run` or `game-dev package` fails

The helper prints this checklist:

```text
If this looks like a setup issue:
    game-dev doctor --explain

If this looks like an asset/data issue:
    game-dev asset-check
    game-dev validate-data assets/game.ron

See:
    docs/tutorials/common-errors.md
```

Run the setup command first for SDL3, Vulkan, shader compiler, or audio
prerequisite errors. Run the asset/data commands first for missing files,
unknown prefab names, bad map symbols, or `assets/game.ron` validation errors.

SDL3 fails while building from source

Normal local source builds use the system SDL3 development libraries checked by
`game-dev doctor --explain`. CI and generated-template checks can opt into the
source-built SDL3 path with `ci-build-sdl3`, which is useful for reproducible
automation but needs build tools such as CMake and Ninja.

From a generated project:

```bash
cargo check --features ci-build-sdl3
game-dev check --features ci-build-sdl3
```

If the source-build path fails before your game code compiles, check
CMake/Ninja/build dependency errors first.

Cargo cannot fetch or find `game-starter`

Generated projects depend on `game-starter` through a pinned Git dependency.
That pin is intentional: it keeps a project reproducible instead of following a
moving branch. If Cargo cannot fetch it, check the network error first, then
retry the dependency update:

```bash
cargo update -p game-starter
```

Release-candidate templates may use a `git rev` pin. Tagged releases should use
a `git tag` pin. If an old generated project is stuck on an earlier release tag
or revision, regenerate from the current template or update the dependency line
deliberately:

```bash
cargo generate --git https://github.com/P2949/game templates/simple-demo --name my-game
```

When you are developing against a local checkout of this repository, prefer the
xtask helper so the generated demo uses a local path dependency instead of a Git
pin:

```bash
cargo xtask new-demo /tmp/my-local-demo
```

F5 says restart required

F5 can reload existing values and files attached to existing names: edited text
maps, tuning values, registered textures and sounds, and supported existing
`assets/game.ron` entries. Adding, removing, or reordering assets, prefabs,
maps, actions, scenes, rules, or custom rule names changes the startup shape of
the game and requires a restart. This is intentional for 1.0.

See [Fast iteration](12-fast-iteration.md) for the full reload table.

Rule `projectiles_damage_enemies` needs the `projectiles` rule

Add the projectile rules preset before the damage rule:

```rust
game.rules()
    .projectiles()
    .projectiles_damage_enemies()
    .build();
```

If the data-file error says projectile rules need a `Projectile` prefab, also
define a projectile prefab such as `"bolt"`.

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

Prefab names are exact and case-sensitive. If a diagnostic says
`Did you mean 'player'?`, fix the typo in the map legend, Tiled object mapping,
door/spawner target, or data-file object mapping instead of adding another
nearly identical prefab.

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

`unknown asset file 'assets/textures/player.pgn'`

`game-dev asset-check`, `game-dev check`, and `game-dev package` fail on files
with unknown runtime extensions so typos do not get silently packaged. Fix the
extension or move notes/source files outside `assets/`. For common typos the
message suggests the likely extension:

```text
Did you mean '.png'?
```

`Missing texture asset 'player'`

The folder helper looked for the conventional file, usually:

```text
assets/textures/player.png
```

Either add that file or register the path explicitly:

```rust
game.asset_bag()
    .texture("player", "textures/my_player.png")?
    .build();
```

For self-contained Tiled examples, these can also mean the demo is running from
the repository root while looking at the wrong `assets/` folder:

```text
Missing texture asset 'player'
Missing texture asset 'slime'
Missing texture asset 'floor'
Missing texture asset 'wall'
```

Point `GAME_ASSET_DIR` at that example's assets folder:

```bash
GAME_ASSET_DIR=examples/tiled-demo/assets cargo run -p tiled-demo
GAME_ASSET_DIR=examples/data-driven-tiled-demo/assets cargo run -p data-driven-tiled-demo
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

`unknown map 'levle_2'`

Map transition names are exact. Check the string passed to `.change_map(...)`,
`.go_to_map(...)`, a door prefab, or `assets/game.ron` against the names passed
to `game.map("level_2")` / `game.map_from_text("level_2", ...)`. Fix the typo
at the caller; do not try to switch maps with raw map IDs.

Tiled object maps to the wrong prefab

If the Tiled object has class, type, or name `"Slime"` but Rust or `game.ron`
maps it to `"slmie"`, the importer cannot spawn the prefab you meant. Fix
either the prefab name or the object mapping so both sides use the same key.

Rust Tiled mapping:

```rust
.object("Slime", "slime")
```

No-Rust `game.ron` mapping:

```ron
objects: {"Slime": "slime"}
```

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

Too many sounds play in one frame

The audio mixer has a voice cap. If code starts a sound every frame or several
rules all spam the same sound at once, new play requests can be dropped and the
runtime logs a warning with the dropped count and cap. Gate sounds on events
such as collect/hit/death, add a short cooldown, or use looping music for
continuous audio.

## Data-file errors

New `assets/game.ron` files use structured, case-sensitive names:

```ron
controls: TopDown,
rules: [TopDownControls, PlayerCollectsPickups, ShowBasicUi],
```

Old strings such as `"top_down"` and `"show_score"` still work, but the current
templates use structured names. If a data file says an asset, prefab, map,
scene, sound, music track, tag, or rule is unknown, fix the spelling in
`game.ron`; the message lists known values and suggests close matches.

`references unknown music 'theem'. Known music: theme. Did you mean 'theme'?`

Use the same key in `audio.music_on_scene` that you listed in
`assets.music`:

```ron
assets: (music: ["theme"]),
audio: (music_on_scene: {"level_1": (track: "theme")}),
```

`unknown action 'Attak'. Known actions: Attack, Pause, Reset, Reload, MenuAccept`

Action names in `game.ron` are case-sensitive. Use one of the standard
top-down actions:

```ron
start_on: Some(Attack),
actions: [PlayerShoots((prefab: "bolt", action: Attack))],
```

`custom rule 'explode' counts down key 'fues' on tag 'explosive', but no prefab with that tag declares that data key`

Countdown rules read numeric `data` from prefabs with the selected tag. This
usually means the key is misspelled in either the prefab or the rule.

Add the key to a tagged prefab:

```ron
Trigger((name: "bomb", tags: ["explosive"], data: {"fuse": 3.0}))
```

or change the countdown rule to use the existing key:

```ron
Countdown((name: "explode", tag: "explosive", key: "fuse", when_zero: [DespawnSelf]))
```

`beginner game file 'game.ron' is not valid RON`

Check enum spelling and commas first. Names are case-sensitive:

```ron
controls: TopDown,
rules: [TopDownControls, Projectiles, ShowScore],
```

`Rule ProjectilesDamageEnemies needs the Projectiles rule`

Use the combined rule unless you are intentionally composing projectile pieces:

```ron
rules: [TopDownControls, Projectiles]
```

In Rust, the matching beginner call is:

```rust
game.rules().projectiles().build();
```

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

## Advanced Path: Rust Compiler Errors From Advanced Code

`Query`, `ResMut`, `Transform`, or `Collider` is not found

Beginner tutorials should import:

```rust
use game_kit::beginner::prelude::*;
```

Standalone generated games usually import:

```rust
use game_starter::prelude::*;
```

Use `game_kit::advanced::prelude::*` only when you intentionally need custom
tuple prefabs, typed query parameters, raw ECS/world inspection in tests, or an
engine-facing experiment. For normal player/enemy/pickup/door behavior, stay on
beginner prefabs, rules, hooks, and `assets/game.ron`.

`the ? operator can only be used`

The closure passed to `run_game` must return `Result<()>`, and fallible builder
calls need `?`:

```rust
run_game("My Game", |game| {
    game.asset_bag().texture("player", "textures/player.png")?.build();
    Ok(())
})
```

`expected Result<(), Error>, found ()`

Add the final success line:

```rust
Ok(())
```

`this is not a texture reference`

`.sprite(...)` expects a registered texture name:

```rust
.sprite("player")
```

`this is not a movement axis`

Use the movement axis from top-down controls:

```rust
let controls = game.input(|input| input.top_down_controls())?;
.moves_with(controls.movement, 130.0)
```

`cannot borrow ... as mutable`

In custom callbacks, read values before starting a chain that mutably borrows
the event or actor:

```rust
game.on_projectile_hit("bolt", "slime", |event| {
    let position = event.position();
    event.spawn("spark").at_world(position);
});
```

Avoid copying advanced examples that mention raw world access. Beginner custom
behavior should use handles such as `event.enemy()`, `game.enemies()`, and
`game.actors_tagged("tag")`.

## Setup and Runtime Errors

`glslc` is missing

Install the Vulkan shader compiler package for your OS, then rerun:

```bash
game-dev doctor --explain
```

`SDL3` or `Vulkan` setup failed

Run:

```bash
game-dev doctor --explain
```

Fix the first failed check. The doctor explains missing SDL3 libraries, Vulkan
loader/device issues, validation layers, and asset-folder problems.

## Next step

Return to the tutorial you were following and rerun the demo after each small
change.

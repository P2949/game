# Content Authoring

Content crates use `game-kit` as their public engine surface. New demos should
start on the beginner path and only drop to the advanced path when they need
custom ECS-style prefabs, systems, or tests.

Current examples are split this way:

- `simple-content`: pure beginner example.
- `arena-content`: beginner-style playable demo.
- `testbed-content`: advanced testbed showing manual systems, RON maps, and
  lower-level `game-kit` APIs.

For focused beginner recipes, see the [cookbook](cookbook/README.md).

Read the project in this order:

1. [Beginner authoring](beginner-authoring.md) and the tutorials for a
   script-like game.
2. This document when a project grows into a content crate.
3. [Advanced content authoring](advanced-content-authoring.md) only when custom
   low-level behavior is intentional.

`testbed-content` is intentionally advanced: it exercises manual systems, RON
maps, patrol setup, and lower-level facade APIs. Beginners should copy
`simple-content`, `arena-content`, `examples/one-file-demo`, or
`templates/simple-demo` instead.

## Beginner Path

Beginner content imports the beginner facade:

```rust
use game_kit::beginner::prelude::*;
```

The first useful plugin can live in one file:

```rust
pub struct DemoPlugin;

impl GamePlugin for DemoPlugin {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        game.asset_bag()
            .texture("player", "textures/test.png")?
            .texture("slime", "textures/test.png")?
            .texture("floor", "textures/test.png")?
            .texture("wall", "textures/test.png")?
            .sound("hit", "sounds/hit.wav")?
            .build();

        let controls = game.input(|input| input.top_down_controls())?;

        game.player_prefab("player")
            .sprite("player")
            .moves_with(controls.movement, 130.0)
            .build()?;

        game.enemy_prefab("slime")
            .sprite("slime")
            .chases_player()
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
            .camera_follows_player()
            .build();

        Ok(())
    }
}
```

Use `game.asset_bag()` for first projects. It registers assets under friendly
names, which the beginner builders use directly:

```rust
let assets = game
    .asset_bag()
    .texture("player", "textures/test.png")?
    .spritesheet("hero", "textures/test.png", 4, 1)?
    .sound("hit", "sounds/hit.wav")?
    .music("theme", "music/theme.wav")?
    .build();
```

### Conventional asset folders

When filenames match their gameplay names, avoid repeating paths entirely:

```rust
game.assets_from_folders()
    .texture("player")?
    .texture("slime")?
    .texture("floor")?
    .texture("wall")?
    .sound("coin")?
    .music("theme")?
    .build();
```

This looks for `assets/textures/player.png`, `assets/sounds/coin.wav` (then
`coin.ogg`), and `assets/music/theme.wav` (then `theme.ogg`). PNG is the current
beginner texture convention. OGG still needs the runtime's optional `ogg`
feature; a WAV file is always supported.

Audio assets support file-backed WAV sound effects and music through `.sound(...)`
and `.music(...)`. WAV is always available; OGG Vorbis is available when the
optional `ogg` feature is enabled on your runtime package. WAV and OGG are loaded
into memory at startup and normalized to the mixer sample rate and channel count.
For a standalone game, enable it in `Cargo.toml`:

```toml
game-starter = { path = "../game/crates/game-starter", features = ["ogg"] }
```

`generated_sound(...)` remains useful for tests and quick placeholders. MP3 and
streaming music are not implemented yet.

Play named assets through the beginner audio surface—no handles need to travel
into gameplay callbacks:

```rust
game.audio().play_sound("hit");
game.audio().play_music("theme").volume(0.4).fade_in(0.5);
game.audio().set_master_volume(0.8);
game.audio().set_sfx_volume(0.8);
game.audio().set_music_volume(0.5);
game.audio().fade_music_to(0.0, 1.0);
game.audio().pause_music();
game.audio().resume_music();
```

Music is intentionally memory-loaded for the small demos today; streaming is a
future optimization.

Use `game.input(|input| input.top_down_controls())?` for the standard first-game
bindings: WASD/arrows movement, Space/Enter attack, `R` reset, `P` or Escape
pause, `F1` debug overlay, and zoom keys.

Use prefab builders for common actors:

```rust
game.player_prefab("player")
    .sprite("player")
    .health(100)
    .moves_with(controls.movement, 130.0)
    .build()?;

game.enemy_prefab("slime")
    .sprite("slime")
    .chases_player()
    .melee(26.0, 6)
    .build()?;
```

Use in-code maps for simple levels. `.legend(...)` turns tile letters into
prefab spawns, and `.spawn(...)` places a specific object:

```rust
game.map("level_1")
    .tile_size(32.0)
    .tiles([
        "########",
        "#P...E.#",
        "########",
    ])
    .simple_theme("floor", "wall")
    .legend('P', "player")
    .legend('E', "slime")
    .spawn("extra_slime", "slime", cell(5, 1))
    .start();
```

Declare one `.start()` map. Additional maps should use `.finish()` and can be
entered through scene flow or door rules.

Use `game.rules()` to compose common game behavior without naming systems:

```rust
game.rules()
    .top_down_controls(controls)
    .player_collects_pickups()
    .doors_change_maps()
    .enemies_damage_player()
    .dead_enemies_despawn()
    .camera_follows_player()
    .pause_and_reset()
    .show_basic_ui()
    .build();
```

When you need sound on hits, animation switching, patrol behavior, or another
specific top-down option, use the beginner top-down preset directly:

```rust
game.use_top_down_game()
    .controls(controls)
    .hit_sound_named("hit")
    .with_melee_combat()
    .with_enemy_chase()
    .with_collision()
    .with_camera_follow()
    .with_pause_death_ui()
    .with_player_animation_by_movement()
    .build();
```

## Typed assets for larger content crates

The name-based path above is the right default for a first game. For a larger
Rust content crate, a typed asset struct can make dependencies explicit:

```rust
struct ArenaAssets {
    player: TextureHandle,
    slime: TextureHandle,
}
```

Use `game.assets(..)` and `AssetAuthor` when you deliberately choose that
shape:

```rust
let assets = game.assets(|assets| {
    Ok(DemoAssets {
        player: assets.texture("demo/player", "textures/test.png")?,
        hit: assets.sound("demo/hit", "sounds/hit.wav")?,
    })
})?;
```

This is an intermediate Rust-content pattern. It is not required for beginner
prefabs, maps, rules, or named audio playback.

## Advanced Path

Advanced content imports the lower-level facade:

```rust
use game_kit::advanced::prelude::*;
```

Use this path for custom tuple prefabs, manual schedules, explicit queries, RON
map experiments, or specialized content tests. Advanced content still depends on
`game-kit`; it does not wire SDL, Vulkan, audio devices, schedules, validators,
registries, command queues, or raw runtime contexts.

Raw tuple prefabs belong in the advanced path:

```rust
game.prefab("demo/player", |prefab| {
    prefab
        .spawn(move |at| {
            (
                Transform::at(at),
                Velocity::default(),
                Sprite::new(assets.player, vec2s(20.0)),
                Collider::box_of(vec2s(20.0)),
            )
        })?
        .require::<Transform>()
        .require::<Collider>()
        .require::<Sprite>();
    Ok(())
})?;
```

Register custom systems through `GameApp`:

```rust
game.startup(startup);
game.fixed_active::<GameState>(player_control);
game.update(camera_follow);
game.ui(ui);
game.fixed_systems_are_pause_guarded();
```

Systems use `GameCtx` helpers for entity queries, input, map movement, pathing,
camera/UI/audio/resources, and reset behavior:

```rust
fn player_control(game: &mut GameCtx<'_, '_>, _dt: f32) {
    game.drive_input::<PlayerController, MoveSpeed>();
}

fn physics(game: &mut GameCtx<'_, '_>, dt: f32) {
    game.move_and_collide(dt);
}
```

Query-style helpers keep common component scans inside `GameCtx`:

```rust
game.each2::<Transform, Sprite>(|entity, transform, sprite| {
    let _ = (entity, transform, sprite);
});

let input = game.input().clone();
game.for_each3_copy_mut::<PlayerController, MoveSpeed, Velocity>(
    |_, controller, speed, velocity| {
        velocity.0 = input.axis2d(controller.move_axis) * speed.0;
    },
);
```

Use the explicit `for_each*` names when they make borrowing or copy behavior
clearer. The facade intentionally does not provide query macros or automatic
system-parameter injection yet.

Startup systems are fallible because content initialization can fail.
Fixed/update/UI systems are infallible by design. Runtime operations that should
not fail after validation expose infallible helpers such as
`reset_to_start_map_or_log`, which logs invariant failures instead of making
every gameplay system return `Result`.

Deferred commands are available through `game.commands()`:

```rust
let mut commands = game.commands();
commands.play_sound(assets.hit);
commands.despawn(entity);
```

Only commands consumed by the runtime are exposed.

## Validation

Plugin build finalization validates duplicate names, required prefab
components, map shape, required map objects, prefab references, content assets,
and renderer built-in assets before backend creation. Authoring mistakes return
`anyhow::Result` from plugin build instead of panicking in the facade.

## Testing

Production beginner content:

```rust
use game_kit::beginner::prelude::*;
```

Beginner tests:

```rust
use game_kit::beginner::testing::prelude::*;
```

Advanced content:

```rust
use game_kit::advanced::prelude::*;
```

Advanced tests needing raw ECS/world inspection:

```rust
use game_kit::advanced::testing::prelude::*;
```

## Do Not Touch

Content crates should not import `GameBuilder`, `Schedule`, `PrefabRegistry`,
`MapRegistry`, validators, raw `Ctx` / `StartCtx`, `CommandQueue`, runtime
crates, renderer/platform/audio backends, Vulkan, SDL, or GPU allocator types.
Production beginner content should also avoid raw `World`, `Entity::new`, raw
`Input`, raw `TileMap`, raw `NavGrid`, `ids_with`/`get`/`get_mut` loops, direct
`game-ai`/`game-physics` systems, and `apply_damage`; use beginner builders,
rules, collections, events, or `GameCtx` helpers instead.

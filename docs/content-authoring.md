# Content Authoring

Content crates use `game-kit` as their public engine surface:

```rust
use game_kit::prelude::*;
```

They describe assets, input, prefabs, maps, systems, and content-owned gameplay
state. They do not wire SDL, Vulkan, audio devices, schedules, validators,
registries, command queues, or raw runtime contexts.

## Allowed Surface

Content can define gameplay components/resources, register content through
`GameApp`, and write systems against `GameCtx` / `StartupGameCtx`. The runtime
owns the event loop, fixed timestep, backend startup, renderer extraction, asset
preflight, and command application.

## Assets

Use `game.assets(..)` and `AssetAuthor`:

```rust
let assets = game.assets(|assets| {
    Ok(DemoAssets {
        player: assets.texture("demo/player", "textures/test.png")?,
        hit: assets.generated_sound("demo/hit")?,
    })
})?;
```

Audio is generated-only today. File-backed sound requests are modeled and
validated, but the runtime does not decode or mix file audio yet.

## Input

Use logical actions and axes:

```rust
let controls = game.input(|input| {
    Ok(DemoControls {
        attack: input.action("attack")?.keys([Key::Space, Key::Enter]),
        movement: input.axis2d("move")?.wasd().arrows(),
    })
})?;
```

Systems read `game.pressed(..)`, `game.down(..)`, and `game.axis2d(..)` rather
than SDL keys.

## Prefabs

Register prefabs by string name and spawn tuple bundles:

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

Prefab requirements are validated during plugin build before backend startup.

## Maps

Use in-code maps or RON maps through `MapAuthor`:

```rust
game.map("demo")
    .tile_size(32.0)
    .tiles(["#####", "#...#", "#####"])
    .theme(TileTheme {
        floor: Sprite::new(assets.floor, vec2s(32.0)),
        wall: Sprite::new(assets.wall, vec2s(32.0)),
    })
    .spawn("player_start", "demo/player", cell(2, 1))
    .require_object("player_start")
    .start();
```

Map objects reference prefabs by name. The facade resolves and validates them.

## Systems

Register systems through `GameApp`:

```rust
game.startup(startup);
game.fixed_active::<GameState>(player_control);
game.update(camera_follow);
game.ui(ui);
game.fixed_systems_are_pause_guarded();
```

Systems use `GameCtx` helpers for entity queries, input, map movement, pathing,
camera/UI/audio/resources, and reset behavior.

```rust
fn player_control(game: &mut GameCtx<'_, '_>, _dt: f32) {
    game.drive_input::<PlayerController, MoveSpeed>();
}

fn physics(game: &mut GameCtx<'_, '_>, dt: f32) {
    game.move_and_collide(dt);
}
```

## Commands

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

Production content uses:

```rust
use game_kit::prelude::*;
```

Tests that need raw ECS inspection use:

```rust
use game_kit::testing::prelude::*;
```

## Do Not Touch

Content crates should not import `GameBuilder`, `Schedule`, `PrefabRegistry`,
`MapRegistry`, validators, raw `Ctx` / `StartCtx`, `CommandQueue`, runtime
crates, renderer/platform/audio backends, Vulkan, SDL, or GPU allocator types.
Production content should also avoid raw `World`, `Entity::new`, raw `Input`,
raw `TileMap`, raw `NavGrid`, `ids_with`/`get`/`get_mut` loops, direct
`game-ai`/`game-physics` systems, and `apply_damage`; use `GameCtx` helpers
instead.

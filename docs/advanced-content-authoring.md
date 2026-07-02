# Advanced content authoring

This document is intentionally second, not the starting point. Begin with the
[beginner authoring guide](beginner-authoring.md) unless you specifically need
custom ECS-shaped prefabs, systems, queries, advanced RON maps, or engine
pressure tests.
If you are deciding whether to cross that boundary, read
[when to use the advanced API](when-to-use-advanced-api.md) first.
The maintained import and layer contract is in
[api-boundary.md](api-boundary.md).

The repository's `testbed-content` crate uses the advanced surface on purpose:
it is an engine testbed, not a template for first projects.

The authoring levels are deliberate: start by copying `examples/one-file-demo`,
`examples/no-rust-shapes-demo`, `examples/script-like-custom-rules`,
`simple-content`, or `templates/simple-demo`; use `arena-content` for
structured beginner Rust; and treat `testbed-content` as advanced-only.

## Imports and typed assets

Advanced content imports the explicit lower-level facade:

```rust
use game_kit::advanced::prelude::*;
```

For a larger Rust content crate, typed asset structures can make dependencies
explicit:

```rust
struct ArenaAssets {
    player: TextureHandle,
    slime: TextureHandle,
}

let assets = game.assets(|assets| {
    Ok(ArenaAssets {
        player: assets.texture("arena/player", "textures/player.png")?,
        slime: assets.texture("arena/slime", "textures/slime.png")?,
    })
})?;
```

## Raw prefabs and systems

Use this path for custom tuple prefabs, manual schedules, explicit queries,
legacy/advanced RON map experiments, or specialized content tests. Advanced
content still depends on `game-kit`; it does not wire SDL, Vulkan, audio devices, schedules,
validators, registries, command queues, or raw runtime contexts.

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

Query helpers keep common component scans inside `GameCtx`:

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
system-parameter injection for beginner content. Advanced top-level functions
can additionally use typed query parameters:

```rust
fn move_player(
    mut players: Query<(&mut Transform, &Speed), With<Player>>,
    input: Res<Input>,
    dt: DeltaTime,
) {
    let _mouse = input.mouse_position();
    for (_, (transform, speed)) in &mut players {
        transform.pos.x += speed.0 * dt.0;
    }
}

game.fixed_params(move_player)?;
```

`fixed_params` and `update_params` validate all component and resource
accesses while the plugin is built. A mutable query cannot coexist with another
query or filter that reads or mutates the same component type; split that logic
into two systems or query different component types. `With<T>` and
`Without<T>` filter entities, `Res<T>` reads a resource (including the
frame's `Input`), and `ResMut<T>` mutates a world resource.

Resource borrows are checked the same way: reading and mutating the same
resource, or requesting two mutable borrows of it, fails during plugin build and
the error includes the resource type name. The current typed-function adapters
expose one resource parameter per system signature; use a single wrapper
resource or a `GameCtx` helper when a system needs several resource values.
Beginner content should keep using rules, hooks, and builders instead of typed
query parameters.

Startup systems are fallible because content initialization can fail. Fixed,
update, and UI systems are infallible by design. Runtime operations that should
not fail after validation expose helpers such as `reset_to_start_map_or_log`;
they log invariant failures instead of making every gameplay system return a
`Result`.

Deferred runtime operations use the command queue:

```rust
let mut commands = game.commands();
commands.play_sound(assets.hit);
commands.despawn(entity);
```

Map transitions are deliberately name-based through `GameCtx::change_map("level_2")`
or `change_map_or_log`. Do not queue raw active-map switches by `MapId`; the
name-based helper also updates the content runtime and respawns the new map's
objects.

## Validation and testing

Plugin build finalization validates duplicate names, required prefab
components, map shape, required map objects, prefab references, content assets,
and renderer built-in assets before backend creation. Authoring mistakes return
`anyhow::Result` from plugin build rather than panicking in the facade.

Use the testing prelude that matches the content layer:

```rust
// Beginner production content
use game_kit::beginner::prelude::*;
// Beginner tests
use game_kit::beginner::testing::prelude::*;
// Advanced production content
use game_kit::advanced::prelude::*;
// Advanced tests with raw ECS/world inspection
use game_kit::advanced::testing::prelude::*;
```

## Keep the boundary intact

Even advanced content should not import `GameBuilder`, `Schedule`,
`PrefabRegistry`, `MapRegistry`, validators, raw `Ctx` / `StartCtx`,
`CommandQueue`, runtime crates, renderer/platform/audio backends, Vulkan, SDL,
or GPU allocator types. When an advanced project repeatedly needs a
beginner-friendly pattern, add one focused `game-kit` builder or rule instead
of copying lower-level plumbing into every game.

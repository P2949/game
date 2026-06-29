# Custom tags and timers

Use `.tag(...)` to name a group of actors and `.data(...)` to give each one a
small numeric value. A tick rule can then select that group with
`actors_tagged(...)` and update each actor through a friendly handle.

## A short fuse

```rust
game.enemy_prefab("bomber")
    .sprite("slime")
    .tag("enemy")
    .tag("explosive")
    .data("fuse", 3.0)
    .build()?;

game.every_active_tick::<SimpleGameState>(|game, dt| {
    game.actors_tagged("explosive").each(|actor| {
        let fuse = actor.data("fuse").unwrap_or(0.0) - dt;
        actor.set_data("fuse", fuse);
    });
});
```

`data` is useful for small per-actor numbers such as a fuse, a glow amount, a
spawn delay, or a temporary movement multiplier.

## A temporary slow effect

Give affected enemies a `slowed` tag and a time remaining value when your spell
hits them. Then clear the effect when its timer ends:

```rust
game.actors_tagged("slowed").each(|actor| {
    let remaining = actor.data("slow_seconds").unwrap_or(0.0) - dt;
    actor.set_data("slow_seconds", remaining);
    if remaining <= 0.0 {
        actor.set_data("slow_multiplier", 1.0);
    }
});
```

For a new effect, tag its prefab with `.tag("slowed")` and seed it with
`.data("slow_seconds", 2.0)` and `.data("slow_multiplier", 0.5)`.

## A damage glow

Use a tag to pick glowing actors and a value as the amount of glow left to
fade. Your damage rule can set `.data("glow", 1.0)`, then a tick rule can ease
it back down:

```rust
game.actors_tagged("glowing").each(|actor| {
    let glow = (actor.data("glow").unwrap_or(0.0) - dt * 2.0).max(0.0);
    actor.set_data("glow", glow);
});
```

The tag and data APIs keep the custom rule readable. When a behavior needs a
large amount of state or a reusable engine-wide system, that is the point to
graduate to a content crate.

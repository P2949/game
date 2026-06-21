# Triggers And Areas

Copy [examples/trigger-area-demo/src/main.rs](../../examples/trigger-area-demo/src/main.rs)
when a level needs a trap, a checkpoint, a win zone, a door sensor, or another
non-solid overlap region.

Areas have a transform and collider but need no sprite. Their prefab name is
what the event methods match:

```rust
game.trigger_prefab("danger_zone")
    .size(vec2(64.0, 64.0))
    .build()?;

game.on_enter_area("player", "danger_zone", |event| {
    event.actor().damage(10);
    event.play_sound("hurt");
});

game.on_exit_area("player", "danger_zone", |event| {
    event.play_sound("warning");
});
```

Use `game.on_collision("player", "spike", ...)` when the callback should run
every tick that two colliders overlap. The callback's neutral names work for any
pair: `event.a()`, `event.b()`, `event.actor()`, `event.area()`, and
`event.other()`. Add `.visible_debug("debug_trigger")` to an area while
authoring if you want a temporary visual.

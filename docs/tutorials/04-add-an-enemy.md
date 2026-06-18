# 04 - Add An Enemy

## Goal

Create a simple enemy prefab.

## What you will build

A `slime` enemy that has a sprite, chases the player, and can make melee hits.

## Files you will edit

`crates/simple-content/src/game.rs`

## Final code

```rust
game.enemy_prefab("slime")
    .sprite(assets.slime)
    .chases_player()
    .melee(26.0, 6)
    .build()?;
```

To place it on the map:

```rust
.spawn("slime_01", "slime", cell(5, 2))
```

## Explanation

`enemy_prefab` creates a reusable enemy spawn recipe. `.chases_player()` attaches
the beginner chase behavior. `.melee(26.0, 6)` means the enemy can hit in a
26-unit range for 6 damage.

The map still decides where enemies appear. Prefabs describe what an actor is;
map spawns describe where instances start.

## Common errors

If the enemy does not appear, check the map spawn name and position.

If the enemy does not move, make sure the top-down preset later includes
`.with_enemy_chase()`.

If the enemy cannot damage the player, make sure combat is enabled in the next
tutorial.

## Next step

Wire the combat systems in [05 - Add combat](05-add-combat.md).

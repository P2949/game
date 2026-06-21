# LDtk levels

Use LDtk when you want to edit collision and named object placements visually.
This project imports one IntGrid layer: `0` means floor and every non-zero cell
means wall. Entity identifiers map to the prefab names you already use in code.

```rust
game.map_from_ldtk("level_1", "maps/world.ldtk")
    .level("Level_1")
    .simple_theme("floor", "wall")
    .entity("PlayerStart", "player")
    .entity("Slime", "slime")
    .entity("Coin", "coin")
    .entity("Exit", "exit")
    .start();
```

Save the LDtk project beneath `assets/maps/`. Every entity in the selected level
must have an `.entity(...)` mapping; the setup error names a missing mapping and
shows the line to add. The mapped prefab must also exist before the map starts.

Copy [the LDtk demo](../../examples/ldtk-demo/src/main.rs) for a working player
and enemy setup.

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

## Supported LDtk surface

- One embedded level selected by `.level("Level_1")`.
- The first IntGrid layer as collision: `0` is floor and every non-zero value
  is a wall.
- Any number of Entities layers. Each entity uses its LDtk identifier and
  top-left cell to spawn the mapped prefab.
- The normal beginner tile theme through `.simple_theme(...)`.

## Not supported yet

- External-level files (`externalRelPath` / externally stored layers).
- Tile layers, auto layers, backgrounds, fields, and LDtk gameplay values.
- Multiple collision interpretations, per-value collision rules, and LDtk
  project hot reload.
- Tiled `.tmx` import.

## Debugging imports

If setup says the level has no IntGrid collision layer, add one with a positive
grid size and make its dimensions match its `intGridCsv` data. If an entity has
no prefab mapping, copy the suggested `.entity("Identifier", "prefab")` line
from the error. If a mapped prefab is unknown, define that prefab before the
map. Entities with negative positions or cells outside the IntGrid are rejected
so a misplaced object cannot silently spawn somewhere unexpected.

Copy [the LDtk demo](../../examples/ldtk-demo/src/main.rs) for a working player
and enemy setup.

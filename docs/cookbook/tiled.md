# Tiled TMX maps

The beginner importer supports a small, dependable Tiled workflow. It reads an
**orthogonal XML TMX** map with square tiles, a tile layer named `Collision`,
and ordinary object-group objects.

In Tiled:

1. Create a square-tile orthogonal map.
2. Add a tile layer named `Collision` (case-insensitive).
3. Set that layer's data encoding to **CSV**, with no compression. Tile gid `0`
   is floor; every non-zero gid is a wall.
4. Add objects in an object group. Give each object a `Class`, `Type`, or `Name`
   such as `Player` or `Slime`.

Then map those identifiers to the prefabs you registered:

```rust
game.map_from_tiled("level_1", "maps/level_1.tmx")
    .object("Player", "player")
    .object("Slime", "slime")
    .simple_theme("floor", "wall")
    .start();
```

The importer uses the same collision, prefab, and map validation path as text
and LDtk maps. An unmapped object tells you the exact `.object(...)` call to
add; an object outside the map or on a wall also fails before play begins.

## Current scope

This is collision-and-object import, not a general renderer for a Tiled
project. Tilesets, visual tile layers, object properties, image layers,
infinite maps, templates, base64/compressed layer data, non-square tiles, and
isometric maps remain out of scope. Use `.simple_theme(...)` for the floor and
wall rendering in this first workflow.

See [`assets/maps/tiled_demo.tmx`](../../assets/maps/tiled_demo.tmx) for the
small checked-in fixture used by the map-flow tests.

Runnable example:

```bash
cargo run -p tiled-demo
```

## No-Rust Data File

The same importer can be driven from `assets/game.ron`:

```ron
maps: [
    Tiled((
        name: "level_1",
        path: "maps/tiled_demo.tmx",
        theme: ("floor", "wall"),
        objects: {
            "Player": "player",
            "Slime": "slime",
        },
        start: true,
    )),
]
```

Run the focused data-driven example with:

```bash
cargo run -p data-driven-tiled-demo
```

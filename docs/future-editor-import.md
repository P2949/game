# Editor and Import Roadmap

## Supported today

- In-code maps through `game.map(...)`.
- Text maps through `game.map_from_text(...)`.
- Conventional text maps through `game.map_from_text_auto(...)`.
- Minimal LDtk import through `game.map_from_ldtk(...)`.
- Minimal Tiled TMX import through `game.map_from_tiled(...)`.
- F5 reload for text maps, configured tuning files, and registered textures and
  sounds in development builds.

## Recommended beginner workflow

Use text maps first. Keep `#`, `.`, and your legend symbols in
`assets/maps/*.txt`, then load them with `map_from_text_auto(...)`. Text maps
are easy to review, version, and change without Rust code.

Press F5 during development to reload registered textures, sounds, and the
active text map. If the game has a configured tuning file, it reloads before
the map so newly spawned actors use the new values. Press F1 for the debug
overlay, which reports the active map and the result of the latest reload.
Release builds can opt in with `GAME_DEV_RELOAD=1`.

## Visual editor workflow

Use LDtk when you want visual editing and can accept the current minimal
importer. It imports one IntGrid layer (`0` is floor; every non-zero cell is a
wall) and maps named LDtk entities to the prefabs you configure in Rust. See
the [LDtk cookbook](cookbook/ldtk.md) for the supported setup and diagnostics.
Tile layers, auto layers, entity fields, external levels, richer collision,
and project hot reload are not supported today.

Use Tiled when you prefer its object-layer workflow. The supported contract is
an orthogonal square-tile XML TMX map with an uncompressed CSV layer named
`Collision` (`0` floor, non-zero wall) and objects identified by class, type,
or name. Map each identifier to a prefab with `.object(...)`; see the
[Tiled cookbook](cookbook/tiled.md). Tilesets, visual tile layers, object
properties, infinite maps, templates, and compressed/base64 data are not part
of this initial importer.

## Future work

- Sprite-sheet metadata reload.
- Richer LDtk mapping.

Rust code, LDtk projects, and Tiled projects still require a restart.
Registered textures reload in place, including files whose dimensions changed.
Reloading a static sound
stops voices using its old samples; a registered streamed music track restarts
from its updated file.

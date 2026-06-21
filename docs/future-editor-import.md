# Future Editor Import

Text maps are the supported beginner workflow today: keep `#`, `.`, and
legend symbols in `assets/maps/*.txt`, then load them with `map_from_text(...)`.
They are easy to review, version, and change without Rust code.

Tiled (`.tmx`) and LDtk (`.ldtk`) import are intentionally future work. A later
optional integration may add APIs such as:

```rust
game.map_from_tiled("level_1", "maps/level_1.tmx");
game.map_from_ldtk("world", "maps/world.ldtk");
```

Those importers should preserve the current beginner concepts—named prefabs,
tile themes, and clear diagnostics—rather than exposing an editor's raw data
model to ordinary game code.
# Fast iteration before an editor

The first editor loop is deliberately code-and-text based: put files in the
conventional asset folders, load a map with `map_from_text_auto("level_1")`,
edit `assets/maps/level_1.txt`, and press F5 in a debug build. The reload keeps
the same prefabs, legends, and theme while rebuilding the current map.
When a tuning file is configured, it reloads that RON data first so respawned
actors use the new numeric values.

Press F1 for the debug overlay: it names the active map, shows how many assets
were registered, and reports whether the latest reload succeeded. Release builds
can opt in to the same manual reload loop with `GAME_DEV_RELOAD=1`.

Only text maps reload today. Rust code, textures, and sounds still require a
restart; texture reload will follow once renderer asset replacement has a small,
safe API.

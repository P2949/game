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

# RON to TOML

Legacy RON projects use `assets/game.ron` inside a Rust wrapper. The primary
no-Rust package uses a root `game.toml` file and runs through the prebuilt
player.

From the project root, run:

```bash
game-dev migrate-ron assets/game.ron --out game.toml
game-dev check
game-dev preview
```

The migration command parses the legacy RON file, normalizes it through the
authoring model, writes canonical TOML, validates the generated file, and prints
notes for compatibility details. Keep the old RON file in source control until
the converted package has been checked.

## Small Example

Legacy RON:

```ron
Player((
    name: "player",
    melee: Some((range: 30.0, damage: 25)),
))
```

Primary TOML:

```toml
[[prefab]]
kind = "player"
name = "player"

[prefab.melee]
range = 30
damage = 25
```

## What Changes

- Root `assets/game.ron` becomes root `game.toml`.
- Rust-shaped wrappers such as `Player((...))` become `kind = "player"` rows.
- Optional values such as `Some((...))` become ordinary nested TOML tables.
- Projectile `lifetime` is emitted as primary `duration`.
- Script rules become `[[rule]]` and `[[rule.then]]` tables.
- Text maps, textures, sounds, music, and animation sheet names still resolve
  through the same `assets/` folder convention.

After migration, use `game-dev check` as the project gate. Use
`game-dev preview --watch` while editing when structural changes should restart
the prebuilt player automatically.

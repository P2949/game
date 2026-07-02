# No-Rust Package Layout

The primary no-Rust package target is a normal folder, not a Cargo project.
This layout is the contract the CLI, player, templates, docs, and release SDK
use for primary no-Rust authoring.

```text
my-game/
  game.toml
  assets/
    textures/
      player.png
      slime.png
      floor.png
      wall.png
    sounds/
      hit.wav
      coin.wav
    music/
      theme.ogg
    fonts/
      DejaVuSans.ttf
    maps/
      level-1.txt
    animations/
      player.toml
  README.txt
```

## Required Files

`game.toml` is the primary authoring file. It contains the game title, start
map, asset names, prefabs, maps, rules, and other declarative game setup.
Primary package metadata lives here too:

```toml
[game]
title = "My Game"
window_width = 1280
window_height = 720
```

Do not use Cargo metadata such as `[package.metadata.game]` for primary
packages. Cargo metadata remains a secondary Rust starter-project compatibility
surface only.

`assets/` contains media and map files. Beginners should use the conventional
folders first:

- `assets/textures/` for PNG images.
- `assets/sounds/` for sound effects.
- `assets/music/` for music.
- `assets/fonts/` for fonts.
- `assets/maps/` for text maps, Tiled TMX, or LDtk files.
- `assets/animations/` for primary animation metadata in `*.toml` files.

The primary package must not contain `Cargo.toml`, `src/main.rs`, `build.rs`, or
other Rust project files. Legacy RON data can remain in legacy projects, but it
is not part of the primary package layout.

## Target Commands

<!-- primary-no-rust:start -->
The primary no-Rust workflow edits `game.toml` and `assets/`, then runs the
package with a prebuilt executable (`game-player`):

```bash
game-dev check
game-dev preview
game-dev preview --watch
game-dev package --out dist/my-game --zip
```

`game-dev check` validates `game.toml`, maps, animation metadata, and assets
without running Cargo.

`game-dev preview` runs the package through the prebuilt player.

`game-dev preview --watch` restarts the prebuilt player after `game.toml`, map,
or asset edits. This is the primary path for structural changes such as adding
or removing prefabs, maps, rules, actions, or asset keys. It does not compile
user code.

`game-dev package` creates a shareable folder or zip containing the package and
the prebuilt executable and helper files needed to run it.
<!-- primary-no-rust:end -->

No Rust toolchain is required for the primary package workflow.

## Example Packages

The repository keeps checked-in no-Rust packages that follow this layout:

- `examples/no-rust-minimal`
- `examples/no-rust-events`
- `examples/no-rust-waves`
- `examples/no-rust-projectiles`
- `examples/no-rust-full`
- `examples/no-rust-tiled`

Each package has a root `game.toml`, an `assets/` folder, and no Cargo project
files.

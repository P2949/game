{{title}}

1. Open game.toml in any text editor.
2. Run ./game-dev check.
3. Run ./game-dev preview.
4. Run ./game-dev preview --watch while editing game.toml, maps, or assets.
5. No Rust or Cargo is needed for this template.

F5 reloads the current text map and supported existing values in a running
debug player. preview --watch restarts the prebuilt player for structural edits
such as adding or removing prefabs, maps, rules, actions, or asset keys.

Starter files:

- game.toml
- assets/maps/level-1.txt
- assets/textures/player.png
- assets/textures/slime.png
- assets/textures/coin.png
- assets/textures/floor.png
- assets/textures/wall.png
- assets/sounds/coin.wav

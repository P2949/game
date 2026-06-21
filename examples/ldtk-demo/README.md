# LDtk demo

This demo loads `assets/maps/ldtk_demo.ldtk`. Its `Level_1` IntGrid uses `0` for
floor and non-zero values for walls. The entities layer maps `PlayerStart` to
the player prefab and `Slime` to the enemy prefab in `src/main.rs`.

Open the file in LDtk, move or add entities, save it, and run the demo. LDtk
projects reload on the next run today; F5 is reserved for text-map iteration.

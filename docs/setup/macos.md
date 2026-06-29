# macOS setup

Install Rust from <https://rustup.rs>, then install the developer dependencies
with Homebrew:

```bash
brew install sdl3 shaderc molten-vk vulkan-tools
```

Run the checker from a generated project:

```bash
game-dev doctor
```

macOS uses Vulkan through MoltenVK. It is useful for development, but it does
not behave exactly like native Windows/Linux Vulkan drivers. If validation
layers or a present mode are unavailable, try
`GAME_DISABLE_VALIDATION=1 cargo run -p game`; report reproducible renderer
issues with the output from `game-dev doctor` or `cargo xtask doctor`.
Use `game-dev doctor --explain` when you want longer beginner explanations for
each failed check.

## Prebuilt demo package

The first prebuilt demo packages in
[Releases](https://github.com/P2949/game/releases) target Linux x86_64 and
Windows x86_64. macOS support is still source-build first: install the Homebrew
dependencies above, then run a generated project or local checkout. Like the
other platforms, macOS still needs a Vulkan-capable GPU/driver path through
MoltenVK.

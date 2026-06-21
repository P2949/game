# macOS setup

Install Rust from <https://rustup.rs>, then install the developer dependencies
with Homebrew:

```bash
brew install sdl3 shaderc molten-vk vulkan-tools
```

Run the checker from the repository:

```bash
cargo xtask doctor
```

macOS uses Vulkan through MoltenVK. It is useful for development, but it does
not behave exactly like native Windows/Linux Vulkan drivers. If validation
layers or a present mode are unavailable, try
`GAME_DISABLE_VALIDATION=1 cargo run -p game`; report reproducible renderer
issues with the output from `cargo xtask doctor`.

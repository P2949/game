# Linux setup

Install Rust from <https://rustup.rs>, then install SDL3, Vulkan tools,
validation layers, and a shader compiler. Package names differ by distribution.

Ubuntu or Debian:

```bash
sudo apt update
sudo apt install build-essential pkg-config libsdl3-dev vulkan-tools \
  vulkan-validationlayers glslc libasound2-dev
```

Fedora:

```bash
sudo dnf install gcc pkgconf-pkg-config SDL3-devel vulkan-tools \
  vulkan-validation-layers shaderc alsa-lib-devel
```

Arch Linux:

```bash
sudo pacman -S --needed base-devel pkgconf sdl3 vulkan-tools \
  vulkan-validation-layers shaderc alsa-lib
```

Gentoo:

```bash
sudo emerge --ask media-libs/libsdl3 media-libs/shaderc dev-util/vulkan-tools media-libs/alsa-lib
```

Then run `game-dev doctor` from a generated project, or `cargo xtask doctor`
from an engine checkout. If the Vulkan driver is absent, install the
vendor package appropriate for your GPU before attempting to run the game.
Use `game-dev doctor --explain` when you want longer beginner explanations for
each failed check.

## Prebuilt demo package

Want to try before building? Download `game-demo-linux-x86_64.zip` from the
latest [Releases](https://github.com/P2949/game/releases), unzip it, and run
`./run.sh` from the extracted folder. The package includes the executable,
`assets/`, launcher scripts, and `README.txt`; it does not require a Rust
toolchain or SDL3 development headers.

It still requires a Vulkan-capable GPU/driver. If the prebuilt demo does not
start, install the Vulkan loader/tools package plus the GPU vendor driver for
your distribution.

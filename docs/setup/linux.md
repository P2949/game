# Linux setup

Install Rust from <https://rustup.rs>, then install SDL3, Vulkan tools,
validation layers, and a shader compiler. Package names differ by distribution.

Ubuntu or Debian:

```bash
sudo apt update
sudo apt install build-essential pkg-config libsdl3-dev vulkan-tools \
  vulkan-validationlayers glslc
```

Fedora:

```bash
sudo dnf install gcc pkgconf-pkg-config SDL3-devel vulkan-tools \
  vulkan-validation-layers shaderc
```

Arch Linux:

```bash
sudo pacman -S --needed base-devel pkgconf sdl3 vulkan-tools \
  vulkan-validation-layers shaderc
```

Gentoo:

```bash
sudo emerge --ask media-libs/libsdl3 media-libs/shaderc dev-util/vulkan-tools
```

Then run `cargo xtask doctor`. If the Vulkan driver is absent, install the
vendor package appropriate for your GPU before attempting to run the game.

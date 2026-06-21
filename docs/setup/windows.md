# Windows setup

Install Rust from <https://rustup.rs>, then open PowerShell and install the
Vulkan SDK:

```powershell
winget install KhronosGroup.VulkanSDK
```

Restart PowerShell so `glslc` is on `PATH`, then run:

```powershell
cargo xtask doctor
```

Install SDL3 with vcpkg if the doctor reports it missing:

```powershell
git clone https://github.com/microsoft/vcpkg $HOME/vcpkg
& $HOME/vcpkg/bootstrap-vcpkg.bat
& $HOME/vcpkg/vcpkg install sdl3:x64-windows
$env:SDL3_DIR = "$HOME/vcpkg/installed/x64-windows"
```

Keep the Vulkan SDK validation layers installed during development. If a
machine cannot provide them, `GAME_DISABLE_VALIDATION=1 cargo run -p game` is
an explicit fallback, not the preferred default.

# Windows setup

Install Rust from <https://rustup.rs>, then open PowerShell and install the
Vulkan SDK:

```powershell
winget install KhronosGroup.VulkanSDK
```

Restart PowerShell so `glslc` is on `PATH`, then run:

```powershell
game-dev doctor
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
Use `game-dev doctor --explain` when you want longer beginner explanations for
each failed check.

## Prebuilt demo package

Want to try before building? Download `game-demo-windows-x86_64.zip` from the
latest [Releases](https://github.com/P2949/game/releases), unzip it, and run
`run.ps1` or `run.bat` from the extracted folder. The package includes the
executable, `assets/`, launcher scripts, and `README.txt`; it does not require
a Rust toolchain or SDL3 development headers.

It still requires a Vulkan-capable GPU/driver. If the prebuilt demo does not
start, update your graphics driver; the Vulkan Runtime is usually included with
current NVIDIA, AMD, and Intel drivers.

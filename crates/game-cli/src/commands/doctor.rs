use std::env;
use std::fs;
use std::process::Command;

use anyhow::{Result, bail};

#[derive(Clone, Copy)]
pub(crate) struct DoctorOptions {
    pub(crate) explain: bool,
}

pub(crate) fn parse_doctor_options(
    args: impl IntoIterator<Item = String>,
) -> Result<DoctorOptions> {
    let mut explain = false;
    for argument in args {
        match argument.as_str() {
            "--explain" => explain = true,
            extra => bail!("unexpected doctor argument '{extra}'; expected --explain"),
        }
    }
    Ok(DoctorOptions { explain })
}

pub(crate) fn doctor(options: DoctorOptions) {
    println!("game environment doctor\n");

    report_check(
        check_cargo(),
        "Cargo",
        "Cargo builds and runs your generated project.",
        "Install Rust from https://rustup.rs, then open a new terminal.",
        "Cargo is the Rust build tool. Without it, `game-dev run` and `game-dev package` cannot call `cargo run` or `cargo build`.",
        options,
    );

    report_check(
        check_rustc_version(),
        "rustc 1.87 or newer",
        "The framework uses Rust 2024 and dependencies that require Rust 1.87+.",
        "Run `rustup update stable`, or install Rust from https://rustup.rs.",
        "The project declares rust-version = 1.87 so Cargo can give consistent errors across machines.",
        options,
    );

    let shader = executable_on_path("glslc") || executable_on_path("shaderc");
    report_check(
        shader,
        "shader compiler (glslc or shaderc)",
        "Shaders must compile before the Vulkan renderer can start.",
        &shader_install_hint(),
        "Debug and release builds both need a GLSL compiler. `glslc` comes from the Vulkan SDK or distro shaderc packages.",
        options,
    );

    let vulkan_summary = command_output("vulkaninfo", &["--summary"]);
    let vulkan = vulkan_summary
        .as_ref()
        .is_some_and(|output| output.status.success());
    report_check(
        vulkan,
        "Vulkan loader and a usable driver",
        "The windowed renderer needs Vulkan before it can open the game.",
        &vulkan_install_hint(),
        "If `vulkaninfo --summary` fails, the renderer cannot choose a physical device. Install the loader, tools, and the GPU vendor driver.",
        options,
    );

    let vulkan_13 = vulkan_summary
        .as_ref()
        .and_then(|output| String::from_utf8(output.stdout.clone()).ok())
        .is_some_and(|text| has_vulkan_13_device(&text));
    report_check(
        vulkan_13,
        "Vulkan 1.3+ physical device",
        "The renderer expects a Vulkan 1.3 capable GPU or software driver.",
        &vulkan_install_hint(),
        "Integrated GPUs are fine if their driver reports Vulkan 1.3 or newer. On CI, lavapipe provides a software device.",
        options,
    );

    let sdl3 =
        command_succeeds("pkg-config", &["--exists", "sdl3"]) || env::var_os("SDL3_DIR").is_some();
    report_check(
        sdl3,
        "SDL3 development files",
        "SDL3 creates the window, reads input, and opens the audio device.",
        &sdl3_install_hint(),
        "On Linux, `pkg-config --exists sdl3` should succeed. On Windows, set SDL3_DIR when using vcpkg.",
        options,
    );

    let audio = audio_prerequisites_available();
    report_check(
        audio,
        "audio backend prerequisites",
        "The mixer needs the platform audio development files that SDL3 links against.",
        &audio_install_hint(),
        "Linux source builds commonly need ALSA development files. Windows and macOS usually get this through their normal SDK/Homebrew setup.",
        options,
    );

    let assets = env::current_dir()
        .map(|dir| dir.join("assets"))
        .is_ok_and(|path| path.is_dir());
    report_check(
        assets,
        "assets/ folder",
        "Generated games load maps, textures, sounds, and data from assets/.",
        "Run this command from your generated project folder, or create the project with `game-dev new my-game`.",
        "The runtime searches for an assets folder next to your project or packaged binary. Running commands from another directory is a common first-run mistake.",
        options,
    );

    let font = env::current_dir()
        .map(|dir| dir.join("assets/fonts/DejaVuSans.ttf"))
        .is_ok_and(|path| path.is_file());
    report_optional_check(
        font,
        "assets/fonts/DejaVuSans.ttf",
        "Release packages need the built-in UI font beside assets/.",
        "For generated projects, `game-dev package --release --out dist/my-game` adds the bundled font automatically. To run a hand-made release layout, copy assets/fonts/DejaVuSans.ttf into your project.",
        "Debug builds can use the framework checkout fallback, but release packages should be self-contained.",
        options,
    );

    let validation = vulkan
        && vulkan_summary.as_ref().is_some_and(|output| {
            String::from_utf8_lossy(&output.stdout).contains("VK_LAYER_KHRONOS_validation")
        });
    report_optional_check(
        validation,
        "Vulkan validation layers",
        "Validation layers turn many renderer mistakes into readable debug messages.",
        &validation_install_hint(),
        "If your machine cannot provide validation layers, use GAME_DISABLE_VALIDATION=1 only as a local fallback. Keep validation enabled when debugging renderer issues.",
        options,
    );

    if shader && vulkan && vulkan_13 && sdl3 && audio && assets {
        println!("\nCore prerequisites look available. Try: game-dev run");
    } else {
        println!("\nFix the failed checks above, then run this command again.");
    }
}

fn report_check(
    ok: bool,
    name: &str,
    why: &str,
    fix: &str,
    explanation: &str,
    options: DoctorOptions,
) {
    if ok {
        println!("[ok] {name}");
    } else {
        println!("[fail] {name}");
        println!("  why: {why}");
        println!("  fix: {fix}");
        if options.explain {
            println!("  details: {explanation}");
        }
    }
}

fn report_optional_check(
    ok: bool,
    name: &str,
    why: &str,
    fix: &str,
    explanation: &str,
    options: DoctorOptions,
) {
    if ok {
        println!("[ok] {name}");
    } else {
        println!("[warn] {name}");
        println!("  why: {why}");
        println!("  fix: {fix}");
        if options.explain {
            println!("  details: {explanation}");
        }
    }
}

fn command_succeeds(command: &str, args: &[&str]) -> bool {
    command_output(command, args).is_some_and(|output| output.status.success())
}

fn command_output(command: &str, args: &[&str]) -> Option<std::process::Output> {
    Command::new(command).args(args).output().ok()
}

fn check_cargo() -> bool {
    command_succeeds("cargo", &["--version"])
}

fn check_rustc_version() -> bool {
    command_output("rustc", &["--version"])
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .and_then(|text| parse_rustc_minor(&text))
        .is_some_and(|(major, minor)| major > 1 || (major == 1 && minor >= 87))
}

fn parse_rustc_minor(text: &str) -> Option<(u32, u32)> {
    let version = text.split_whitespace().nth(1)?;
    let mut parts = version.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    Some((major, minor))
}

fn has_vulkan_13_device(summary: &str) -> bool {
    summary.lines().any(|line| {
        let Some(version) = line.split("apiVersion").nth(1) else {
            return false;
        };
        let Some(version) = version.split('=').nth(1) else {
            return false;
        };
        let mut parts = version.trim().split('.');
        let major = parts.next().and_then(|part| part.parse::<u32>().ok());
        let minor = parts.next().and_then(|part| part.parse::<u32>().ok());
        matches!((major, minor), (Some(major), Some(minor)) if major > 1 || (major == 1 && minor >= 3))
    })
}

fn audio_prerequisites_available() -> bool {
    if cfg!(target_os = "linux") {
        command_succeeds("pkg-config", &["--exists", "alsa"])
            || command_succeeds("pkg-config", &["--exists", "libpipewire-0.3"])
    } else {
        true
    }
}

fn shader_install_hint() -> String {
    match platform_hint() {
        PlatformHint::Linux(command) => {
            format!("{command}; package names are usually glslc or shaderc")
        }
        PlatformHint::Macos => "brew install shaderc".to_string(),
        PlatformHint::Windows => "winget install KhronosGroup.VulkanSDK".to_string(),
        PlatformHint::Other => {
            "install the Vulkan SDK or your platform's shaderc/glslc package".to_string()
        }
    }
}

fn vulkan_install_hint() -> String {
    match platform_hint() {
        PlatformHint::Linux(command) => {
            format!("{command}; also install your GPU vendor Vulkan driver if needed")
        }
        PlatformHint::Macos => "brew install molten-vk vulkan-tools".to_string(),
        PlatformHint::Windows => "winget install KhronosGroup.VulkanSDK".to_string(),
        PlatformHint::Other => "install Vulkan loader, tools, and a Vulkan 1.3 driver".to_string(),
    }
}

fn validation_install_hint() -> String {
    match platform_hint() {
        PlatformHint::Linux(command) => format!("{command}; include vulkan-validationlayers"),
        PlatformHint::Macos => "brew install vulkan-tools".to_string(),
        PlatformHint::Windows => "winget install KhronosGroup.VulkanSDK".to_string(),
        PlatformHint::Other => "install Vulkan validation layers for your platform".to_string(),
    }
}

fn sdl3_install_hint() -> String {
    match platform_hint() {
        PlatformHint::Linux(command) => format!("{command}; include SDL3 development files"),
        PlatformHint::Macos => "brew install sdl3".to_string(),
        PlatformHint::Windows => {
            "vcpkg install sdl3:x64-windows; set SDL3_DIR to the installed prefix".to_string()
        }
        PlatformHint::Other => "install SDL3 development files for your platform".to_string(),
    }
}

fn audio_install_hint() -> String {
    match platform_hint() {
        PlatformHint::Linux(command) => format!("{command}; include ALSA development files"),
        PlatformHint::Macos => {
            "Homebrew SDL3 setup normally covers audio prerequisites".to_string()
        }
        PlatformHint::Windows => {
            "The Windows SDK and SDL3/vcpkg setup normally cover audio prerequisites".to_string()
        }
        PlatformHint::Other => "install your platform audio development files".to_string(),
    }
}

enum PlatformHint {
    Linux(String),
    Macos,
    Windows,
    Other,
}

fn platform_hint() -> PlatformHint {
    if cfg!(target_os = "windows") {
        return PlatformHint::Windows;
    }
    if cfg!(target_os = "macos") {
        return PlatformHint::Macos;
    }
    if cfg!(target_os = "linux") {
        return PlatformHint::Linux(linux_install_command());
    }
    PlatformHint::Other
}

fn linux_install_command() -> String {
    let id_like = fs::read_to_string("/etc/os-release").unwrap_or_default();
    let lower = id_like.to_ascii_lowercase();
    if lower.contains("id=ubuntu") || lower.contains("id=debian") || lower.contains("debian") {
        "sudo apt install build-essential pkg-config libsdl3-dev vulkan-tools vulkan-validationlayers glslc libasound2-dev".to_string()
    } else if lower.contains("id=fedora") || lower.contains("fedora") {
        "sudo dnf install gcc pkgconf-pkg-config SDL3-devel vulkan-tools vulkan-validation-layers shaderc alsa-lib-devel".to_string()
    } else if lower.contains("id=arch") || lower.contains("arch") {
        "sudo pacman -S --needed base-devel pkgconf sdl3 vulkan-tools vulkan-validation-layers shaderc alsa-lib".to_string()
    } else if lower.contains("id=gentoo") || lower.contains("gentoo") {
        "sudo emerge --ask media-libs/libsdl3 media-libs/shaderc dev-util/vulkan-tools media-libs/alsa-lib".to_string()
    } else {
        "install SDL3 development files, vulkan-tools, validation layers, shaderc/glslc, and ALSA development files".to_string()
    }
}

fn executable_on_path(name: &str) -> bool {
    let Some(paths) = env::var_os("PATH") else {
        return false;
    };
    env::split_paths(&paths).any(|dir| {
        let plain = dir.join(name);
        if plain.is_file() {
            return true;
        }
        #[cfg(windows)]
        {
            return dir.join(format!("{name}.exe")).is_file();
        }
        #[cfg(not(windows))]
        false
    })
}

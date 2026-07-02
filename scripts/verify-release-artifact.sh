#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -lt 2 ] || [ "$#" -gt 3 ]; then
    echo "usage: scripts/verify-release-artifact.sh <archive.zip> <linux|windows> [demo|sdk]" >&2
    exit 2
fi

archive="$1"
platform="$2"
kind="${3:-demo}"

if [ ! -f "$archive" ]; then
    echo "archive not found: $archive" >&2
    exit 1
fi

case "$platform" in
    linux)
        executable="game"
        runtime_library="libSDL3.so.0"
        ;;
    windows)
        executable="game.exe"
        runtime_library="SDL3.dll"
        ;;
    *)
        echo "unknown platform '$platform'; expected linux or windows" >&2
        exit 2
        ;;
esac

entries="$(unzip -Z1 "$archive")"

require_entry() {
    local entry="$1"
    if ! printf '%s\n' "$entries" | grep -Fxq "$entry"; then
        echo "release archive '$archive' is missing '$entry'" >&2
        exit 1
    fi
}

reject_entry() {
    local entry="$1"
    if printf '%s\n' "$entries" | grep -Fxq "$entry"; then
        echo "release archive '$archive' must not contain '$entry'" >&2
        exit 1
    fi
}

case "$kind" in
    demo)
        require_entry "$executable"
        require_entry "$runtime_library"
        require_entry "run.sh"
        require_entry "run.ps1"
        require_entry "run.bat"
        require_entry "README.txt"
        require_entry "assets/fonts/DejaVuSans.ttf"
        require_entry "assets/game.ron"
        require_entry "assets/maps/level_1.txt"
        require_entry "assets/maps/tiled_demo.tmx"
        require_entry "assets/textures/test.png"
        require_entry "assets/sounds/hit.wav"
        ;;
    sdk)
        case "$platform" in
            linux)
                player="game-player"
                game_dev="game-dev"
                ;;
            windows)
                player="game-player.exe"
                game_dev="game-dev.exe"
                ;;
        esac
        require_entry "$player"
        require_entry "$game_dev"
        require_entry "$runtime_library"
        require_entry "run.sh"
        require_entry "run.ps1"
        require_entry "run.bat"
        require_entry "README.txt"
        require_entry "LICENSE"
        require_entry "THIRD_PARTY_NOTICES.md"
        require_entry "templates/no-rust-demo/game.toml"
        require_entry "templates/no-rust-demo/README.txt"
        require_entry "templates/no-rust-demo/assets/maps/level-1.txt"
        require_entry "templates/no-rust-demo/assets/fonts/DejaVuSans.ttf"
        require_entry "templates/no-rust-demo/assets/textures/player.png"
        require_entry "examples/no-rust-minimal/game.toml"
        require_entry "examples/no-rust-minimal/assets/fonts/DejaVuSans.ttf"
        require_entry "examples/no-rust-full/game.toml"
        require_entry "examples/no-rust-full/assets/fonts/DejaVuSans.ttf"
        require_entry "examples/no-rust-full/assets/animations/player.toml"
        reject_entry "templates/no-rust-demo/Cargo.toml"
        reject_entry "templates/no-rust-demo/build.rs"
        reject_entry "templates/no-rust-demo/src/main.rs"
        if ! unzip -p "$archive" README.txt | grep -Eiq 'No Rust|no Rust'; then
            echo "release archive '$archive' README.txt must say no Rust is required" >&2
            exit 1
        fi
        ;;
    *)
        echo "unknown artifact kind '$kind'; expected demo or sdk" >&2
        exit 2
        ;;
esac

echo "release archive '$archive' contains the expected $platform $kind package layout"

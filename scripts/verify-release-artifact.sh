#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 2 ]; then
    echo "usage: scripts/verify-release-artifact.sh <archive.zip> <linux|windows>" >&2
    exit 2
fi

archive="$1"
platform="$2"

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

echo "release archive '$archive' contains the expected $platform package layout"

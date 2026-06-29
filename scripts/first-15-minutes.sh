#!/usr/bin/env bash
set -euo pipefail

repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
workdir="${FIRST15_WORKDIR:-$(mktemp -d /tmp/game-first-15-XXXXXX)}"
game_dev="${GAME_DEV:-game-dev}"
features="${FIRST15_FEATURES:-}"
smoke_frames="${FIRST15_SMOKE_FRAMES:-60}"
timeout_duration="${FIRST15_TIMEOUT:-180s}"

if ! command -v cargo-generate >/dev/null 2>&1 && ! cargo generate --version >/dev/null 2>&1; then
    echo "cargo-generate is required. Install it with: cargo install cargo-generate" >&2
    exit 1
fi

if [ "${FIRST15_KEEP:-0}" != "1" ]; then
    trap 'rm -rf "$workdir"' EXIT
fi

mkdir -p "$workdir"

cargo generate --path "$repo/templates/simple-demo" \
    --name first-demo \
    --destination "$workdir" \
    --define title="First Demo" \
    --define "game_starter_dependency={ path = \"$repo/crates/game-starter\" }"

project="$workdir/first-demo"
cd "$project"
package_name="$(sed -n 's/^name = "\(.*\)"/\1/p' Cargo.toml | head -n 1)"

cargo_args=()
if [ -n "$features" ]; then
    cargo_args=(--features "$features")
fi

run_smoke() {
    if [ "${FIRST15_USE_XVFB:-0}" = "1" ]; then
        timeout -k 5s "$timeout_duration" xvfb-run -a --server-args="-screen 0 1280x720x24" "$@"
    elif command -v timeout >/dev/null 2>&1; then
        timeout -k 5s "$timeout_duration" "$@"
    else
        "$@"
    fi
}

cargo check "${cargo_args[@]}"

if [ "${FIRST15_SKIP_SMOKE:-0}" != "1" ]; then
    GAME_SMOKE_FRAMES="$smoke_frames" run_smoke cargo run "${cargo_args[@]}"
fi

printf '%s\n' \
    '##########' \
    '#P..C..E.#' \
    '#..C.....#' \
    '##########' \
    > assets/maps/level_1.txt

"$game_dev" asset-check
"$game_dev" package --release "${cargo_args[@]}" --out dist/first-demo --zip

if [ ! -f "dist/first-demo/$package_name" ] && [ ! -f "dist/first-demo/$package_name.exe" ]; then
    echo "packaged executable for $package_name was not found" >&2
    exit 1
fi
test -d dist/first-demo/assets
test -f dist/first-demo/README.txt
test -f dist/first-demo/run.sh
test -f dist/first-demo/run.ps1
test -f dist/first-demo/run.bat
test -f dist/first-demo.zip

if [ "${FIRST15_SKIP_SMOKE:-0}" != "1" ]; then
    GAME_SMOKE_FRAMES="$smoke_frames" run_smoke dist/first-demo/run.sh
fi

echo "first 15 minutes acceptance test passed at $project"

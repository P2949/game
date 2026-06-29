#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat >&2 <<'USAGE'
usage: scripts/verify-github-release-artifacts.sh [run-id|latest]

Downloads the Linux and Windows demo artifacts from a completed Release
Artifacts workflow run, then verifies their package layout.

Environment:
  GH_REPO             GitHub repo to inspect. Defaults to P2949/game.
  RELEASE_WORKFLOW   Workflow selector. Defaults to release.yml.
USAGE
}

if [ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]; then
    usage
    exit 0
fi

if [ "$#" -gt 1 ]; then
    usage
    exit 2
fi

repo="${GH_REPO:-P2949/game}"
workflow="${RELEASE_WORKFLOW:-release.yml}"
run_id="${1:-latest}"

if [ "$run_id" = "latest" ]; then
    run_id="$(
        gh run list \
            --repo "$repo" \
            --workflow "$workflow" \
            --status success \
            --limit 1 \
            --json databaseId \
            --jq '.[0].databaseId // empty'
    )"
    if [ -z "$run_id" ]; then
        echo "no successful '$workflow' runs found for $repo" >&2
        exit 1
    fi
fi

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"
tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

download_and_verify() {
    local artifact="$1"
    local platform="$2"
    local destination="$tmp_dir/$artifact"

    mkdir -p "$destination"
    gh run download "$run_id" \
        --repo "$repo" \
        --name "$artifact" \
        --dir "$destination"

    "$repo_root/scripts/verify-release-artifact.sh" \
        "$destination/$artifact.zip" \
        "$platform"
}

download_and_verify "game-demo-linux-x86_64" linux
download_and_verify "game-demo-windows-x86_64" windows

echo "GitHub release artifacts from run $run_id contain the expected package layouts"

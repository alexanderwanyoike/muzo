#!/usr/bin/env bash
set -euo pipefail

appimage=$(realpath -- "${1:?Pass the Muzo AppImage}")
profile=$(mktemp -d "${TMPDIR:-/tmp}/muzo-appimage-start.XXXXXXXX")
export XDG_CONFIG_HOME="$profile/config"
export XDG_DATA_HOME="$profile/data"
export XDG_CACHE_HOME="$profile/cache"
export XDG_RUNTIME_DIR="$profile/runtime"
mkdir -m 700 "$XDG_RUNTIME_DIR"
unset WAYLAND_DISPLAY
export APPIMAGE_EXTRACT_AND_RUN=1

setsid "$appimage" > "$profile/app.log" 2>&1 &
app=$!
cleanup() {
    kill -- "-$app" 2>/dev/null || true
    wait "$app" 2>/dev/null || true
    rm -rf -- "$profile"
}
trap cleanup EXIT

if ! timeout 30s xdotool search --sync --onlyvisible --limit 1 --name '^Muzo$' >/dev/null; then
    cat "$profile/app.log" >&2
    exit 1
fi
printf 'Opened the packaged Muzo window.\n'

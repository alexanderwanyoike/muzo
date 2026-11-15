#!/usr/bin/env bash
set -euo pipefail

appimage=$(realpath -- "${1:?Pass the Muzo AppImage}")
profile=$(mktemp -d "${TMPDIR:-/tmp}/muzo-appimage-start.XXXXXXXX")
trap 'rm -rf -- "$profile"' EXIT
export XDG_CONFIG_HOME="$profile/config"
export XDG_DATA_HOME="$profile/data"
export XDG_CACHE_HOME="$profile/cache"
export XDG_RUNTIME_DIR="$profile/runtime"
mkdir -m 700 "$XDG_RUNTIME_DIR"
unset DISPLAY WAYLAND_DISPLAY WAYLAND_SOCKET DBUS_SESSION_BUS_ADDRESS PULSE_SERVER
export APPIMAGE_EXTRACT_AND_RUN=1
export GVFS_DISABLE_FUSE=1

xvfb-run -a dbus-run-session -- bash -s -- "$appimage" "$profile/app.log" <<'RUN'
set -euo pipefail
setsid "$1" > "$2" 2>&1 &
app=$!
trap 'kill -- "-$app" 2>/dev/null || true; wait "$app" 2>/dev/null || true' EXIT

if ! timeout 30s xdotool search --sync --onlyvisible --limit 1 --name '^Muzo$' >/dev/null; then
    cat "$2" >&2
    exit 1
fi
printf 'Opened the packaged Muzo window.\n'
RUN

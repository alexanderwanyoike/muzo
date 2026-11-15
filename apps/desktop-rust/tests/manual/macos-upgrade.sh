#!/usr/bin/env bash
set -euo pipefail

legacy_app=$1
candidate_app=$2
fixture=$3
manual=$(cd -- "$(dirname -- "$0")" && pwd)
support="$HOME/Library/Application Support"
data="$support/muzo"
chromium="$support/Muzo"
leveldb="$chromium/Local Storage/leveldb"

if [[ -n ${MUZO_DATA_DIR:-} || -e "$data/muzo.sqlite" || -e "$data/muzo-rust.sqlite" || -e "$leveldb" ]]; then
  echo 'The profile migration test requires the default paths on a fresh runner.' >&2
  exit 1
fi

mkdir "$fixture"
installed="$fixture/Muzo.app"
preferences="$fixture/preferences.json"
baseline="$fixture/baseline.json"
music="$fixture/Music"
application=

cleanup() {
  if [[ -n "$application" ]]; then
    kill "$application" 2>/dev/null || true
    wait "$application" 2>/dev/null || true
  fi
}
trap cleanup EXIT

open_bundle() {
  local name
  name=$(/usr/libexec/PlistBuddy -c 'Print :CFBundleExecutable' "$installed/Contents/Info.plist")
  "$installed/Contents/MacOS/$name" "$@" &
  application=$!
}

close_bundle() {
  swift "$manual/macos-quit.swift" "$application"
  wait "$application"
  application=
}

ditto "$legacy_app" "$installed"
open_bundle --remote-debugging-port=0
node "$manual/seed-upgrade-profile.mjs" "$chromium" "$music" "$preferences"
close_bundle
node "$manual/profile-upgrade.mjs" capture "$data" "$leveldb" "$baseline" "$preferences" "$music/Tone.mp3"

mv "$installed" "$fixture/Previous Muzo.app"
ditto "$candidate_app" "$installed"
open_bundle
node "$manual/profile-upgrade.mjs" verify "$data" "$leveldb" "$baseline"
close_bundle

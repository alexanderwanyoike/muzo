#!/usr/bin/env bash
set -euo pipefail

target=$1
candidate_dmg=$2
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

case "$target" in
  macos-arm64) legacy_sha256=dab8ba61cba09174fa57cebc3bbe1eff8bd0a9c21416c737b3b76c295aca2cbf ;;
  macos-x64) legacy_sha256=7188c61ed2383527ff04167cf400a7fd0a2f6fba0cefbb979f3d882d4f5770c9 ;;
  *) echo "Unknown macOS fixture target $target" >&2; exit 1 ;;
esac
legacy_asset="Muzo-0.0.6-$target.dmg"
legacy_image="$fixture/$legacy_asset"
legacy_mount="$fixture/legacy"
candidate_mount="$fixture/candidate"
mkdir -p "$legacy_mount" "$candidate_mount"
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
  hdiutil detach "$candidate_mount" -force -quiet || true
  hdiutil detach "$legacy_mount" -force -quiet || true
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

curl --fail --silent --show-error --location \
  "https://github.com/alexanderwanyoike/muzo/releases/download/v0.0.6/$legacy_asset" \
  --output "$legacy_image"
printf '%s  %s\n' "$legacy_sha256" "$legacy_image" | shasum -a 256 -c -
hdiutil attach "$legacy_image" -readonly -nobrowse -noautoopen -mountpoint "$legacy_mount"
hdiutil attach "$candidate_dmg" -readonly -nobrowse -noautoopen -mountpoint "$candidate_mount"

ditto "$legacy_mount/Muzo.app" "$installed"
open_bundle --remote-debugging-port=0
node "$manual/seed-upgrade-profile.mjs" "$chromium" "$music" "$preferences"
close_bundle
node "$manual/profile-upgrade.mjs" capture "$data" "$leveldb" "$baseline" "$preferences" "$music/Tone.mp3"

mv "$installed" "$fixture/Previous Muzo.app"
ditto "$candidate_mount/Muzo.app" "$installed"
open_bundle
node "$manual/profile-upgrade.mjs" verify "$data" "$leveldb" "$baseline"
close_bundle

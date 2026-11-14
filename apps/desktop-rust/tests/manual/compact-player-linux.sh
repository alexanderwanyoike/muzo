set -euo pipefail
cd "$1"
export XDG_CONFIG_HOME="$PWD/config" XDG_DATA_HOME="$PWD/data" XDG_CACHE_HOME="$PWD/cache"
metacity --sm-disable --replace >wm.log 2>&1 &
wm=$!
sleep .3
"$2" "$PWD/Silt and Light.mp3" "$PWD/Tideland.mp3" "$PWD/Floodplain.mp3" >app.log 2>&1 &
app=$!
trap 'kill "$app" "$wm" 2>/dev/null || true' EXIT
visible() { timeout 12 xdotool search --sync --onlyvisible --name "$1" | head -1; }
geometry() { xdotool getwindowgeometry --shell "$1"; }
capture() {
    xdotool mousemove 1240 760
    sleep .65
    import -window "$1" -frame -trim +repage "$2"
}
library=$(visible '^Muzo$')
sleep .7
wmctrl -ir "$(printf '0x%x' "$library")" -e 0,190,105,-1,-1
sleep .5
geometry "$library" > library-before
capture "$library" "$3/rust-library.png"
xdotool mousemove --window "$library" 813 38 click 1
mini=$(visible 'Mini player')
sleep .5
geometry "$mini" > mini-before
capture "$mini" "$3/rust-mini-player.png"
xdotool mousemove --window "$mini" 395 29 click 1
sleep 1.3
geometry "$mini" > mini-queue
capture "$mini" "$3/rust-mini-queue.png"
xdotool mousemove --window "$mini" 40 314 click 1
sleep .3
xdotool mousemove --window "$mini" 272 145 click 1
sleep .5
capture "$mini" "$3/rust-mini-player-seek.png"
xdotool mousemove --window "$mini" 30 29 click 1
sleep .5
geometry "$library" > library-after
capture "$library" "$3/rust-library-restored.png"
diff -u library-before library-after
test "$(sed -n 's/^HEIGHT=//p' mini-queue)" -gt "$(sed -n 's/^HEIGHT=//p' mini-before)"

set -euo pipefail
cd "$1"
export XDG_CONFIG_HOME="$PWD/config" XDG_DATA_HOME="$PWD/data" XDG_CACHE_HOME="$PWD/cache"
metacity --sm-disable --replace >wm.log 2>&1 &
wm=$!
sleep .3
"$2" "$PWD/Silt and Light.mp3" "$PWD/Tideland.mp3" "$PWD/Floodplain.mp3" >app.log 2>&1 &
app=$!
trap 'kill "$app" "$wm" 2>/dev/null || true' EXIT
geometry() { xdotool getwindowgeometry --shell "$1"; }
capture() {
    xdotool mousemove 1240 760
    sleep .65
    import -window "$1" -frame -trim +repage "$2"
}
active_mini() {
    for attempt in {1..20}; do
        local active
        active=$(xdotool getactivewindow)
        if [[ "$active" != "$library" ]] && [[ "$(xdotool getwindowname "$active")" == Muzo ]]; then
            printf '%s\n' "$active"
            return
        fi
        sleep .1
    done
    return 1
}
library=$(timeout 12 xdotool search --sync --onlyvisible --name '^Muzo$' | head -1)
sleep .7
wmctrl -ir "$(printf '0x%x' "$library")" -e 0,190,105,-1,-1
sleep .5
geometry "$library" > library-before
capture "$library" "$3/rust-library.png"
xdotool windowactivate --sync "$library" key ctrl+m
mini=$(active_mini)
capture "$mini" "$3/rust-mini-player.png"
geometry "$mini" > mini-before
xdotool mousemove --window "$mini" 355 25 click 1
sleep 1
geometry "$mini" > mini-queue
capture "$mini" "$3/rust-mini-queue.png"
xdotool mousemove --window "$mini" 88 188 click 1
sleep .3
xdotool mousemove --window "$mini" 467 414 click 1
capture "$mini" "$3/rust-mini-player-seek.png"
xdotool key Escape
sleep .5
test "$(xdotool getactivewindow)" = "$library"
geometry "$library" > library-after
capture "$library" "$3/rust-library-restored.png"
diff -u library-before library-after
test "$(sed -n 's/^WIDTH=//p' mini-before)" = 420
test "$(sed -n 's/^WIDTH=//p' mini-queue)" = 740
test "$(sed -n 's/^HEIGHT=//p' mini-before)" = 560
test "$(sed -n 's/^HEIGHT=//p' mini-queue)" = 560

- Sprint: 02
- Type: Fix
- Status: In progress
- Depends on: 032 (merged)

# Card 033 - Crisp Linux panel icon

The user reported a distorted waveform icon in Linux Mint Cinnamon. Preserve the
approved large blue/black mark and ship a pixel-aligned small window variant.

## Acceptance criteria

- [ ] Keep the nine waveform bars distinct at 24px and 32px panel sizes.
- [ ] Verify the actual X11 _NET_WM_ICON exported by an Electron window.
- [ ] Preserve the high-resolution launcher, Windows ICO and macOS ICNS artwork.
- [ ] Make the small PNG reproducible from code through the existing icon command.
- [ ] Complete ordered gates and two review cycles before merging into dev.

## Diagnosis and established patterns

A native Xvfb/Electron probe found only a 1254px image advertised to X11. Scaling
it to 16/24/32px merged all nine bars into one blue run at the common centreline.
Adding NativeImage scale representations still advertised only the 1x image.
Cinnamon's window-backed icon path scales the window icon to the panel size:
https://github.com/linuxmint/cinnamon/blob/master/src/cinnamon-app.c
Electron's Linux window implementation delegates its ImageSkia to SetWindowIcons:
https://github.com/electron/electron/blob/v43.0.0/shell/browser/native_window_views.cc
Use a dedicated 48px raster aligned to the 24px pixel grid, a standard small-icon
hinting pattern, and Electron's existing nativeImage bitmap/PNG export API. No
external implementation is copied and no new dependency is required.

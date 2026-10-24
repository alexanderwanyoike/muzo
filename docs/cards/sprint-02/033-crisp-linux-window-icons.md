- Sprint: 02
- Type: Fix
- Status: In review
- Depends on: 032 (merged)

# Card 033 - Crisp Linux panel icon

The user reported a distorted waveform icon in Linux Mint Cinnamon. Preserve the
approved large blue/black mark and ship a pixel-aligned small window variant.

## Acceptance criteria

- [x] Keep the nine waveform bars distinct at 24px and 32px panel sizes.
- [x] Verify the actual X11 _NET_WM_ICON exported by an Electron window.
- [x] Preserve the high-resolution launcher, Windows ICO and macOS ICNS artwork.
- [x] Make the small PNG reproducible from code through the existing icon command.
- [x] Complete ordered gates and two review cycles before merging into dev.

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

## Verification

The regression reads the full X11 property rather than xprop's truncated default
output. With the original icon it observes 1254x1254 and one merged blue run at
24/32px. With the fixed icon it observes 48x48 and nine distinct runs at both
sizes. Original PNG/ICO/ICNS exports are byte-identical. All ordered gates pass
with 329 tests, and icon regeneration has zero diff. Linux CI installs x11-utils
and checks actual window pixels; Windows/macOS CI also checks reproducibility.
The initial Standards review found one test-variable naming issue, corrected
before re-review. Both final axes have zero findings.
See [review record](../../reviews/2026-10-24-crisp-linux-window-icons.md).

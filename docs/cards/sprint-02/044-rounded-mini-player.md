---
status: in-progress
---

# Card 044 - Round the native mini player edges

Keep the same BrowserWindow, playback engine and native window controls while
adding curved compact-player edges where the native platform supports shaping.

## Acceptance criteria

- [x] Windows compact modes use a rounded native drawing/input region.
- [x] X11 compact modes round the native client bottom corners, preserving the
  straight join to the separately drawn native title bar and its own top corners.
- [x] Shape follows solo/queue changes and manual resize without clipping controls.
- [x] Library mode restores its original bounds, minimum size and maximized state.
- [x] Library mode has a full rectangular drawing/input region and remains resizable.
- [x] Native errors use Result rollback; resize errors try a rectangular fallback.
- [x] Resize listeners are removed when the window closes.
- [x] The renderer follows the native bottom curve for every material.
- [x] macOS and native Wayland retain compositor-provided corner treatment.
- [ ] Ordered gates, two review cycles and PR handoff complete.

## Established pattern and platform constraints

Uses Electron 43's existing `BrowserWindow.setShape(Rectangle[])` API, with a
small circle-scanline region and a named 14-DIP radius. The API controls both
native drawing and effective mouse input. It is experimental on Windows/Linux.
The standard native frame is retained; no transparent or second window, new
state bridge or frameless title bar is needed. macOS has no setShape support.
Native Wayland shaping is not promised and follows the desktop compositor.

Electron's `roundedCorners` constructor option applies primarily to frameless
windows, so changing the whole window architecture for that option is avoided.
The local installed `electron.d.ts` and official docs informed the implementation:

- https://www.electronjs.org/docs/latest/api/browser-window#winsetshaperects-windows-linux-experimental
- https://www.electronjs.org/docs/latest/api/base-window

## Verification

Atomic red/green tests cover region corners/centre, resize, restore, close cleanup,
unsupported platforms and failed-region rollback. Actual Electron X11 tests check
native shape extents at all 16 aluminium/glass light/dark default/minimum compact
sizes, alongside the existing real preload, wheel, Tab, Enter and contrast checks.

A native regression exposed an Electron 43/X11 discrepancy: `setShape([])` leaves
an empty 0x0 region here, rather than restoring the rectangle documented by the
API. Linux therefore restores an explicit full rectangle and synchronizes it
with later library resizes. Windows uses the documented empty-list reset.

Manual verification uses a disposable Xvfb display and Metacity, queries actual
XShape rectangles through libXext, and captures the root desktop with the native
frame. The compact region excludes both bottom corners and includes the centre;
the restored library region is one full rectangle. Root captures contain safe,
original fictional artwork and are not postprocessed to simulate corner clipping.

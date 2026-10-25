# 048: Refine mini player window chrome

## Request

Remove the mini player's added curved window ends. Move its entry control from
the sidebar into the large player's top-right corner, below native Close and
above Volume. The control is icon only.

## Acceptance criteria

- [x] Remove custom native window shaping and renderer bottom-corner clipping.
- [x] Keep native controls, resizing and library bounds/maximized-state restoration.
- [x] Place the icon-only toggle at the top-right of the playback bar, above volume.
- [x] Preserve tooltip, accessible name, pending state, playback continuity and focus return.
- [x] Check native positioning at normal/minimum widths in light and dark materials.
- [x] Complete ordered gates and both review cycles.

## Verification

Red/green unit and native Electron regressions cover rectangular client windows,
zero renderer corner radius, top-right button geometry and playback/focus continuity.
The native fixture checks eight library layouts and sixteen compact/queue layouts.
Native compositor-provided title bar corners remain under OS control; Muzo no
longer installs its own shape region. Uses existing Electron BrowserWindow sizing
and the existing Lucide Minimize2 icon, with no new dependency or copied assets.

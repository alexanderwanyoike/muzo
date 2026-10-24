- Sprint: 02
- Type: Feature
- Status: Done
- Depends on: 020 (PR #47)

# Card 021 - Polish the desktop music workspace

Give Muzo a restrained desktop music-player layout informed by iTunes: persistent
library sidebar, compact top transport, dense music rows and scrollable preferences
inside the window. Preserve playback, queue, metadata and library workflows.

## Acceptance criteria

- [x] Studio, Brushed aluminium and Warm paper materials, each in light and dark.
- [x] Independent Cobalt, Copper, Sage, Plum and Amber accents.
- [x] Persist validated preferences across launches and apply them before rendering.
- [x] Recover safely from invalid or unavailable preference storage, with visible feedback.
- [x] Keyboard-operable labelled material, mode and accent controls with visible focus.
- [x] Cohesive surfaces, controls, typography, icons and states across the app.
- [x] Fixed top transport, compact sidebar and full-row source selection.
- [x] Keep the desktop arrangement at the 800 by 600 minimum window size.
- [x] Full repository gates and both review cycles pass (246 tests, Linux zip/deb).

Source removal and real embedded album artwork are separate functional cards 022
and 023. The transport reserves an artwork slot in this visual change.

## Established patterns and licences

Use the independent appearance/accent semantic-token pattern documented by Radix
Themes (MIT): https://www.radix-ui.com/themes/docs/theme/color
Use the pre-render HTML attribute, color-scheme and localStorage pattern from
next-themes (MIT): https://github.com/pacocoursey/next-themes/blob/main/next-themes/src/script.ts
No implementation code is copied and no theme framework is needed for this app.

Manrope Variable 5.3.0 is bundled for offline use under OFL-1.1. Lucide React 1.61.0
provides consistent desktop control icons under ISC. Full font and icon licences
ship in public/licenses, which Vite copies into the packaged renderer.
Brushed aluminium uses a restrained CSS grain rather than image downloads.

## Verification

User-facing tests cover selecting material/mode/accent, restoring them after
remounting, malformed storage, failed saving, and selecting a source by its name.
Existing playback, queue, playlist, metadata and Dropbox tests remain in place.
Visual checks exercise the rendered app through controlled runtime fixtures, with
no changes to real files or Dropbox data. Verify all six material/mode combinations,
accent contrast, minimum desktop width and packaged offline assets.

Review evidence: [two review cycles](../../reviews/2026-10-24-desktop-appearance-themes.md).

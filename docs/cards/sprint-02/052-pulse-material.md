# 052: Add the Pulse material

## Background

The user requested a Spotify-like material. Pulse adds flat, layered charcoal surfaces in dark mode and a neutral light variant while retaining Muzo's desktop layout and identity. Glowing accents are a separate change and PR.

## Acceptance criteria

- [x] Offer Pulse alongside the existing materials, with light and dark variants.
- [x] Preserve the selected accent when changing material and restore preferences across restarts.
- [x] Use the same material in the full player, Settings and mini player without adding a box behind the compact timeline.
- [x] Verify text contrast and Settings alignment across all materials, both modes and minimum/desktop widths.
- [x] Verify native mini-player controls, queue access and window restoration in both Pulse modes.
- [x] Complete the ordered repository gates and deliver a focused PR to dev.

## Design and implementation

Extend Muzo's existing semantic CSS surface tokens and appearance validation. The separation of colour roles from theme-specific CSS follows the established [Spicetify theme structure](https://spicetify.app/docs/development/themes). Spicetify CLI is LGPL-2.1; no implementation, assets or styles were copied, and no dependency is added.

Pulse uses charcoal/black raised surfaces with softened panel corners. The light variant uses warm-neutral greys with the same hierarchy. Accent choice remains independent. No Spotify assets or branding are used.

## Verification

The initial persistence tests failed because Pulse was unavailable. They pass after registration. A separate native regression exposed an opaque timeline box in the mini player; scoping the full-player display rule fixes it. Both failing tests were committed separately from implementation. Existing Settings checks cover text contrast, distinct material colours, equal section widths, full-width source entries and overflow at 800 and 1280 pixels.

## Interaction reference

The user requested correcting hover and focus against Spotify itself. The live [Spotify web player](https://open.spotify.com/) was inspected directly: its Home surface changes from #1f1f1f to #2a2a2a on pointer hover, and keyboard focus uses a pale ring. Pulse uses neutral hover and selection fills, muted inactive navigation, a white focus ring in dark mode and a dark ring in light mode. Primary source actions retain their accent styling.

A native regression now uses real mouse movement and Tab input to check inactive, selected, hovered and keyboard-focused navigation in both modes at desktop and minimum widths. It failed before the interaction rules and passes after them. Screenshots were inspected after settling theme transitions.

Initial review found missing destructive-hover coverage and clipped sidebar focus rings. A native regression reproduced both with real pointer and Tab events, including the source Remove action, its confirmation dialog, the source row and sidebar footer. The correction preserves danger colours and the components' existing inset outline offsets. The regression now passes.

Final local checks passed sequentially with one test worker: install, lint, strict types, 430 tests, desktop build, package, make and formatting. Initial review and fix re-review are recorded on the PR. No release version or existing preference is changed.

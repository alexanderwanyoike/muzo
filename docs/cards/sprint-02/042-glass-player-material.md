---
status: in-review
---

# Card 042 - Add frosted glass and refine the queue scrollbar

The compact player needs a softer glass material, and the wide native queue
scrollbar distracts from the artwork. Add a seventh material while retaining all
existing materials, accents and saved preferences.

## Acceptance criteria

- [x] Frosted glass is selectable and persists in light/dark with all ten accents.
- [x] Renderer surfaces use translucent layers and blurred artwork with legible text.
- [x] Opaque semantic backgrounds remain the fallback without backdrop filtering.
- [x] Reduced transparency uses opaque surfaces and removes artwork atmosphere.
- [x] Queue scrollbar is thin with a transparent track and a clearer hover/focus thumb.
- [x] Native wheel and Tab navigation reach the last queued track; Enter selects it.
- [x] Default/minimum native compact sizes fit in both modes with/without the queue.
- [x] Settings layout/contrast covers 56 scenarios across seven materials.
- [x] Ordered gates, two review cycles and PR handoff complete.

## Design and verification

Uses the established CSS backdrop-filter translucent-surface pattern and native
CSS scrollbar-width/scrollbar-color styling. No custom scrolling engine,
Electron transparent window or platform desktop vibrancy is introduced.

- https://developer.mozilla.org/en-US/docs/Web/CSS/backdrop-filter
- https://developer.mozilla.org/en-US/docs/Web/CSS/scrollbar-width

Persistence tests were written and run red before adding the material, then run
green. The styling pass uses real Electron captures and existing native tests.
Native compact verification checks computed glass surface colours composited
over both black and white, conservatively covering arbitrary artwork backdrops,
with normal and muted text contrast at least 4.5:1. It also checks actual wheel,
Tab and Enter input through Chromium with the real preload and window bridge.
Safe original fictional artwork and disposable profiles are used for captures.

Depends on cards 040 and 041 (PRs #67 and #68), which include #63/#65 ancestry.

All ordered gates passed on 5087094 with 350 tests, build/package and Linux
ZIP/DEB make. The initial Spec review found a translucent queue fallback without
backdrop support. Regression ec325db failed at alpha 224; fix 5087094 restores
opaque alpha 255. Final Standards and Spec reviews returned zero findings.
After a host restart, both reviewers freshly audited this final pin with zero
findings. See [the review record](../../reviews/2026-10-24-glass-player-material.md).

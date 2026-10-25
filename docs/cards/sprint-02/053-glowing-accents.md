# 053: Add glowing accent variations

## Background

The user explicitly requested high-saturation glowing accent variations for personalization. Add six optional colours independently of the Pulse material in card 052. The existing muted accents remain available and remain the default.

## Acceptance criteria

- [x] Offer Glow green, cyan, blue, violet, pink and orange in a labelled Glow group beside Classic accents.
- [x] Persist every choice in both light and dark mode without changing the material or losing existing preferences.
- [x] Use a bright fill with a readable foreground for playback controls, plus static glow on playback, progress, volume and active navigation icons.
- [x] Use separate readable accent ink for text, icons, input controls and focus indicators on light surfaces.
- [x] Restore the existing effects when switching back to a classic accent.
- [x] Disable glow under forced-colour preferences and add no glow animation.
- [x] Verify contrast and actual rendered playback controls across all existing materials, both modes and full/compact player styles.
- [ ] Complete two review cycles, the ordered repository gates and a focused PR to dev.

## Implementation

Extend the existing appearance registry, typed validation and Result-based preference storage; the v1 preference format remains compatible. Classic and Glow share one radio group and an explicit visual grouping. Glow does not require Pulse or change material selection.

The existing CSS-token approach and the established [Spicetify theme structure](https://spicetify.app/docs/development/themes) inform the separation of palette roles from scoped visual effects. Spicetify CLI is LGPL-2.1; no source, assets or styles are copied and no dependency is added.

Bright accent fills and darker control text remain distinct from readable inline accent ink. Dark-mode ink uses the brighter palette variant. Shadows are static and limited to controls and swatches; track text and album artwork keep their existing treatment.

## Verification

Twelve persistence cases and the native palette regression failed before implementation. They pass after adding the registry, grouped controls and colour roles. Settings checks verify all sixteen colours, glow previews, foreground contrast, alignment and overflow across all existing materials and modes.

A separate native test renders the production playback component in full and compact contexts, checks all six glow fills against their glyph colours across every material and mode, verifies classic effect restoration and emulates forced colours. This test is included in Windows/macOS native CI as well as the Linux suite. Local validation uses a single worker after the user's memory-pressure restart.

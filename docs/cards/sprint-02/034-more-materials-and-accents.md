- Sprint: 02
- Type: Feature
- Status: In review
- Depends on: 033 (PR #60)

# Card 034 - More materials and accents

The user requested more personalization while preparing the first release.
Extend the restrained desktop material language with carbon fibre, walnut and
porcelain, each with light/dark variants. Add teal, rose, slate, violet and ice
accents, for six materials and ten accents in total.

## Acceptance criteria

- [x] Select and persist each new material and accent through Settings/restart.
- [x] Supply all material surface/text tokens in both light and dark modes.
- [x] Keep appearance controls usable at 800x600 with a wide sidebar.
- [x] Verify actual selected material/mode in native layout coverage.
- [x] Check normal/muted text and accent-label contrast across all materials/modes/accents.
- [x] Preserve existing saved preferences and defaults.
- [x] Complete ordered gates, browser inspection and two review cycles.

## Pattern and scope

Extend the existing data-theme/data-mode/data-accent CSS token system, radio
controls and persistent Appearance value. Materials use subtle CSS grain/weave
and ceramic highlights, with no new images or dependencies. Keep the six previews
in the same consistent Settings section and wrap the larger accent palette.
Use WCAG relative luminance/contrast guidance for text and accent labels:
https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html

## Verification

Red-first UI tests require persistence across restart for each of the three new
materials and five new accents. Native coverage now exercises 48 layouts, checks
the actual selected material/mode, distinct material surfaces and accent colours,
normal/muted text on all four surfaces and labels on all ten accent fills at
4.5:1 or better. Two existing light muted colours were slightly darkened after
this check exposed insufficient sidebar/chrome contrast.

Browser checks inspected Walnut dark/Teal, Carbon fibre dark/Violet at 800x600
with the widest sidebar, and Porcelain light/Rose. Ten swatches occupy 328px and
cause no horizontal overflow. The six previews remain aligned in Settings.
All ordered gates pass with 337 tests; both review cycles have zero findings.
See [review record](../../reviews/2026-10-24-more-materials-and-accents.md).

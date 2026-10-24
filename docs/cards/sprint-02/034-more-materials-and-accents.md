- Sprint: 02
- Type: Feature
- Status: In progress
- Depends on: 033 (PR #60)

# Card 034 - More materials and accents

The user requested more personalization while preparing the first release.
Extend the restrained desktop material language with carbon fibre, walnut and
porcelain, each with light/dark variants. Add teal, rose, slate, violet and ice
accents, for six materials and ten accents in total.

## Acceptance criteria

- [ ] Select and persist each new material and accent through Settings/restart.
- [ ] Supply all material surface/text tokens in both light and dark modes.
- [ ] Keep appearance controls usable at 800x600 with a wide sidebar.
- [ ] Verify actual selected material/mode in native layout coverage.
- [ ] Check normal/muted text and accent-label contrast across all materials/modes/accents.
- [ ] Preserve existing saved preferences and defaults.
- [ ] Complete ordered gates, browser inspection and two review cycles.

## Pattern and scope

Extend the existing data-theme/data-mode/data-accent CSS token system, radio
controls and persistent Appearance value. Materials use subtle CSS grain/weave
and ceramic highlights, with no new images or dependencies. Keep the six previews
in the same consistent Settings section and wrap the larger accent palette.
Use WCAG relative luminance/contrast guidance for text and accent labels:
https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html

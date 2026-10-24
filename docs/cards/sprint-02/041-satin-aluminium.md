- Sprint: 02
- Type: Visual polish
- Status: In review
- Depends on: 035 (#63), 038 (#65)

# Card 041 - Make aluminium feel like satin metal

The user said the current brushed aluminium material does not feel like MacBook
metal. Its regular horizontal stripes look more like ribbed chrome than a quiet
satin surface. Refine this one material, retaining its saved identity and choices.

## Acceptance criteria

- [x] Replace visible directional stripes with very fine, non-directional matte grain.
- [x] Use cool silver light surfaces and refined space-gray dark surfaces with soft highlights and borders.
- [x] Keep the existing aluminium ID/name, six materials, ten accents and saved preferences.
- [x] Let the existing appearance sample use the same material tokens as the player.
- [x] Verify both variants with native before/after captures and the existing 48 Settings contrast/layout scenarios.
- [x] Complete ordered repository checks and both review cycles, then deliver a feature PR to dev.

This is styling and descriptive copy only. No application logic, theme API,
preference schema or runtime dependency changes, and no artificial TDD tests.
The renderer uses one authored monochrome SVG fractal-noise tile through a local
CSS data URI and a broad, low-opacity satin highlight. The small static tile
repeats without directional banding or remote image requests. Its mechanism follows
the standard SVG feTurbulence texture pattern documented by MDN:
https://developer.mozilla.org/en-US/docs/Web/SVG/Reference/Element/feTurbulence
No third-party texture, implementation or Apple artwork is copied.

Native captures use the maintained Settings fixture, a disposable profile and
isolated Xvfb display. The user's library, credentials and running player remain
untouched. The current semantic text/accent contrast and layout checks remain
the validation boundary rather than tests that mirror CSS declarations.

All ordered checks pass with 338 tests in 78 files, including all 48 native
Settings contrast/layout scenarios. Both review cycles have zero findings on
both axes. See the [review record](../../reviews/2026-10-24-anodised-aluminium.md).

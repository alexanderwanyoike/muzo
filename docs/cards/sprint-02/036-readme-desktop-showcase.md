- Sprint: 02
- Type: Documentation
- Status: In review

# Card 036 - Show the current desktop player in the README

The user requested a README screenshot because the player has changed substantially.

## Acceptance criteria

- [x] Put a clear, current listening screenshot near the top of the README.
- [x] Show the actual interface, artwork, transport and Sources panel using safe demo data.
- [x] Refresh status and feature copy, with honest source installation availability.
- [x] Share canonical screenshots with the forthcoming project site and record provenance.
- [x] Complete ordered checks and two review cycles, then open a feature PR against dev.

This documentation and screenshot change introduces no production logic, so it
requires direct content and visual verification rather than artificial TDD tests.
The branch starts at dev and includes pending PR #63 as a prerequisite; merge #63
first to preserve its merge policy and release ancestry documentation.

All ordered gates pass, including 337 desktop tests and Linux ZIP/DEB packaging.
Both review cycles have zero findings on both axes. See the
[review record](../../reviews/2026-10-24-readme-desktop-showcase.md).

Screenshot refresh: depends on the seek-focus fix (Card 038) before integration.
The current capture uses that fix and lives in the future Vite public/ asset
directory with its native .jpg extension. Source art remains original PNG demo
art, while the viewport capture itself is JPEG. Application checks now total 338
tests on the included fix; the original 337-test baseline above is historical.

## Current integrated capture refresh

The native Electron captures now reflect the combined preview of cards 040, 041,
042 and 045: mini-player control, satin aluminium, seven materials including
Frosted glass and source rows that reach the panel edge. The listening image is
1440 x 900 Glass dark/Cobalt; Settings is 1280 x 1000 Aluminium light/Rose.
The canonical files remain shared with the site; no image retouching is used.
The application changes stay in their own PRs, which must merge before Site #66.

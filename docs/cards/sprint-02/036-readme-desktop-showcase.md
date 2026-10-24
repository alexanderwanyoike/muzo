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

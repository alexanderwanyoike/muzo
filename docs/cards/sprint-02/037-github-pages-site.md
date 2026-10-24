- Sprint: 02
- Type: Website
- Status: In review

# Card 037 - Introduce Muzo on GitHub Pages

The user proposed a project site similar to sibling Vibez and Showbiz. Prepare a
reviewable static site and deployment workflow, using the README's current captures.

## Acceptance criteria

- [x] Create a restrained, responsive, screenshot-led Muzo landing page with the existing blue waveform identity.
- [x] Explain actual local/Dropbox libraries, playlists, cached artwork and personalization without future-feature claims.
- [x] Use six material samples and explain light/dark variants and ten accents.
- [x] Link source installation and release history honestly; no invented binary download URLs.
- [x] Self-host screenshots, the existing icon and Manrope font with its licence.
- [x] Verify assets, navigation, metadata and installation availability through meaningful red-first tests.
- [x] Enforce formatting, lint, strict types and site verification in CI; deploy only main through official Pages actions.
- [x] Preview desktop/mobile and keyboard navigation; run ordered repository gates and both review cycles.

The static HTML/CSS site follows the established sibling site/ layout and GitHub's
official Pages configure/upload/deploy pattern. Vite, already in this repo, builds
project-relative assets. There is no client JavaScript or custom release API.
Published releases can be reached through GitHub's release history; source setup
is the primary action until binary assets are actually published.

Merge order: #63, #65 (seek focus), #64 (captures), then this site PR. No dev/main merge or public Pages
configuration is performed in this feature PR. Enable Pages with GitHub Actions
and promote through a release PR when the site is approved.

All ordered local gates pass with 338 desktop tests and five site checks.
The built site has no horizontal overflow at 320, 390, 768 and 1440 pixels.
Both review cycles report zero findings on both axes. See the
[review record](../../reviews/2026-10-24-github-pages-site.md).

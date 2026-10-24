- Sprint: 02
- Type: Website
- Status: In review

# Card 037 - Introduce Muzo on GitHub Pages

The user proposed a project site similar to sibling Vibez and Showbiz. Prepare a
reviewable static site and deployment workflow, using the README's current captures.

## Acceptance criteria

- [x] Create a restrained, responsive, screenshot-led Muzo landing page with the existing blue waveform identity.
- [x] Explain actual local/Dropbox libraries, playlists, cached artwork and personalization without future-feature claims.
- [x] Use seven material samples and explain light/dark variants and ten accents.
- [x] Offer native Windows installers, macOS Apple silicon/Intel DMGs and Linux AppImage through verified published releases, with no source-build installation flow.
- [x] Self-host screenshots, the existing icon and Manrope font with its licence.
- [x] Verify assets, navigation, metadata and installation availability through meaningful red-first tests.
- [x] Enforce formatting, lint, strict types and site verification in CI; deploy only main through official Pages actions.
- [x] Preview desktop/mobile and keyboard navigation; run ordered repository gates and both review cycles.

The static HTML/CSS site follows the established sibling site/ layout and GitHub's
official Pages configure/upload/deploy pattern. Vite, already in this repo, builds
project-relative assets. There is no client JavaScript. Build-time release validation consumes GitHub's official releases API and the shared distribution manifest.
Published stable releases supply all download URLs. Local previews show disabled
Preparing release controls until complete real artifacts exist. Production builds
require the manifest and fail closed for missing or inconsistent assets.

Merge order: #63, #65 (seek focus), #64 (captures), then this site PR. No dev/main merge or public Pages
configuration is performed in this feature PR. Enable Pages with GitHub Actions
and promote through a release PR when the site is approved.

The original site passed 338 desktop tests and five site checks. Current
integrated verification includes the native player/distribution and 12 site tests.
The built site has no horizontal overflow at 320, 390, 768 and 1440 pixels.
The original review cycles reported zero findings on both axes. See the
[review record](../../reviews/2026-10-24-github-pages-site.md).

The user rejected the initial source-installation fallback and requested the
normal released-download experience used by Showbiz and Jolt. This PR now consumes
the shared Card 039 distribution contract and will depend on its packaging PR.
The initial review records below describe the earlier iteration; renewed review
cycles cover the corrected download and publication behavior.

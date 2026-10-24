# Muzo project site

The static HTML/CSS site lives in `site/`. Its quiet, screenshot-led design uses
the actual listening interface, approved blue waveform icon and local Manrope
font. The font's SIL Open Font License is retained in `site/public/fonts/OFL.txt`.
See [screenshots.md](./screenshots.md) for capture provenance. Material swatches
are illustrative; the desktop captures are real.

The site follows Showbiz and Jolt's released-download pattern. Visitors choose
Windows installer, macOS DMG (Apple silicon or Intel), or Linux AppImage. Developer
checkout instructions belong in the repository README, outside the product's
installation flow. The site has no client JavaScript, analytics or remote fonts.

## Local checks and preview

```sh
yarn install
yarn site:verify
yarn site:build
yarn site:preview
```

Open the URL printed by Vite, including `/muzo/`. Without release metadata, local
previews show disabled Preparing release controls. They do not fabricate download
URLs. `yarn workspace @muzo/site dev` also serves this pending state.

The root format, lint, strict typecheck and test commands include site tooling.
Tests exercise the real production build with synthetic release metadata, verify
complete installer links and checksums, and reject missing metadata, draft or
prerelease releases, ambiguous filenames, foreign URLs and size/digest mismatches.
Public screenshots, icon and font keep stable URLs under the project base.

## Published download contract

The private `@muzo/distribution` build-tool workspace is shared with native release
assembly. Its versioned manifest identifies the exact source commit and the four
supported installers, including their filenames, sizes and SHA-256 hashes.

The Pages workflow reads the latest published stable GitHub release and downloads
its `muzo-X.Y.Z-release.json`. The build checks that tag/version match and every
installer, the manifest and `SHA256SUMS` are present. Installer sizes must match;
GitHub-provided SHA-256 digests must also match when available. URLs must point to
that exact release in the Muzo repository. A failed check stops deployment.

Production builds use these environment variables:

- `MUZO_REQUIRE_RELEASE=1`
- `MUZO_GITHUB_RELEASE_JSON`: GitHub releases API JSON file
- `MUZO_DISTRIBUTION_MANIFEST`: distribution manifest JSON file

Release metadata is injected through Vite's official `transformIndexHtml` hook:
https://v5.vite.dev/guide/api-plugin#transformindexhtml
No network call or release parsing runs in the visitor's browser.

## Publication

PRs to dev/main verify source and build behavior. Main pushes and manual runs on main can deploy. A published stable release
event dispatches a fresh run on main; checking out main within a tag-triggered
job would not satisfy the main-only environment policy. Deployment checks out main,
then rebuilds against the current complete published release. Main/release runs
share one production concurrency group so Pages deployments cannot race.

The feature PR does not merge dev/main, enable hosting or publish a release.
Merge prerequisites using merge commits, then promote dev through a release PR.
The distribution workflow prepares installers from the new immutable main tag;
publish the reviewed draft release with all its assets. In Settings > Pages,
select GitHub Actions and keep the github-pages environment limited to main.
A published release event dispatches the main rebuild automatically. If Pages is configured
later, run Project site manually on main.

Intended address: https://alexanderwanyoike.github.io/muzo/
The existing v0.0.1 tag stays immutable. Current development captures can contain
changes after that tag; download versions always come from the published manifest.
The repository remains UNLICENSED/proprietary. Initial packages are unsigned.

## Visual assets

The download section uses the Windows, Apple and Tux symbols from Bootstrap Icons
(MIT), pinned to source revision e568465481c0c541f63a78f6452d09387711d489.
Original SVGs, provenance and the licence are kept in `site/public/icons/`.
Platform names and architecture choices remain text, so the marks are decorative.

GitHub explicitly permits workflow_dispatch runs created with GITHUB_TOKEN:
https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/trigger-a-workflow
Only the release-refresh job has actions:write; PR verification stays read-only.

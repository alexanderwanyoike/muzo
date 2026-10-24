# Card 037 review record

Base: `d634223` (PR #64, including PR #65 focus fix).
Reviewed head: `00b46370187823941087073ebe0b74ec81322b73`.
Both cycles reviewed the full three-dot diff and six-commit list.

## Initial Standards

Hard documented violations: none found.

Judgement-call smells: none actionable in the pinned Card 037 diff from d634223b5b0f0c07f6b0567ec7a3c7fbf426929f to 00b46370187823941087073ebe0b74ec81322b73.

The site remains a cohesive static HTML/CSS workspace using existing Vite tooling, canonical screenshots, the established waveform icon and a local licensed font. It adds no client application, release API or unnecessary dependency. Public assets preserve stable sharing URLs; the build/preview scripts consistently use the project base. Expected application-failure handling is unaffected because the new TypeScript is integration coverage only.

Meaningful content/asset regressions precede the page implementation, followed by focused CI/documentation and preview-base correction. The new workspace has runnable formatting, Airbnb/type-aware lint, strict type checks and five integration tests, with root commands including it. Official SHA-pinned Pages actions verify PRs and limit artifact publication/deployment to main. The docs preserve the explicit publication/setup boundary and dependency order.

Card 037 and the project-site guide record the established sibling/Vite/official Pages patterns without copied implementation. Font licensing and screenshot provenance are retained. Copy and visual structure follow the user's restrained preference and distinguish illustrative samples from current app captures.

The twelve-smell baseline raises no actionable findings; tool-enforced concerns are excluded. Final gates and desktop/mobile/keyboard visual checks remain with the parent review.

Findings: 0 hard violations; 0 judgement-call smells.


## Initial Spec

# Spec review: GitHub Pages project site, initial cycle

Pinned diff: `d634223b5b0f0c07f6b0567ec7a3c7fbf426929f...00b46370187823941087073ebe0b74ec81322b73`. Reviewed the complete effective diff and six commits against card 037 and the requested sibling-style project site.

## (a) Missing or partial requirements

No actionable implementation findings. Final desktop/mobile/keyboard inspection, ordered gates and handoff remain parent-owned work in progress.

## (b) Unasked scope

No actionable findings. The static workspace, asset verification and guarded Pages workflow directly support a reviewable project site. Desktop application source is unchanged. The feature does not enable hosting, merge branches or publish a release.

## (c) Apparently implemented requirements done incorrectly

No actionable findings. The restrained page uses the existing waveform identity and corrected canonical app screenshots, describes actual local/Dropbox sources, queues and persistent artwork, and provides six material samples plus light/dark and ten-accent explanations. Illustrative samples are labelled separately from real app captures, meeting the actual-feature/no-future-claims criterion.

Source setup targets immutable v0.0.1; GitHub release history is linked and unpublished packaged downloads are explicitly stated. There are no invented binary URLs or custom release API, satisfying "Link source installation and release history honestly; no invented binary download URLs."

Screenshots, favicon and licensed Manrope are self-hosted. Navigation targets, sharing metadata and public asset references have meaningful source-contract tests. Vite build/preview use `/muzo/`; inspection of the generated CSS confirms the font resolves under `/muzo/fonts/`, while sharing images retain stable public paths.

Root workspace checks include the site. The Pages workflow enforces formatting/lint/types/tests/build for relevant PRs; artifact upload and deployment require main and exclude PR events, matching "deploy only main through official Pages actions." Manual feature-branch runs cannot publish. The guide preserves the explicit approval/setup boundary.

This read-only review did not alter repository/browser/hosting state or duplicate the parent's gate and visual runs.


## Final Standards

Hard documented violations: none found.

Judgement-call smells: none actionable on the second review of the pinned Card 037 diff from d634223b5b0f0c07f6b0567ec7a3c7fbf426929f to 00b46370187823941087073ebe0b74ec81322b73.

The standalone static workspace, canonical public assets and project-base build/preview paths retain clear responsibilities. No client JavaScript, release API or application-layer changes are introduced. Local font licensing, screenshot provenance and the distinction between illustrative material samples and actual app captures remain documented. The five HTML integration tests are meaningful content/navigation/asset contracts rather than mirrored implementation tests.

Rechecked the focused red-first history and official SHA-pinned Pages workflow. Formatting, Airbnb/type-aware lint and strict checks are runnable and enforced; artifact/deployment steps remain gated to main. The project guide preserves explicit dependency, approval and publication boundaries. No hosting enablement or repository merge is performed by this review.

The twelve-smell baseline raises no actionable findings; tool-enforced concerns are excluded. The parent reports 338 desktop plus five site tests and root formatting/lint/types green. Built-browser checks at 1440/768/390/320px found no horizontal overflow, verified image/font responses and keyboard skip navigation. Desktop build/package/make and site build are finishing and are not newly certified here.

Findings: 0 hard violations; 0 judgement-call smells.


## Final Spec

# Spec review: GitHub Pages project site, second cycle

Pinned diff: `d634223b5b0f0c07f6b0567ec7a3c7fbf426929f...00b46370187823941087073ebe0b74ec81322b73`. Rechecked the complete changed-file set, six commits and card 037 requirements. The initial review required no code correction.

## (a) Missing or partial requirements

No actionable findings.

## (b) Unasked scope

No actionable findings. The effective feature remains a static site, verification workspace and guarded deployment configuration. No desktop source, hosting setting or branch integration is changed.

## (c) Apparently implemented requirements done incorrectly

No actionable findings on fresh inspection. Actual features, corrected canonical screenshots, six material samples and ten-accent/light-dark explanations remain accurate. Source instructions point to immutable v0.0.1, release history is linked and unpublished binaries are stated honestly, meeting "Link source installation and release history honestly; no invented binary download URLs."

Self-hosted images, icon and licensed Manrope retain stable public paths under the `/muzo/` build/preview base. The parent reports actual preview checks returned JPEG and WOFF2 resources successfully, with all image dimensions correct. Desktop/tablet/mobile views at 1440, 768, 390 and 320px had no horizontal overflow; hero, materials and source installation were inspected. Tab exposed the outlined skip link and Enter reached main with visible spacing, supporting the responsive and keyboard-preview requirements.

Pages artifact upload/deployment still require main and exclude pull-request events. The guide keeps approval and Pages setup explicit, satisfying "deploy only main through official Pages actions" without enabling hosting in this feature.

The parent reports 338 desktop plus five site tests, formatting, lint and strict types green. Build/package/make and final site build remain in progress, so this review does not certify them complete. No repository/browser/hosting state was changed by this read-only re-review.


## Verification

The initial site integration checks ran red before the page existed. They verify
real HTML navigation, installation/release availability, screenshot references,
local assets under the Pages base and canonical sharing metadata. Native JPEG
captures were then retained under Vite's public/ convention, with tests adapted
to that actual encoding and HTML whitespace normalization.

All ordered local gates pass: Yarn install, format check, Airbnb/TypeScript lint,
strict types, 338 desktop tests plus five site tests, desktop build, package and
Linux ZIP/DEB make. The static site production build also passes. The new workspace
uses the existing dependency versions, with no client JavaScript or release API.

The built preview was inspected at 1440, 768, 390 and 320 pixels, with zero document
horizontal overflow. Listening, material samples, personalization and source setup
sections were inspected. All images decoded at their expected intrinsic sizes;
the sharing-image URL returns 200 image/jpeg and the local font returns 200
font/woff2 beneath /muzo/. Keyboard Tab reveals the skip link with a visible
outline; Enter lands at main with the intended scroll margin. CSS respects
prefers-reduced-motion. Main-only publish conditions and PR verification were
reviewed directly; no public deployment or dev/main merge was performed.

The actual seek correction and refreshed README also passed Linux, Windows and
macOS CI in PRs #65 and #64. Pages remains unconfigured (GitHub API 404) until the
user approves publication through the documented main promotion/setup process.

Totals: Standards 0, Spec 0 in each cycle; no fixes required after review.

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

## Corrected native-download experience

The preceding records describe the initial source-installation iteration. The
user rejected it. The current build uses native installer release metadata,
a shared distribution manifest and a revised OS-icon download section instead.

The first corrected-flow review found one Spec P2: a release-published event
runs from a tag ref, so checking out main does not satisfy the main-only Pages
environment. Commit 516f42f dispatches a fresh workflow run on main, scoped to
a published stable release; PR verification stays read-only.

Current complete review base: `c81b032`; source head:
`f98feed65426525b51ad420641783dca7b8a4209`. Both cycles inspect the full
three-dot diff and log. Separately reviewed dependent app/distribution PRs are
included as prerequisites, not folded into this site's responsibility.

### Current Initial Standards

# Corrected project site Standards review

Base `c81b032`; reviewed head `f98feed65426525b51ad420641783dca7b8a4209`.
Reviewed the complete three-dot changed-file set and commit list, then the effective site/workflow/tooling, README and documentation changes against AGENTS.md and the twelve Fowler heuristics. Native player, materials, distribution, licensing and source-width dependencies have separate reviews and are not reclassified as site scope.

Hard documented violations: none found.

Judgement-call smells: none actionable.

Release validation, HTML rendering and the build entry point have separate cohesive responsibilities. Unknown GitHub metadata is validated at the boundary; expected invalid-release/storage failures use the shared typed Result contract. Native target names and manifest validation come from the distribution workspace rather than a second filename policy. DOM construction avoids interpolating untrusted metadata into markup. No browser runtime, custom network service or speculative abstraction is added.

Red-first commits cover native download presentation, real production builds, duplicate assets and malformed digests before their corresponding implementation fixes. Portable Node build tests cover the actual Vite transform and fail-closed production contract. Root/site checks retain strict TypeScript, Airbnb rules and unsafe-value bans without suppressions.

OS symbols are local unchanged Bootstrap SVGs with pinned provenance and preserved MIT notice. Project MIT and font notices remain separate. Current native screenshots are canonical, and seven illustrative material samples are clearly distinguished from real captures. Public installation copy offers the four native targets; source setup remains contributor documentation.

The official pinned Pages actions keep publication on main, with read-only PR checks and scoped actions:write only for the published-release refresh dispatch. The corrected release event schedules a main-ref run rather than attempting to deploy under a tag-ref environment policy. The documented process preserves release/main approval boundaries and immutable tags.

Final gate results, browser inspection, dependency ordering in the delivered PR and publication remain parent-owned handoff work.

Findings: 0 hard violations; 0 judgement-call smells.


### Current Initial Spec

# Spec review: corrected native-download site, initial cycle

Pinned diff: `c81b032...f98feed65426525b51ad420641783dca7b8a4209`. Reviewed the full changed-file/commit set and the site, workflow, tooling, README and current documentation scope against Card 037. Desktop/distribution implementation in separately reviewed prerequisite PRs is treated as dependency context.

## Missing or partial requirements

No actionable findings. Card 037 requires native Windows, both macOS architectures and Linux AppImage downloads without a source-build listener flow. The OS-labelled deck supplies decorative licensed platform symbols, clear formats, architecture-specific macOS choices, install notes and checksum/release links. Local pending controls are disabled; complete published metadata produces four actual installer URLs.

## Unasked scope

No actionable findings. Build-time metadata validation, the private shared contract and guarded Pages publication directly support the requested release-download experience. The site adds no client JavaScript, analytics or remote fonts. Prerequisite player, appearance and licence changes remain separately identified feature work.

## Apparently implemented requirements done incorrectly

No actionable findings. Validation rejects draft/prerelease releases, missing or duplicate assets, inconsistent tags, foreign URLs and installer size/digest mismatches. Production metadata is mandatory. Actual CLI build tests verify four links/checksums and the missing-metadata failure, satisfying Card 037's asset/availability verification criterion.

The earlier release-tag/environment defect is resolved: published stable releases dispatch `pages.yml` on main, while deploy still requires `refs/heads/main` and the main-only environment. Checking out main inside a tag-ref deployment is avoided. PR checks are read-only; production runs share concurrency protection.

Seven material samples, ten accents, current Glass/satin native JPEGs and the MIT footer agree with their documented sources. Local font/icon licences remain present. This review did not rerun gates, configure Pages, publish or certify the parent's pending responsive-browser inspection.

Findings: 0.


### Current Final Standards

# Corrected project site final Standards review

Base `c81b032`; reviewed head `f98feed65426525b51ad420641783dca7b8a4209`.
Rechecked the effective site/workflow/tooling and documentation diff, commit list and AGENTS.md against the twelve Fowler heuristics. No source edits followed the initial review. Separately reviewed native-feature prerequisites remain explicit dependencies.

Hard documented violations: none found.

Judgement-call smells: none actionable.

The second inspection confirms cohesive boundary validation, DOM rendering and build orchestration. Unknown release data is checked before use, expected failures retain the shared Result contract, and canonical filenames/manifest rules reuse the distribution workspace. Complete installer sets, exact repository release URLs, positive sizes, duplicate-name rejection and optional SHA-256 digest validation remain enforced before production links are rendered. Missing production metadata fails closed; local Preparing release controls do not invent download URLs.

Rechecked correction `516f42f`: stable published-release events dispatch a new workflow run on main with narrowly scoped actions:write. Deployment itself still requires a main ref, successful verification and the main-limited Pages environment. It rebuilds using complete published release metadata before official pinned upload/deploy actions. The feature does not merge, tag, publish a release or enable hosting.

Native download presentation remains server-built static HTML with local licensed OS symbols, canonical app screenshots, seven material samples and ten accents. Parent reports fresh browser inspection at 320, 768 and 1440px without horizontal overflow, with OS symbols and both current screenshots loaded. This report does not independently duplicate that browser inspection or certify the parent's still-running serialized ordered gates.

Red/green commits and portable real-build tests remain focused. No suppressed strict-type boundary, unnecessary application service or speculative abstraction was found.

Findings: 0 hard violations; 0 judgement-call smells.


### Current Final Spec

# Spec review: corrected native-download site, final cycle

Pinned diff: `c81b032...f98feed65426525b51ad420641783dca7b8a4209`. Rechecked the complete diff/commit set and site, workflow, tooling, README and documentation scope against Card 037. No code changed after the initial zero-finding review; player/distribution changes remain separately reviewed prerequisites.

## Missing or partial requirements

No actionable findings. The native OS deck has Windows EXE, Apple silicon and Intel DMGs, Linux AppImage, decorative platform symbols, install guidance and checksum/release links, satisfying Card 037 line 15. The parent reports fresh browser checks at 320, 768 and 1440px with no horizontal overflow, loaded native screenshots, seven materials and OS icons/cards. I did not duplicate those browser actions.

## Unasked scope

No actionable findings. Release validation and guarded build-time rendering support the requested native-download experience. The site remains static, with self-hosted images/font/icons and no client JavaScript or source-build visitor flow.

## Apparently implemented requirements done incorrectly

No actionable findings. Complete stable published metadata must match the shared manifest; duplicate/missing assets, foreign URLs, version mismatches and inconsistent sizes/digests stop production builds. Local previews retain disabled Preparing release controls instead of invented download URLs. Maintained tests exercise the actual CLI build and required-metadata rejection (lines 17-18).

Rechecked the prior release-event correction: release jobs dispatch a new workflow run with `--ref main`; deployment separately requires `refs/heads/main` and excludes PRs. This preserves the main-only environment rule rather than relying on a checkout to change the event ref. PR jobs remain read-only, and production runs share concurrency protection.

The current screenshots, captions, seven materials and ten accents agree. Root and public MIT notices have identical hashes; third-party notices remain separate. Parent ordered gates are still running serially after the computer hang, so this review does not certify their completion, Pages configuration or public publication.

Findings: 0.


### Current verification

Both renewed cycles returned zero findings on each axis. Frozen Yarn install,
formatting, Airbnb/type-aware lint, strict types, 375 tests (357 desktop, six
distribution, 12 site), desktop build/package and Linux ZIP/DEB make pass. The
site builds successfully in preview mode; real production-build tests verify
complete published-release URLs and reject absent/inconsistent metadata.
Test workers were capped at two after the host restart to control local load.

Browser inspection at 320, 768 and 1440px found no horizontal overflow. All
canonical native JPEGs and decorative icons decode; Settings shows seven
materials and ten accents. Native download cards preserve architecture choices.
The preview honestly displays Preparing release until a complete stable native
release is published; no source-build installation or invented binary link is
shown. No public deployment, dev/main merge or tag change was performed.

The Frosted glass material provides renderer-layer translucency over internal
artwork. The operating-system window is opaque; it does not reveal the desktop.

# Card 036 review record

Base: `4e23fd61c3efa927ee7d6c0bded62d3e643667e2` (PR #63 prerequisite).
Reviewed head: `d18998dca67577f418972f868dce61d59a6ced9c`.
Full three-dot diff and commit list reviewed in both cycles.

## Initial Standards

Hard documented violations: none found.

Judgement-call smells: none actionable in this documentation/screenshot-only diff from 4e23fd61c3efa927ee7d6c0bded62d3e643667e2 to d18998dca67577f418972f868dce61d59a6ced9c.

The single focused documentation commit updates README status/features, adds canonical screenshots and records capture provenance. Both images show the existing interface with restrained materials, generic fictional sources and original demo artwork, consistent with the user's muted creative preference. README copy clearly distinguishes the current development interface, tagged preview and unavailable packaged downloads.

The screenshot guide records the actual-component fixture boundary and a repeatable capture procedure without committing a throwaway application or exposing user data. Card 036 identifies the pending policy prerequisite and keeps feature-PR delivery against dev. No production logic changes, artificial TDD tests or new error-handling abstractions are introduced.

The twelve-smell baseline raises no actionable findings; tool-enforced concerns are excluded. Both PNGs were visually inspected. Ordered repository checks and PR handoff remain with the parent review.

Findings: 0 hard violations; 0 judgement-call smells.


## Initial Spec

# Spec review: README desktop showcase, initial cycle

Pinned diff: `4e23fd61c3efa927ee7d6c0bded62d3e643667e2...d18998dca67577f418972f868dce61d59a6ced9c`. Reviewed the sole commit, complete five-file diff and card 036.

## (a) Missing or partial requirements

No actionable findings. The listening screenshot is embedded immediately after the introduction, satisfying "Put a clear, current listening screenshot near the top of the README." The project site is explicitly a subsequent slice, so this PR does not need to implement that site.

## (b) Unasked scope

No actionable findings. The second Settings screenshot is a focused supporting asset for the forthcoming site and the updated personalization feature summary. No production code or unrelated feature is introduced.

## (c) Apparently implemented requirements done incorrectly

No actionable findings. Independently inspected both PNGs and verified their hashes match the pinned blobs. The listening image shows the actual current transport, Sources search/list/footer, track artwork and listening sleeve with restrained demo content, meeting "Show the actual interface, artwork, transport and Sources panel using safe demo data." The Settings image shows the current six-material/ten-accent controls.

The README accurately states the desktop-preview status, tagged version, source quick start and absence of published package downloads. Its feature summary matches the integrated player, Dropbox, queue, metadata, persistent artwork and appearance functionality, satisfying "Refresh status and feature copy, with honest source installation availability."

`docs/screenshots.md` records fixture-based capture, dimensions, themes, fictional data/original artwork and the canonical shared asset paths. It explicitly distinguishes current components from future-feature mockups and excludes real catalogue/credential data, fulfilling the provenance/shared-screenshot criterion.

Ordered gates, review record and feature-PR handoff remain in progress as intended. This read-only content/visual review added no tests, changed no repository/browser state and did not duplicate those checks.


## Final Standards

Hard documented violations: none found.

Judgement-call smells: none actionable on the second review of the pinned Card 036 diff from 4e23fd61c3efa927ee7d6c0bded62d3e643667e2 to d18998dca67577f418972f868dce61d59a6ced9c.

README copy, image descriptions and provenance remain consistent with the visually inspected PNGs: current listening/Settings interfaces, restrained materials, fictional sources and original demo artwork. The README distinguishes the development interface from the tagged preview and accurately directs users to source installation while packaged downloads remain unpublished.

The single focused documentation commit introduces no production code or artificial TDD tests. Canonical screenshots and their refresh procedure avoid duplicated assets or throwaway application scaffolding. Card 036 preserves the explicit policy-PR prerequisite and feature delivery against dev. The twelve-smell baseline raises no actionable findings; tool-enforced concerns are excluded.

The parent reports formatting, lint and type checking green. Desktop tests, build, package and make are still running and are not claimed complete by this review.

Findings: 0 hard violations; 0 judgement-call smells.


## Final Spec

# Spec review: README desktop showcase, second cycle

Pinned diff: `4e23fd61c3efa927ee7d6c0bded62d3e643667e2...d18998dca67577f418972f868dce61d59a6ced9c`. Rechecked the sole commit and full five-file scope against card 036. No initial finding required a fix.

## (a) Missing or partial requirements

No actionable findings. The canonical listening screenshot remains directly beneath the introductory copy, meeting "Put a clear, current listening screenshot near the top of the README." Site implementation remains intentionally separate.

## (b) Unasked scope

No actionable findings. Both screenshot assets, provenance notes and refreshed status copy support the requested README showcase and planned shared site assets. Application behavior is untouched.

## (c) Apparently implemented requirements done incorrectly

No actionable findings after rechecking the pinned README and provenance against the previously inspected matching PNG blobs. The listening image includes current artwork, transport and Sources controls, while the Settings image demonstrates six materials and ten accents. Fixture provenance explicitly identifies fictional libraries/tracks and original artwork with no real credentials/catalogue data, satisfying the actual-interface/safe-demo requirement.

The README calls the app a desktop preview, links the real v0.0.1 tag, directs readers to the existing source quick start and states packaged downloads are not published. Feature copy remains consistent with the current integrated implementation. Shared canonical paths and capture guidance satisfy "Share canonical screenshots with the forthcoming project site and record provenance."

The parent reports formatting, lint and type checks green; desktop tests, build, package and make remain ongoing. This report does not certify those unfinished checks. The read-only review made no repository/browser changes or artificial tests.


## Verification

Yarn install, formatting, lint, strict types, all 337 desktop tests, desktop build,
package and make pass. Visually inspected both screenshots at their native sizes.
No production code changed, so no artificial tests were added.

Totals: Standards 0, Spec 0 in each cycle; no fixes required.

## Capture refresh after Card 038

Base: `e805aae` (PR #65); reviewed head: `f9a75108ec15ae7775dff95058f3611fcbebd04a`.
Both cycles used the full three-dot diff and four-commit list.

### Refresh initial Standards

Hard documented violations: none found.

Judgement-call smells: none actionable in the refreshed Card 036 diff from e805aae3cdf57f55468d2b1189d71f7514faa8f2 to f9a75108ec15ae7775dff95058f3611fcbebd04a.

The scoped diff remains documentation and canonical image assets; application files match the reviewed seek-fix prerequisite. The new listening image was visually inspected and has no pointer-focus rectangle. Both assets have genuine JPEG encoding matching their .jpg names and documented dimensions. README/provenance now share the public asset paths, preserving one canonical copy for documentation and the forthcoming site.

Copy and imagery remain restrained and identify fictional data/original demo artwork. The README distinguishes development UI, tagged preview and unpublished package downloads. The card explicitly distinguishes historical 337-test evidence from the included fix's 338-test baseline rather than silently rewriting the earlier review record.

Focused documentation/history commits preserve the prerequisite merge and screenshot refresh without introducing production logic or artificial TDD tests. The twelve-smell baseline raises no actionable findings; tool-enforced concerns are excluded. Final repository checks and refreshed re-review remain with the parent.

Findings: 0 hard violations; 0 judgement-call smells.


### Refresh initial Spec

# Spec review: README capture refresh, initial cycle

Pinned diff: `e805aae3cdf57f55468d2b1189d71f7514faa8f2...f9a75108ec15ae7775dff95058f3611fcbebd04a`. Reviewed the full four-commit list, effective six-file diff and card 036 with the user's fix-before-capture instruction.

## (a) Missing or partial requirements

No actionable findings. The reviewed seek-focus dependency is incorporated before the replacement capture; site implementation remains a separate slice.

## (b) Unasked scope

No actionable findings. The effective diff contains only README/documentation and canonical screenshot assets. Independent comparison confirms application/workspace source and lockfile are unchanged from the reviewed focus-fix base.

## (c) Apparently implemented requirements done incorrectly

No actionable findings. Independently inspected both JPEGs and confirmed their hashes match the pinned blobs. The 1440x900 listening capture shows the actual current transport, source panel, track art and sleeve, with no blue rectangle around the seek bar. This addresses the user's requested correction before capture while satisfying "Put a clear, current listening screenshot near the top of the README."

The 1280x900 Settings capture displays the six-material/ten-accent controls. Both use restrained, fictional demo content. Native file signatures and dimensions match the `.jpg` names and recorded provenance. Canonical assets now live under `site/public/images`; the README points to the actual committed listening file and the guide records their unchanged-asset role for the forthcoming Vite site, meeting "Share canonical screenshots with the forthcoming project site and record provenance."

README preview/version/source-installation copy remains accurate and clearly says package downloads are unpublished. The card distinguishes historical 337-test evidence from the included 338-test focus-fix baseline. This read-only review did not rerun ordered gates, change repository/browser state or treat pending refreshed review records as a behavior defect.


### Refresh final Standards

Hard documented violations: none found.

Judgement-call smells: none actionable on the second review of the refreshed Card 036 diff from e805aae3cdf57f55468d2b1189d71f7514faa8f2 to f9a75108ec15ae7775dff95058f3611fcbebd04a.

README and capture provenance remain consistent with the inspected listening image: current interface, fictional catalogue, original demo artwork and no pointer-focus rectangle. Canonical .jpg paths match the verified JPEG encodings and preserve one asset copy for README/site use. The public-directory placement supports stable site URLs without introducing a build/runtime dependency into the desktop app.

The four-commit history keeps the prerequisite merge, historical verification record and fresh capture update focused. The actual diff contains only documentation and image assets; application code remains at the reviewed seek-fix prerequisite. No artificial TDD tests are warranted, and prior review records are identified as historical evidence.

The twelve-smell baseline raises no actionable findings; tool-enforced concerns are excluded. Repository check completion and PR handoff remain with the parent review and are not newly certified here.

Findings: 0 hard violations; 0 judgement-call smells.


### Refresh final Spec

# Spec review: README capture refresh, second cycle

Pinned diff: `e805aae3cdf57f55468d2b1189d71f7514faa8f2...f9a75108ec15ae7775dff95058f3611fcbebd04a`. Rechecked the full six-file diff and four commits against card 036 and the fix-before-capture request. No initial finding required a correction.

## (a) Missing or partial requirements

No actionable findings. The focus-fix dependency precedes the refreshed capture; the project site remains intentionally separate.

## (b) Unasked scope

No actionable findings. The effective change is README/provenance/review documentation plus two screenshots. Production source remains unchanged from the reviewed seek-fix base.

## (c) Apparently implemented requirements done incorrectly

No actionable findings. The README embeds the committed listening JPEG directly below the introduction, meeting "Put a clear, current listening screenshot near the top of the README." The previously inspected pinned image shows a clean seek timeline together with current transport, Sources controls, track covers and listening sleeve. The Settings JPEG shows the same current six-material/ten-accent UI.

The image signatures, dimensions and canonical `site/public/images` paths match the guide and README. The provenance identifies actual app fixtures, fictional metadata/original artwork and native JPEG captures, satisfying "Show the actual interface, artwork, transport and Sources panel using safe demo data" and the canonical sharing/provenance criterion.

Status copy accurately distinguishes the desktop preview, tagged version, source installation and unpublished package downloads. Historical 337-test evidence is qualified separately from the included focus fix's 338-test baseline. Final gate/review-record handoff remains parent-owned; this read-only re-review did not duplicate those runs or change repository/browser state.


The fresh capture follows the original mouse-seek reproduction after Card 038.
Both native JPEG dimensions/encoding were verified. Format check passes; included
application files are identical to PR #65, whose complete ordered gates pass with
338 tests. No additional application changes require repeating those gates.
Totals for the refreshed content: Standards 0, Spec 0 in both cycles.

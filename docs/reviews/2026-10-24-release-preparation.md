# Release preparation review record

Base: `04c08af4f863979847b844e05f8c8e320938270b`.
Reviewed code/metadata head: `a6359cbe67fb013615c34a694f69d94c0c6e2700`.
Both cycles use the full three-dot diff and commit list against Card 032,
AGENTS.md and release notes. The initial cycle found no defects requiring changes.
User authorization covers review and merging into dev.

## Initial Standards

No documented-standard violations or actionable baseline smells found in the pinned release-preparation diff from 04c08af4f863979847b844e05f8c8e320938270b to a6359cbe67fb013615c34a694f69d94c0c6e2700.

The two commits keep synchronized package-version metadata separate from release notes and Card 032. The slice introduces no production logic, dependency, error-handling or architecture change, so artificial failing tests are unnecessary. Existing application and packaging verification appropriately validate the version bump.

Card 032 records the scoped adaptation of the release workflow, feature-PR preparation into dev, subsequent dev-to-main release PR and explicit publication boundary. The supplied user authorization covers completed-work merges into dev; the preparation itself does not introduce a direct main/dev commit or unauthorized release merge.

The twelve-smell baseline raises no actionable findings in this metadata/documentation slice. Tool-enforced concerns are excluded. Final ordered gates, packaged-version inspection and PR handoff remain with the parent review.

Findings: 0 hard violations; 0 judgement-call smells.

## Initial Spec

# Spec review: release 0.0.1 preparation, initial cycle

Pinned diff: `04c08af4f863979847b844e05f8c8e320938270b...a6359cbe67fb013615c34a694f69d94c0c6e2700`. Reviewed the exact two commits against card 032 and the authorized release-preparation scope.

## (a) Missing or partial requirements

No actionable findings. Both root workspace and desktop package versions change from 0.0.0 to 0.0.1, satisfying "Set root workspace and desktop package metadata to 0.0.1." No remaining tracked application version declaration uses 0.0.0 outside the card's historical explanation.

## (b) Unasked scope

No actionable findings. Changes are restricted to two version fields, first-release notes and the preparation card. No Helm files, tags, publication workflows or unrelated features are introduced.

## (c) Apparently implemented requirements done incorrectly

No actionable findings. The release summary accurately describes the integrated library/Dropbox, metadata, playback/queue, artwork cache, appearance, sidebar and Settings behavior previously reviewed. The five-accent claim matches the retained appearance definitions. Native CI coverage is described separately from manual desktop verification rather than claiming all target-shell behavior was manually tested.

The diff preserves runtime source, secure-storage identity and data-path configuration, satisfying "Preserve application code, secure-storage identity and existing data paths." It introduces no implementation requiring artificial red-first tests. Gate and packaged-version verification remain unchecked until the parent finishes them.

The card also explicitly keeps the subsequent dev-to-main release PR open for approval and excludes tagging/publication. This metadata diff performs none of those external actions. The parent owns final gate evidence and release-PR handoff; this read-only review did not duplicate those operations.

## Final Standards

No documented-standard violations or actionable baseline smells found on the second review of the pinned release-preparation diff from 04c08af4f863979847b844e05f8c8e320938270b to a6359cbe67fb013615c34a694f69d94c0c6e2700.

Both workspace versions remain synchronized at 0.0.1. Package metadata and release documentation are separate focused commits, with no production logic, dependency or architecture changes. Avoiding artificial red tests for this slice is appropriate; maintained tests and packaging provide the relevant verification.

Card 032 preserves feature-PR preparation into dev and a subsequent dev-to-main release PR, consistent with the supplied user authorization and documented Git flow. It leaves release merging, tagging and publication outside the preparation change. The first-release notes describe the integrated desktop feature set and existing verification scope.

The twelve-smell baseline raises no actionable findings; tool-enforced concerns are excluded. The parent reports install, format, lint, types, 329 tests, build and package passing. Final make completion and packaged 0.0.1 inspection remain with the parent review.

Findings: 0 hard violations; 0 judgement-call smells.

## Final Spec

# Spec review: release 0.0.1 preparation, second cycle

Pinned diff: `04c08af4f863979847b844e05f8c8e320938270b...a6359cbe67fb013615c34a694f69d94c0c6e2700`. Rechecked both commits, the complete changed-file set, card 032 and release notes. No initial finding required a fix.

## (a) Missing or partial requirements

No actionable findings. Root workspace and desktop package metadata both declare 0.0.1, satisfying "Set root workspace and desktop package metadata to 0.0.1." The first-release document covers the integrated desktop feature set.

## (b) Unasked scope

No actionable findings. The four changed files contain only the two version updates, preparation card and release notes. No Helm additions, tagging, publication or production logic changes appear.

## (c) Apparently implemented requirements done incorrectly

No actionable findings on fresh inspection. Release notes remain consistent with reviewed libraries/Dropbox, metadata, playback/queue, persistent artwork, appearance, matching panels and Settings behavior. They distinguish native CI coverage from manual desktop checks.

Runtime source, secure-storage identity and catalogue/profile paths remain unchanged, matching "Preserve application code, secure-storage identity and existing data paths." The card's subsequent dev-to-main promotion remains an open release PR for approval; tagging/publication are explicitly outside scope.

The parent reports install, formatting, lint, types, 329 tests, build and package passed. Make and direct packaged-version verification remain in progress, so this review does not certify those unfinished checks. The read-only review made no repository changes and did not duplicate the parent's gate run or external handoff operations.


Initial totals: Standards 0, Spec 0. Final totals: Standards 0, Spec 0.

Install, format check, lint, typecheck, 329 tests, build, package and make all pass. Direct inspection confirms the packaged desktop manifest version is 0.0.1.

# Card 043 - Independent MIT project licence review

Base: `f3897c1f56f872cebee6d3aec857baae7db53e45` (`origin/dev`).
Reviewed head: `d3afb00872953bee605350c03007cb88ca6740b8`.

The project owner explicitly authorised MIT licensing and had already authorised
two review cycles for this project's PRs. The parent coordinated independent
Standards and Spec reviewers in both cycles. No fixes were requested; the final
cycle reviewed the same core head afresh. Only this evidence and the card status
were added after review.

Verification reported by the parent: frozen Yarn install, format, lint,
typecheck, 337 maintained tests, desktop build/package/make and native notice
comparison all passed. The patch contains licence text, metadata and docs only;
no artificial TDD tests were added. Pending website/private-workspace notice
synchronisation is delivered separately in PR #66.

## Initial Standards review

Hard documented violations: none found.

Judgement-call smells: none actionable in the independent MIT branch diff from f3897c1f56f872cebee6d3aec857baae7db53e45 to d3afb00872953bee605350c03007cb88ca6740b8, reviewed in the mit-license worktree.

The six-file, single-commit scope implements the owner's explicit licensing instruction without pulling pending site/distribution features into project adoption. Root and desktop metadata both declare MIT, README removes the proprietary claim and preserves third-party terms, and the root/native MIT notices have identical SHA-256 hashes. Native notice placement follows the existing packaged-assets boundary.

Card 043 clearly identifies the independent dev-based feature PR and separates later website/workspace synchronization. It records the authoritative standard template and direct-comparison verification while preserving third-party notices and historical records. There is no runtime logic or application error-handling change, so artificial TDD tests are unnecessary.

The twelve-smell baseline raises no actionable findings; tool-enforced concerns are excluded. Final verification, second review and feature-PR handoff remain with the parent. This review does not merge dev/main or modify repository files.

Findings: 0 hard violations; 0 judgement-call smells.

## Initial Spec review

# Card043 independent MIT core: initial Spec review

Pinned diff: `f3897c1f56f872cebee6d3aec857baae7db53e45...d3afb00872953bee605350c03007cb88ca6740b8`. Reviewed the complete six-file diff and its single commit against the revised Card043 and the user's MIT request.

## (a) Missing or partial requirements

No actionable findings. Root and desktop metadata declare MIT, and README replaces its proprietary claim with a direct root-licence link, satisfying "Set the root and desktop package metadata to MIT" and "Replace the current README proprietary claim with the MIT licence link."

The root and native notices are byte-identical, SHA-256 `e6d26c00d2f72560e6fb95569c7083c265c573d716267879c15d448dd89c839b`. Existing Forge packaging includes the assets/licenses directory, supporting "Include an identical project notice in the packaged native app assets." Verification and handoff remain intentionally pending in the card.

## (b) Unasked scope

No actionable findings. The branch directly extends the fixed dev base with licence text, metadata and documentation only. No website/distribution dependency, version bump, runtime change or release operation is included. Their pending notice synchronization is explicitly scoped to PR66; those workspaces do not exist at this pinned head.

## (c) Implemented incorrectly

No actionable findings. The licence matches the previously checked standard MIT template with the requested attribution, satisfying "Add the standard MIT licence, copyright 2026 Muzo contributors." The authoritative source is the [GitHub Choose a License MIT template](https://choosealicense.com/licenses/mit/).

No third-party notice or historical record is modified, satisfying "Preserve all third-party licence terms and historical decision records." The README also explicitly retains third-party terms. No artificial test-first cycle is needed for this metadata/documentation-only patch.

This review was read-only and did not rerun the parent’s verification gates.

## Final Standards review

Hard documented violations: none found.

Judgement-call smells: none actionable on the second review of the independent MIT branch from f3897c1f56f872cebee6d3aec857baae7db53e45 to d3afb00872953bee605350c03007cb88ca6740b8, reviewed in the mit-license worktree.

The same focused six-file documentation/metadata commit remains unchanged. Root and native MIT notices again match exactly at SHA-256 e6d26c00d2f72560e6fb95569c7083c265c573d716267879c15d448dd89c839b. Root/desktop metadata and README consistently implement the owner's instruction. Third-party notice files and historical ADRs are untouched.

The independent feature remains based directly on dev; Card 043 explicitly separates pending site/distribution synchronization. Native notice inclusion uses the existing packaged-assets boundary. No runtime logic, application failure path or artificial tests are introduced. The twelve-smell baseline raises no actionable findings; tool-enforced concerns are excluded.

The parent reports 337 maintained tests and all ordered repository gates passing. No fixes were required after initial review. This review makes no repository merge, tag or publication change.

Findings: 0 hard violations; 0 judgement-call smells.

## Final Spec review

# Card043 independent MIT core: final Spec review

Pinned diff: `f3897c1f56f872cebee6d3aec857baae7db53e45...d3afb00872953bee605350c03007cb88ca6740b8`. Freshly reviewed the complete six-file diff and single commit against revised Card043. The initial review required no fixes; this head is unchanged.

## (a) Missing or partial requirements

No actionable findings. Reconfirmed both package declarations are MIT, README directly links the root licence, and root/native notices are identical with SHA-256 `e6d26c00d2f72560e6fb95569c7083c265c573d716267879c15d448dd89c839b`. These fulfill "Set the root and desktop package metadata to MIT" and "Include an identical project notice in the packaged native app assets."

The existing Forge packaging retains assets/licenses. The parent reports all ordered gates, including 337 tests, passed. This review did not rerun them. PR handoff remains intentionally pending.

## (b) Unasked scope

No actionable findings. The independent branch adds no website/distribution dependency, version bump, runtime behavior, tag or publication action. Card043 explicitly assigns pending website/workspace synchronization to PR66, avoiding a dependency on that stack.

## (c) Implemented incorrectly

No actionable findings. Standard MIT text and the requested 2026 Muzo contributors attribution fulfill "Add the standard MIT licence, copyright 2026 Muzo contributors." The template was checked against [Choose a License](https://choosealicense.com/licenses/mit/) in the initial licensing review and its text remains unchanged.

README removes its proprietary claim and explicitly preserves third-party terms. No third-party notice, immutable ADR or historical review changes, meeting "Preserve all third-party licence terms and historical decision records." No artificial runtime tests were introduced for this licence/metadata/documentation-only task.

This review changed no repository or external state.

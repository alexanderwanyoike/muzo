# Card 035 review record

Base: `f3897c1f56f872cebee6d3aec857baae7db53e45`.
Reviewed head: `3513327df47e72d5d9cd912c28a587810caa8dfb`.
Both cycles used the three-dot diff/full commit list, updated AGENTS.md and Card 035.
The latest user instruction supersedes the old squash-merge rule.

## Initial Standards

No documented-standard violations or actionable baseline smells found in the pinned Card 035 diff from f3897c1f56f872cebee6d3aec857baae7db53e45 to 3513327df47e72d5d9cd912c28a587810caa8dfb.

The user's latest instruction supersedes the former squash policy. AGENTS.md, README and new ADR 0008 consistently require merge commits and immutable annotated release tags while retaining feature-to-dev/release-to-main flow. The decision is recorded in a new ADR without rewriting accepted historical decisions or release commits.

Verified the ancestry-repair merge has dev and the released main commit as parents with identical trees; its separate focused commit changes no application files. The subsequent documentation commit changes only the four policy/card files. The local v0.0.1 ref is an annotated tag resolving to the recorded actual main release commit. No artificial TDD tests are warranted for this documentation/ancestry-only slice.

The twelve-smell baseline raises no actionable findings; tool-enforced concerns are excluded. The parent reports repository merge settings and remote tag verified directly. Final ordered gates, re-review and the open feature-PR handoff remain with the parent; this review performs no dev/main merge.

Findings: 0 hard violations; 0 judgement-call smells.

## Initial Spec

# Spec review: merge history and release tag, initial cycle

Pinned diff: `f3897c1f56f872cebee6d3aec857baae7db53e45...3513327df47e72d5d9cd912c28a587810caa8dfb`. Reviewed all three commits and the four documentation/policy files against card 035 and the user's request.

## (a) Missing or partial requirements

No actionable findings. A direct read-only GitHub API check confirms merge commits enabled, squash disabled and rebase merging disabled, satisfying "Enable merge commits only in GitHub; disable squash and rebase merging."

Remote tag enumeration confirms `v0.0.1` is an annotated tag object whose peeled target is `f626742e1e8d48af6ec665b69fd448ca46ff4bdb`, matching "Push annotated v0.0.1 at the released main commit, f626742."

## (b) Unasked scope

No actionable findings. The ancestry repair supports future dev-to-main promotions while preserving the existing released commit. No GitHub Release publication, application change or shared-history rewrite is introduced. The policy PR is explicitly reserved for the user's merge.

## (c) Apparently implemented requirements done incorrectly

No actionable findings. `AGENTS.md` and README consistently require merge commits for feature and release PRs, with immutable annotated release tags on the merged main commit. ADR 0008 explains the existing squash history and preserves historical records.

Independent commit inspection confirms `0af3412` has both the actual dev head `f3897c1` and released main `f626742` as parents. All three commits share tree `a3e4ff87c1bbf099a30123b791d64f094e984aed`, proving that repair changes ancestry only. The full feature diff then adds only the stated policy documents, meeting "Include the main release commit in the feature branch ancestry with identical application files."

No production logic changed, so artificial red-first tests are unnecessary. Final ordered checks and PR handoff remain with the parent task. This review used read-only repository/API operations and changed no repository or external state.

## Final Standards

No documented-standard violations or actionable baseline smells found on the second review of the pinned Card 035 diff from f3897c1f56f872cebee6d3aec857baae7db53e45 to 3513327df47e72d5d9cd912c28a587810caa8dfb.

The updated AGENTS.md, README and new ADR consistently implement the user's latest merge-history instruction while preserving feature-to-dev and release-to-main flow. Historical ADRs/releases remain unchanged. The separate ancestry-only merge and focused policy-documentation commit introduce no production logic or reason for artificial TDD tests.

Reverified v0.0.1 is an annotated tag resolving to f626742e1e8d48af6ec665b69fd448ca46ff4bdb, and origin/main is an ancestor of the feature head. The initial review verified identical merge-parent trees. The policy PR remains the integration boundary; this review performs no new dev/main merge or tag mutation.

The twelve-smell baseline raises no actionable findings; tool-enforced concerns are excluded. The parent reports merge-only GitHub settings verified and 337 tests, install/format/lint/types/build/package passing. Final make completion and PR handoff remain with the parent review.

Findings: 0 hard violations; 0 judgement-call smells.

## Final Spec

# Spec review: merge history and release tag, second cycle

Pinned diff: `f3897c1f56f872cebee6d3aec857baae7db53e45...3513327df47e72d5d9cd912c28a587810caa8dfb`. Rechecked the full three-commit history and four-file policy/documentation diff. The initial review required no correction.

## (a) Missing or partial requirements

No actionable findings. A fresh read-only GitHub API check confirms merge commits enabled and squash/rebase merging disabled. Remote refs still show an annotated `v0.0.1` tag peeling to the released main commit `f626742e1e8d48af6ec665b69fd448ca46ff4bdb`, satisfying the two directly applied repository requirements.

## (b) Unasked scope

No actionable findings. The remaining feature PR is explicitly left for the user's merge; there is no new dev/main integration or GitHub Release publication. Application files, workspace metadata and lockfile are byte-identical to the base.

## (c) Apparently implemented requirements done incorrectly

No actionable findings. The released main commit is independently confirmed as an ancestor of the feature head. The ancestry-only merge retains the original dev and main parents and identical application tree, meeting "Include the main release commit in the feature branch ancestry with identical application files."

AGENTS.md and README consistently direct feature/release PRs to use merge commits and annotate immutable release tags on the merged main commit. ADR 0008 records the rationale while leaving historical squashes and the existing release intact, satisfying "Record rationale in ADR 0008 without rewriting historical records."

The parent reports 337 tests and ordered checks green except make finishing. No production logic changes require artificial TDD tests. This read-only review did not duplicate those gates or alter repository/settings/tag state; final PR handoff remains with the parent.


Initial totals: Standards 0, Spec 0. Final totals: Standards 0, Spec 0.

All ordered checks pass: install, format check, lint, typecheck, 337 tests, build, package and make. API readback confirms merge commits only; remote v0.0.1 is an annotated tag pointing to f626742. Application files are unchanged and main is an ancestor of this feature branch.

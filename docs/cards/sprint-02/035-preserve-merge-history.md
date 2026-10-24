- Sprint: 02
- Type: Repository policy
- Status: In review

# Card 035 - Preserve merge history and tag the first release

The user requested disabling squash merges after the first release was squashed,
and tagging the release. Use merge commits consistently and preserve shared history.

## Acceptance criteria

- [x] Enable merge commits only in GitHub; disable squash and rebase merging.
- [x] Push annotated v0.0.1 at the released main commit, f626742.
- [x] Update AGENTS.md and README with the merge/tag policy.
- [x] Record rationale in ADR 0008 without rewriting historical records.
- [x] Include the main release commit in the feature branch ancestry with identical application files, ready for integration into dev through this PR.
- [x] Complete ordered checks and both review cycles.

This change contains repository policy/documentation and an ancestry-only merge.
There is no production logic change, so no artificial TDD tests are added. Repo
settings and the remote annotated tag are verified directly. The policy PR stays
open for the user's merge; the setting change and release tag are already applied.

All ordered gates pass with 337 tests. Both Standards/Spec review cycles have
zero findings. See [review record](../../reviews/2026-10-24-preserve-merge-history.md).

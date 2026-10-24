- Sprint: 02
- Type: Release preparation
- Status: In progress
- Depends on: 031 (PR #58, merged)

# Card 032 - Prepare Muzo 0.0.1

The user authorized merging all completed work into dev and opening a release PR.
There are no previous tags/releases and no Helm chart in this Electron project.
Adapt the release-pr skill's default patch increment to the existing workspace
and desktop package versions, 0.0.0 to 0.0.1. Keep changes on a feature PR into dev.

## Acceptance criteria

- [x] Set root workspace and desktop package metadata to 0.0.1.
- [x] Record first-release notes covering the integrated desktop feature set.
- [x] Preserve application code, secure-storage identity and existing data paths.
- [ ] Verify all ordered gates and packaged desktop version.
- [ ] Complete initial review and re-review before merging preparation into dev.

This is version metadata and documentation only; no production logic changes or
artificial failing tests are needed. Existing tests and packaging validate it.
The subsequent release PR promotes dev to main using a merge commit and remains
open for approval. Tagging or publishing a GitHub release is outside this request.

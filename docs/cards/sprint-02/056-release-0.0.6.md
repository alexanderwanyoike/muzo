# 056: Prepare Muzo 0.0.6

## Background

The user approved merging PR #88 into dev, creating and merging a release PR, then tagging and publishing the release. Promote the reviewed update status bar as Muzo 0.0.6 without changing updater behaviour.

## Acceptance criteria

- [x] Merge PR #88 into dev with a merge commit.
- [x] Match root and desktop versions at 0.0.6 through a focused preparation PR.
- [x] Explain both the 0.0.5 sidebar upgrade path and the new status-bar update journey.
- [ ] Complete release reviews, ordered local gates and native CI.
- [ ] Merge dev to main through a release PR and create an annotated v0.0.6 tag on that merge commit.
- [ ] Verify tag-built downloads, update metadata and a rendered packaged-player smoke test before publication.
- [ ] Publish the verified release, check update discovery from 0.0.5 and refresh the public download page.

## Verification

Release metadata and documentation only; use existing release-context, application, distribution and site checks instead of artificial version-specific tests. Keep local test workers at one after the previously reported memory-pressure crash. Manually exercise the packaged footer using a disposable profile and capture its resting and checking states. Real AppImage update/restart coverage remains in Linux installer CI. Record final CI, manual verification and publication evidence on the release PR.

Local ordered gates passed with one worker: install, lint, strict types, 444 tests, desktop build, package, make and formatting. Preparation review reported zero Standards and Spec findings in both cycles. Promotion review records one inherited commit-atomicity exception in merged PR #88 without rewriting shared history. [Review record](../../reviews/2026-10-28-release-0.0.6.md). Native CI and final release evidence are recorded on the PRs.

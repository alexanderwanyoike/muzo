# 054: Prepare Muzo 0.0.5

## Background

The user approved merging the reviewed Pulse material and glowing accents, then preparing and merging a release PR. Promote dev to main through a merge commit and publish an annotated v0.0.5 release with matching native installers and updater metadata.

## Acceptance criteria

- [x] Merge PRs #84 and #85 into dev with merge commits.
- [x] Match root and desktop package versions at 0.0.5 through a feature PR.
- [x] Add release notes covering Pulse, glowing accents and the installed updater flow.
- [x] Complete initial review, re-review and the ordered local gates.
- [ ] Promote dev through a release PR after native CI passes, then tag and publish the verified release.

## Verification

This preparation changes release metadata and documentation only. Use the existing application, release-context, distribution and site tests instead of adding artificial version-specific tests. Run all ordered repository checks with one local test worker because the user's computer recently crashed under memory pressure. Native CI also verifies an actual AppImage update and restart before publication.

Local verification passed with one worker: install, lint, typecheck, 443 tests, desktop build, package, make and formatting. The release-context command also accepts both 0.0.5 package versions and the exact source commit. [Review record](../../reviews/2026-10-25-release-0.0.5.md).

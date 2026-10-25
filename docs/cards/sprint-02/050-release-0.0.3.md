# 050: Prepare Muzo 0.0.3

## Background

The user approved releasing the reviewed AppImage menu integration, mini player chrome and explicit updater changes. Promote the merged dev branch through a release PR to main and publish an annotated v0.0.3 release with all native installers and updater metadata.

## Acceptance criteria

- [x] Merge PRs #77, #78 and #79 into dev with merge commits.
- [x] Match root and desktop package versions at 0.0.3 through a feature PR.
- [x] Add release notes covering Linux menus, mini player chrome, update behaviour, the v0.0.2 manual upgrade and unsigned macOS fallback.
- [x] Correct distribution and README wording to match the released behaviour.
- [x] Complete release preparation initial review and re-review, and run existing ordered gates.

## Verification

Release metadata and documentation, plus an explicit timeout for the existing native menu launch test after its default five-second limit was exceeded during the full run. No production behaviour changes or artificial tests. Run the existing release-context, distribution, application and site checks. The release PR records native CI and publication evidence.

Local gates passed: install, formatting, lint, types, 426 tests, build, package and make. Both release review axes reported zero findings in both cycles. [Review record](../../reviews/2026-10-25-release-0.0.3.md). Native installer CI and publication are recorded on the PRs.

CI exposed a second fixture-only timing race: a native pointer click could be lost while the mini player compositor caught up with a resize. The next run confirmed an asynchronous viewport resize: the queue was measured at 360 pixels just after requesting 740 pixels. The fixture now waits for the requested viewport and restored native bounds, then uses native pointer activation for every control. Layout, restoration, pointer scrolling and keyboard assertions remain in place.

# 050: Prepare Muzo 0.0.3

## Background

The user approved releasing the reviewed AppImage menu integration, mini player chrome and explicit updater changes. Promote the merged dev branch through a release PR to main and publish an annotated v0.0.3 release with all native installers and updater metadata.

## Acceptance criteria

- [x] Merge PRs #77, #78 and #79 into dev with merge commits.
- [x] Match root and desktop package versions at 0.0.3 through a feature PR.
- [x] Add release notes covering Linux menus, mini player chrome, update behaviour, the v0.0.2 manual upgrade and unsigned macOS fallback.
- [x] Correct distribution and README wording to match the released behaviour.
- [ ] Complete release preparation initial review and re-review, and run existing ordered gates.

## Verification

Release metadata and documentation only. No production behaviour changes or artificial tests. Run the existing release-context, distribution, application and site checks. The release PR records native CI and publication evidence.

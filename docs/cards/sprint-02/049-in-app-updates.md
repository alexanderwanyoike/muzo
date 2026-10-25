# 049: In-app updates

## Background

An installed Muzo should announce a newer release at the bottom of the left sidebar. The user chose: click the notice to download, then choose Restart to install. Playback must continue until that choice.

Depends on [047: AppImage desktop integration](047-appimage-desktop-integration.md), PR #77. The AppImage replacement must keep the launcher and pinned executable path intact. Mini player chrome is a separate PR #78.

## Acceptance criteria

- [x] Installed builds check the fixed public Muzo GitHub repository at startup and every six hours. Development builds do not check or install.
- [x] Offer only a newer complete, published stable release, never a draft, prerelease or downgrade.
- [x] Display `Update available v<version>` at the bottom of the left sidebar, with explicit download, progress, retry and Restart to install states.
- [x] Download with electron-updater's SHA-512 verification. Reject a release that changed after the notice appeared.
- [x] Normal application exit never installs a pending update. Later keeps it ready in the current session.
- [x] Linux AppImage: stage and atomically replace in the same directory, retain filename and executable permissions, then relaunch. Failure before replacement leaves the old executable intact.
- [x] Windows x64: use the NSIS updater and restart only after the explicit restart choice.
- [x] Unsigned macOS and non-AppImage Linux builds clearly open the release page for manual installation. They never promise automatic installation.
- [x] Renderer commands cannot choose a repository, feed, executable or version; accept only commands from a known window's main frame.
- [x] Release assembly validates and publishes `latest.yml` and `latest-linux.yml`, including exact version, filename, size and SHA-512, and includes them in SHA256SUMS.
- [x] Ordered local gates and native integration checks pass. Installer CI is tracked on the PR.
- [x] Initial code review and fix re-review are recorded.

## Pattern

Follows Showbiz's installed-build updater boundary and explicit download/restart flow, under its MIT licence. Uses electron-updater 6.8.11 and electron-builder 26.18.0 instead of implementing an update protocol. See [ADR 0009](../../adr/0009-explicit-desktop-updates.md).

## Tested

Application state transitions, native backend integrity/version guards, cancelled restart, progress and asynchronous errors, IPC authorization and argument rejection, preload failures, sidebar interactions, release metadata validation and native Electron download/replacement/layout fixture. The native fixture uses an executable test payload and a local HTTP feed, with a corrupt download before a valid retry. It never accesses the user's library or credentials.

Published v0.0.2 predates the updater. Users must install the first updater-enabled release manually once. Published tags are unchanged.

Local verification: `yarn install --frozen-lockfile`, `yarn format:check`, `yarn lint`, `yarn typecheck`, `yarn test` (430 tests), `yarn desktop:build`, `yarn desktop:package`, and `yarn desktop:make` passed. `yarn desktop:distribute --target linux-x64` produced an AppImage and `latest-linux.yml`. Both review cycles are recorded in [the review record](../../reviews/2026-10-25-in-app-updates.md).

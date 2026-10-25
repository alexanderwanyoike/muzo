# 0009: Explicit desktop updates

Status: Accepted

## Context

Users want an update notice in the library sidebar, downloading when clicked and replacing the installed application only after Restart to install. Existing AppImage menu and dock entries refer to the original executable path. Muzo's macOS releases are currently unsigned.

## Decision

Use electron-updater 6.8.11 with the existing electron-builder 26.18.0 pipeline and a fixed public GitHub provider. Disable automatic downloads, install-on-quit, prereleases, downgrades and differential downloads. A small application service owns the update state; the Electron adapter owns downloading, native confirmation and installation. Expected failures use Result values.

Windows x64 uses NSIS. Linux AppImages use electron-updater for HTTP downloading and SHA-512 verification, then copy the verified file into a temporary file in the current AppImage's directory, retain permissions and rename atomically over the existing path. The upstream AppImage installation path unlinks the old file before moving the new file and can change a versioned filename. The same-directory rename avoids both behaviours. Relaunch uses the retained outer AppImage path.

Unsigned macOS builds use an explicit release-page download. Electron's macOS automatic updater requires signed applications. Other Linux packaging formats also use the manual fallback. The UI states this through Download installer. No signing or operating system protections are bypassed.

The renderer can request only check, download and install. The main process selects every feed and executable location. Release assembly independently verifies the two update YAML files against the installers and stages their checksums along with the existing version-1 release manifest. Draft creation remains separate from publication.

## Consequences

A ready update remains pending until the user chooses Restart. Regular exit does not install it. Startup checks and six-hour checks are coalesced and cannot overwrite download or ready states. The first updater-enabled release requires one manual installation because v0.0.2 has no updater code. macOS in-place updates require a future signed and notarized release pipeline.

## References

- [electron-builder 26 auto-update documentation](https://www.electron.build/v26/docs/features/auto-update/)
- [electron-updater AppImage implementation](https://github.com/electron-userland/electron-builder/blob/v26.18.0/packages/electron-updater/src/AppImageUpdater.ts)
- Showbiz `frontend/electron/updates.ts`, `update-runtime.ts`, and `docs/automatic-updates.md`, MIT, Copyright 2026 Alexander Wanyoike. Its installed-build boundary and explicit update flow informed this implementation; no source was copied.

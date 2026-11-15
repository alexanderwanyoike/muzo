# Rust desktop rewrite

The rewrite tracks behaviour from Muzo 0.0.6 on `dev` (baseline `980252b`).
The release targets are Linux x64, Windows x64, and both Apple silicon and Intel
macOS. Develop focused feature branches against `epic/rust-rewrite`, then submit
the epic to `dev` after parity and packaged upgrade checks.

| Card | Parity checks |
| --- | --- |
| [001 Local playback and mini player](001-local-playback.md) | Play, pause, previous, next, end-of-track advance, seek, volume, retry, queue selection, track metadata and artwork. Native compact and queue layouts, keyboard controls, playback continuity and library geometry. |
| [002 Filesystem libraries and data compatibility](002-filesystem-libraries.md) | Native folder selection, multiple recursive sources, deduplication, source search and removal, scan status, startup reconciliation, live edits, moves and deletes. Open existing SQLite catalogs safely, including existing library/track IDs and historical schemas. |
| [003 Playlists, metadata and history](003-playlists-and-history.md) | Add tracks, remove and reorder playlist entries, play duplicate entries independently, edit and clear tags, cache and refresh artwork, record history and show play counts. Preserve migrated records. |
| [004 Dropbox sources](004-dropbox-sources.md) | OAuth with PKCE, OS credential storage and reconnect, folder browsing and multiple selections, account-scoped libraries, startup and configured polling, refresh, bounded audio cache, playback retry and disconnect. Verify migration from Electron encrypted credentials on all targets. |
| [005 Appearance and desktop interactions](005-appearance.md) | Saved materials, light/dark, all default and glow accents, mini player artwork treatment, layout and splitter preferences, system decorations and icons, settings and keyboard interactions. Import existing settings. |
| [006 Distribution and updates](006-distribution.md) | AppImage integration, NSIS and both DMGs, manifests and checksums, existing feed compatibility, explicit download and restart, Linux path retention and macOS manual updates. Install/upgrade smoke tests. |
| [007 Cutover](007-native-runtime-cutover.md) | Switch desktop entry points and release jobs to Rust, remove superseded Electron runtime, verify data upgrades from installed Muzo, and check the full parity matrix on each OS. |

Resolve release discrepancies against the existing feature tests, cards and
installer contracts. Verify audio and native window behaviour on running apps
before marking a platform done. Keep existing user data untouched during smoke
tests by using isolated profiles and disposable fixtures.

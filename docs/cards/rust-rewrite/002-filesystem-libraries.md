# 002: Port filesystem libraries and the SQLite catalogue

- [x] Add and select multiple folder sources, and find them by name.
- [x] Recursively index supported audio, reconcile on startup and react to file changes.
- [x] Keep existing track identities when files are scanned again and remove vanished paths.
- [x] Remove an indexed source without touching its backing folder or files.
- [x] Import existing Electron and Tauri SQLite catalogues and preserve their records.
- [x] Exercise a native folder, playback and restart with a disposable profile.

The port uses rusqlite's SQLite backup and transaction APIs and retains the
existing SQL migrations. Notify provides filesystem events, while a complete
rescan reconciles rename, delete and coalesced events against the saved catalogue.
WalkDir provides recursive traversal without following symlinks.

- Rusqlite (MIT): https://github.com/rusqlite/rusqlite
- Notify (CC0-1.0): https://github.com/notify-rs/notify
- WalkDir (MIT or Unlicense): https://github.com/BurntSushi/walkdir

Verified on Linux in a separate Xvfb display, private D-Bus session and silent
ALSA device. The native picker indexed nested audio, new files appeared while
the app was running, and the source reopened with all three tracks on restart.
Tests also cover duplicate additions, stable track IDs on rescan, deleted or
moved paths, source removal without touching music files, and read-only
migration from Electron and Tauri SQLite files.

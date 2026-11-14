# 002: Port filesystem libraries and the SQLite catalogue

- [ ] Add and select multiple folder sources, and find them by name.
- [ ] Recursively index supported audio, reconcile on startup and react to file changes.
- [ ] Keep existing track identities when files are scanned again and remove vanished paths.
- [ ] Remove an indexed source without touching its backing folder or files.
- [ ] Import existing Electron and Tauri SQLite catalogues and preserve their records.
- [ ] Exercise a native folder, playback and restart with a disposable profile.

The port uses rusqlite's SQLite backup and transaction APIs and retains the
existing SQL migrations. Notify provides filesystem events, while a complete
rescan reconciles rename, delete and coalesced events against the saved catalogue.
WalkDir provides recursive traversal without following symlinks.

- Rusqlite (MIT): https://github.com/rusqlite/rusqlite
- Notify (CC0-1.0): https://github.com/notify-rs/notify
- WalkDir (MIT or Unlicense): https://github.com/BurntSushi/walkdir

- Sprint: 02
- Type: Feature
- Status: In review
- Depends on: 021 (PR #48)

# Card 022 - Remove sources from Muzo

Make removal visible beside each sidebar source and in Indexed sources in Settings.
Use a native modal confirmation with Cancel initially focused, the source path,
and clear consequences. Remove only Muzo's catalogue data, never music files or
Dropbox contents. Preserve playlist entries as unavailable references.

## Acceptance criteria

- [x] Visible removal controls in the sidebar and Settings, with confirmation/cancel.
- [x] Atomically remove the library, tracks, play counts, overrides and Dropbox binding.
- [x] Preserve all source files, other libraries and playlist entries.
- [x] Stop filesystem watching and cancel pending reconciliation callbacks.
- [x] Prevent an in-flight scan from recreating removed catalogue tracks.
- [x] Stop/invalidate pending playback from the removed source.
- [x] Preserve playback from other sources and filter removed entries from its queue.
- [x] Update Sources and Settings immediately and discard stale refresh results.
- [x] Report typed failures without dismissing the confirmation or hiding the source.
- [ ] Full repository checks, visual verification and two review cycles pass.

## Established patterns

Use SQLite's explicit transaction pattern to publish catalogue changes together:
https://www.sqlite.org/lang_transaction.html (SQLite is public domain).
Use the existing MIT-licensed Chokidar adapter's asynchronous close operation and
cancel its debounce before closing: https://github.com/paulmillr/chokidar#methods--events
No external implementation is copied and no new dependency is needed.
The inner removal/watch-stopping ports keep framework and storage responsibilities
outside the application service. Typed Result commands remain distinct from legacy
throwing command compatibility.

## Verification

Real SQLite/IPC tests cover filesystem and Dropbox sources, ownership cleanup,
file preservation, retained playlist references and late scans. UI tests cover
confirmation/cancel, sidebar removal, retry after failure and refresh completion
races. Player tests cover pending audio invalidation and unaffected queue playback;
watcher tests cover closing, debounce cancellation and later registration.
Visual tests use a disposable in-memory fixture, never the user's real catalogue.

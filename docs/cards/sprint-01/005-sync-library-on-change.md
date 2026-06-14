- Sprint: 01
- Status: Pending
- Blocks: -

# Card 005 - Sync library on filesystem changes

## What to build

Muzo must stay in sync with its backing source. This card delivers the sync
requirement for `Filesystem` libraries: when a file is added, changed, renamed,
or removed under a library's location while Muzo is running, the track set
updates without a manual re-scan.

Vertical slice:

- **Domain**: a `SyncPolicy` value object (or similar) is optional; the domain only needs to express "a library is in sync" if a card calls for it. Keep this layer thin.
- **Application**: a `SyncLibrary` use case. Subscribes to filesystem events for a library's location and translates them into track upserts / removals via the existing `TrackRepository` and `FilesystemWalker`.
- **Infrastructure**: a `FilesystemWatcher` port (in application or domain) and a `notify`-crate-based implementation. The implementation must be swappable in tests with a fake that emits synthetic events.
- **Commands**: a `start_sync` Tauri command (and a matching `stop_sync` on shutdown). State is per-library.
- **UI**: a "live" indicator on libraries that are being watched. No need to surface every event to the user - the track count in card 004's UI simply updates.

This card only covers in-process watching. Cold-start reconciliation (catching
up on changes that happened while Muzo was closed) is handled by re-running
`ScanLibrary` on startup, which card 004 already provides.

## Acceptance criteria

- [ ] `SyncLibrary` use case in `application`, unit-tested with a fake watcher emitting synthetic events (added, changed, removed, renamed).
- [ ] A `FilesystemWatcher` port the use case depends on. The port's contract says what events look like, not which crate produces them.
- [ ] `notify`-based watcher implementation in `infrastructure`, with an integration test that creates, modifies, and deletes files in a temp dir and observes the corresponding track changes via the use case.
- [ ] Watcher is started per library when the app launches (or when a library is added) and stopped on shutdown. No orphaned threads.
- [ ] On startup, a `ScanLibrary` run reconciles any drift from when Muzo was closed. Documented in the README and in code.
- [ ] Removing a file outside the watched tree (e.g. while paused, if watcher is ever paused) is caught by the next scan; the system is eventually consistent.
- [ ] UI shows a "synced" state per library. Optional: surface errors (e.g. permission denied on a subfolder) without crashing the watcher.
- [ ] All verification commands green.

## Blocked by

- 004 - Scan filesystem library recursively.

## Notes for the implementer

- The `notify` crate is the standard choice; do not over-engineer. If platform
  differences surface, encapsulate them in the infrastructure impl, never in
  the application layer.
- Decide explicitly whether to watch recursively (yes) and whether to follow
  symlinks (consistent with card 004's decision).
- Watchers can be lossy under heavy churn. The startup reconciliation scan is
  the safety net - call this out in tests.
- Debouncing rapid saves (e.g. tag editor saving a file) belongs in the
  infrastructure impl, not the use case.
- This card does not cover Dropbox sync. Dropbox is a polling + manual refresh
  model (decided - see the sprint 01 README), not a filesystem watcher, and
  lands in sprint 02.

- Sprint: 02
- Type: Fix
- Status: Backend done, UX deferred until after Electron migration
- Blocks: 007

# Card 006 - Automatic filesystem library sync

## Current status

Backend reconciliation and watcher behavior are implemented and covered by the
current Rust suite. The listening-view UX is not complete: `Scan` is still
visible in the main library flow, and sync state/errors are not surfaced as a
quiet Settings-only fallback. That user-facing polish is deferred until after
the Electron migration so it is implemented once in the new shell.

## What to build

Remove manual rescan from the normal listening workflow. Once a user has
added a filesystem library, Muzo should keep that library current without
the user pressing Scan every time the app starts or every time music is added
to the folder.

This card is the user-facing fix for the post-playback problem: the app can
play tracks now, but the library model still feels like a one-shot import.
It should behave like a media library.

## What the user can do after this card

1. Add a filesystem library once.
2. Quit Muzo.
3. Add, edit, or remove music files under that library folder.
4. Reopen Muzo and see the library reconciled automatically.
5. Add a new file while Muzo is running and see it appear without using a
   manual rescan control.

Manual refresh can remain as a fallback action in Settings, but it must not
be the primary path.

## Vertical slice

- **Domain**: no required new entity. Add a sync status value object only if
  the UI needs to distinguish idle, syncing, synced, and failed states.
- **Application**:
  - Add an automatic library reconciliation use case that scans all persisted
    filesystem libraries on startup.
  - Add a per-library watch/sync use case for files added, changed, renamed,
    or removed while Muzo is running.
- **Infrastructure**:
  - Use the existing scanner for cold-start reconciliation.
  - Add a filesystem watcher adapter using `notify`, wrapped behind an
    application-level port.
  - Debounce bursts so a single album copy or tag-edit save does not trigger
    a storm of scans.
- **Commands / app state**:
  - Start reconciliation and watchers during app startup after repositories
    are ready.
  - Start watching a newly added filesystem library immediately.
  - Stop watchers cleanly on shutdown.
- **UI**:
  - Remove Scan as a prominent library-row action.
  - Show quiet sync state where useful, such as "Syncing", "Synced", or a
    recoverable error in Settings.
  - Keep manual refresh in Settings as a recovery action only.

## Acceptance criteria

- [x] On app startup, every persisted `Filesystem` library is reconciled
      automatically without a user action.
- [x] Reconciliation removes tracks whose files no longer exist.
- [x] Reconciliation inserts newly added audio files under the library root.
- [x] Reconciliation updates metadata for changed files without duplicating
      tracks.
- [x] While Muzo is running, adding a supported audio file under a watched
      library causes it to appear in the track list without pressing Scan.
- [x] While Muzo is running, deleting a file removes it from the track list
      without pressing Scan.
- [x] Rapid filesystem events are debounced and coalesced.
- [x] Watchers are per-library and are not duplicated after refreshes,
      navigation, or React re-renders.
- [x] Watchers stop on shutdown. No orphaned threads or leaked tasks.
- [ ] Manual refresh is available from Settings only and is clearly a fallback.
- [ ] Errors such as permission denied are surfaced without crashing playback
      or the library view.
- [x] Unit tests cover the application sync use case with a fake watcher.
- [x] Integration tests cover add and delete events in a temporary filesystem
      library. Change/rename-specific coverage is deferred with the UX polish.
- [ ] UI tests assert the main listening view does not depend on a visible
      Scan button.
- [x] Verification covered by the current test suites.

## Blocked by

- Sprint 01 card 004 - Scan filesystem library recursively.
- Sprint 02 card 001 - Formal SQLite migrations is recommended first if this
  card adds sync state tables.

## Notes for the implementer

- Treat startup reconciliation as the safety net. Filesystem watchers can miss
  events under heavy churn or while the app is closed.
- Keep the domain free of `notify` types. Translate infrastructure events into
  application-level sync events.
- Prefer scanning the affected subtree or file when the watcher gives enough
  information. Fall back to a full library scan when the event is ambiguous.
- Do not make the main listening surface an import/sync control panel. Library
  management belongs in Settings.
- Dropbox libraries are not part of this card. Dropbox sync remains polling
  plus manual refresh.

## Out of scope

- Writing metadata back to audio files.
- Dropbox sync.
- Conflict resolution beyond "the filesystem is the source of truth".
- Smart import rules.

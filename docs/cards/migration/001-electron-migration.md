- Track: Migration
- Type: Architecture
- Status: In progress

# Card 001 - Migrate desktop shell from Tauri/Rust to Electron/TypeScript

## What to build

Move Muzo's desktop shell from Tauri plus Rust to Electron plus TypeScript.
The goal is to remove Rust from the product codebase, keep the existing user
behavior, and make future desktop work happen in the same TypeScript stack as
the frontend and planned mobile/PWA work.

This card should preserve the current SQLite database shape and command names
while the migration is in progress. The existing Rust tests are the behavioral
spec for the TypeScript port.

## Why

Muzo no longer needs a Rust backend for its current feature set. The backend is
mostly filesystem scanning, SQLite persistence, metadata reading, Dropbox HTTP,
watching folders, and feeding browser audio. Those are all normal Electron main
process responsibilities. Keeping them in Rust slows feature work and creates a
second language stack for a product that is otherwise TypeScript-heavy.

## Current baseline

Implemented before this migration:

- Filesystem libraries can be added and listed.
- Local folders can be scanned recursively into tracks.
- Local tracks can be played through browser audio.
- Startup reconciliation and filesystem watching exist in the backend.
- SQLite migrations, playlists backend, play history, metadata editing, and
  Dropbox backend foundations exist.

Deferred until after the migration:

- Playlist UI.
- Dropbox account connection, token storage, polling, downloads, and playback.
- Sync UX polish that removes Scan from the main listening flow.
- Metadata editing UI/test polish.

## Ground rules

- Keep the same SQLite database path where practical, or add an explicit
  migration note if Electron must resolve the app data path differently.
- Keep command names and DTO shapes stable during the port.
- Add a frontend runtime bridge first so React does not import Tauri or
  Electron directly.
- Port one vertical slice at a time with Vitest node-environment tests
  mirroring the Rust tests.
- Do not add new product features during the migration unless they are needed
  to prove parity.
- Do not delete `src-tauri/` until Electron passes the agreed parity checks.

## Suggested phases

### Phase 0 - Runtime bridge and Electron shell

- Add a local bridge module used by `apps/desktop/src/api.ts`.
- Implement Tauri and Electron bridge adapters behind the same interface.
- Add Electron main and preload entry points.
- Boot the existing React app in Electron.
- Port one read command, preferably `list_libraries`, against the same SQLite
  database.

### Phase 1 - Core library backend

- Port domain/value behavior for libraries and tracks into TypeScript.
- Port SQLite migrations and repositories.
- Port add/list library, scan library, list tracks, and metadata reader.
- Keep tests beside the TypeScript modules.

### Phase 2 - Playback and history

- Replace the Rust audio stream server with an Electron-safe audio source
  strategy.
- Port `prepare_track_audio_source`, play history, and play count commands.
- Verify play, pause, seek, volume, and play counting in Electron.

### Phase 3 - Sync and metadata editing

- Port startup reconciliation and filesystem watching.
- Port metadata override commands.
- Preserve override behavior across rescans.

### Phase 4 - Playlists and Dropbox backend

- Port playlist backend commands.
- Port Dropbox OAuth/client/catalog backend code.
- Leave Dropbox product UI and playback deferred unless explicitly pulled into
  the migration parity scope.

### Phase 5 - Cutover and deletion

- Make Electron the default desktop dev/build path.
- Update docs, package scripts, and agent instructions.
- Remove Tauri, Rust source, Cargo files, and Rust verification commands.

## Progress notes

Completed Electron migration slices:

- Runtime bridge and Electron shell boot the existing React app.
- Electron command backend is modularised around commands, interfaces,
  infrastructure, IPC, and Awilix composition.
- Electron command parity exists for `add_library`, `list_libraries`, and
  `list_tracks`.
- Electron SQLite migrations create the current Rust schema and adopt legacy
  inline library/track schemas without losing data.

Remaining parity work:

- `scan_library`.
- Metadata reader and filesystem walker.
- `prepare_track_audio_source`.
- `edit_track_metadata` and `clear_track_metadata_override`.
- `record_track_play` and `list_track_play_counts`.
- Startup reconciliation and filesystem watching.
- Playlist commands.
- Dropbox backend.
- Tauri/Rust removal after verified Electron parity.

## Acceptance criteria

- [ ] React code calls a local runtime bridge, not Tauri or Electron directly.
- [ ] Electron app boots the current desktop UI in development.
- [ ] Existing local-library workflows work in Electron: add, list, scan, play,
      edit metadata, record play counts.
- [ ] Startup reconciliation and filesystem watching work in Electron.
- [ ] SQLite data created by the Rust/Tauri app remains readable.
- [ ] TypeScript tests cover the behavior currently covered by Rust unit and
      integration tests.
- [ ] `src-tauri/` is deleted only after Electron parity is verified.
- [ ] Documentation and scripts describe Electron as the desktop shell.

## Out of scope

- New playlist UI.
- Dropbox account connection and playback.
- Mobile/PWA app.
- MusicBrainz or internet metadata lookup.
- Batch metadata editing.

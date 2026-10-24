- Sprint: 02
- Type: Feature
- Status: Done
- Blocks: -

# Card 004 - Playlists core model

## What to build

Add Muzo's playlist foundation. Users should be able to create named
playlists and maintain an ordered set of tracks in each playlist. This card
is the backend core model and persistence slice; richer playlist UI can land
after the domain and storage boundaries are stable.

## What the user can do after this card

1. Create a playlist with a display name.
2. List persisted playlists.
3. Add tracks to a playlist in a stable order.
4. Remove tracks from a playlist.
5. Reorder playlist entries.
6. Restart the app and keep playlist contents and ordering.

## Vertical slice

- **Domain**:
  - Add a `Playlist` aggregate with value objects for id, name, entry id, and
    position.
  - Preserve insertion order and expose reordering behavior through aggregate
    methods.
  - Define a repository trait in the domain.
- **Application**:
  - Add create/list/add/remove/reorder use cases for playlists.
- **Infrastructure**:
  - Add SQLite migrations for playlists and playlist entries.
  - Implement the playlist repository in SQLite.
- **Commands**:
  - Add thin Tauri command handlers and DTOs for the use cases.
- **UI**:
  - Out of scope for this first core slice.

## Acceptance criteria

- [x] Playlist domain model has no Tauri, serde, SQLite, filesystem, HTTP, or
      frontend dependency.
- [x] Creating a playlist requires a non-empty name.
- [x] Playlist entries keep a stable ordered position.
- [x] Adding a track appends it after existing entries.
- [x] Removing an entry compacts the remaining positions.
- [x] Reordering entries persists the new order.
- [x] SQLite migrations create playlist and playlist-entry tables.
- [x] SQLite repository persists playlists and entries across repository
      instances.
- [x] Application use cases cover create, list, add track, remove entry, and
      reorder.
- [x] Tauri commands expose the playlist use cases through DTOs.
- [x] Tests cover domain behavior, application orchestration, repository
      persistence, migrations, and command DTO mapping.
- [x] All verification commands green.

## Blocked by

- Sprint 02 card 001 - Formal SQLite migrations.

## Notes for the implementer

- Do not store playlist entries as a serialized blob. Use a proper
  playlist-entry table so ordering and future metadata can evolve.
- A playlist entry references a `TrackId`; this card does not need to enforce
  foreign keys across library boundaries yet.
- Generate separate entry ids so duplicate appearances of the same track can
  be represented later. This card can allow duplicate tracks.
- Keep UI out unless the backend slice is already complete and the design is
  obvious.

## Out of scope

- Playlist cover art.
- Smart playlists.
- Drag-and-drop UI.
- Cross-device playlist sync.
- Collaborative playlists.

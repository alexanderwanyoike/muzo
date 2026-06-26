- Sprint: 02
- Type: Feature
- Status: Pending
- Blocks: -

# Card 005 - Play history and play counts

## What to build

Record track play events and expose per-track play counts. A play should be
recorded after the audio engine successfully starts a track, then the library
view should show the updated count for each track.

## What the user can do after this card

1. Play a track and have Muzo record that play.
2. Restart the app and keep recorded play history.
3. See play counts next to tracks in the selected library.

## Vertical slice

- **Domain**:
  - Add play-history value objects and a repository trait.
- **Application**:
  - Add use cases to record a track play and list play counts for a library.
- **Infrastructure**:
  - Add a SQLite migration for play history.
  - Implement the play-history repository in SQLite.
- **Commands**:
  - Add thin Tauri command handlers and DTOs.
- **UI**:
  - Record a play when playback successfully starts.
  - Display play counts in the track list.

## Acceptance criteria

- [x] Domain play-history model has no Tauri, serde, SQLite, filesystem, HTTP,
      or frontend dependency.
- [x] A successful track start records one play event.
- [x] Failed source loading or rejected playback does not record a play.
- [x] Play history persists across app restarts.
- [x] Play counts are grouped per track for the selected library.
- [x] The track list shows the current play count for each track.
- [x] SQLite migrations create the play-history table.
- [x] Tests cover domain behavior, application orchestration, repository
      persistence, migration shape, command DTO mapping, and frontend playback
      recording/count display.
- [x] All verification commands green.

## Notes for the implementer

- Store play events, not only a counter, so future history views can group by
  day, album, artist, or playlist.
- Use the existing track and library identifiers. This card does not need to
  validate cross-library track existence before recording a play.
- Keep playback recording idempotent per successful new track start. Resume or
  pause toggles should not add a new event.

- Sprint: 02
- Type: Feature
- Status: Done
- Blocks: -

# Card 012 - Play playlist entries

## What to build

Let users start playback from the Playlists view. Playlist entries already have
create, add, remove, and reorder actions; this card connects resolved entries
to the existing audio player and now-playing bar.

## What the user can do after this card

1. Open Playlists.
2. Play a playlist entry whose backing track still exists.
3. Pause or resume the current track from the playlist row.
4. See unavailable playlist entries disabled instead of failing at playback
   time.

## Vertical slice

- **Infrastructure**: enrich playlist entries with playback metadata from
  SQLite.
- **UI**: add compact play controls beside playlist entries.
- **App integration**: route playlist playback through the existing audio
  player.
- **Tests**: cover repository enrichment, play, toggle, and unavailable entry
  behavior.

## Acceptance criteria

- [x] Playlist entries include library id and duration when their track still
      exists.
- [x] Playable playlist entries call the app playback callback with a `TrackDto`.
- [x] The current playlist entry toggles through the existing player.
- [x] Entries whose track row no longer exists cannot be played.
- [x] Tests cover repository playback metadata and UI play/toggle behavior.
- [x] Verification covered by the current test suites.

## Out of scope

- Queueing an entire playlist.
- Auto-advance to the next playlist entry.
- Dropbox downloads for remote playlist entries.

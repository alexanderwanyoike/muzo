- Sprint: 02
- Type: Feature
- Status: Done
- Blocks: -

# Card 010 - Remove playlist entries

## What to build

Let users inspect playlist contents and remove an entry from a playlist. Playlist
entry ordering and playlist playback remain separate follow-ups.

## What the user can do after this card

1. Open Playlists.
2. See playlist entries with track labels when Muzo can resolve them.
3. Remove an entry from a playlist.
4. See the playlist count and entry list update without restarting.
5. See a clear error if removal fails.

## Vertical slice

- **Infrastructure**: enrich playlist entries with track labels from SQLite.
- **Frontend API**: add a typed bridge for `remove_playlist_entry`.
- **UI**: render playlist entries and a remove action per entry.

## Acceptance criteria

- [x] Playlist entries display track title and artist when available.
- [x] Playlist entries fall back to track id if the track row no longer exists.
- [x] Removing an entry calls `remove_playlist_entry` with playlist and entry ids.
- [x] Removal updates the playlist count and entry list immediately.
- [x] Empty playlist contents are clear and not treated as an error.
- [x] Remove failures are surfaced without removing the entry locally.
- [x] Tests cover repository label enrichment and UI success/error behavior.
- [x] Verification covered by the current test suites.

## Out of scope

- Reordering playlist entries.
- Playlist playback.
- Deleting playlists.
- Confirm dialogs.

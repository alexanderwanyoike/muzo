- Sprint: 02
- Type: Feature
- Status: Done
- Blocks: -

# Card 009 - Add tracks to playlists from the song list

## What to build

Let users add a visible library track to an existing playlist from the song
list. The playlist backend already supports membership; this card exposes that
capability without adding playlist ordering, removal, or playlist playback yet.

## What the user can do after this card

1. Open a library in Songs.
2. Choose a track row.
3. Open the playlist action for that track.
4. Pick an existing playlist.
5. Add the track and see confirmation.

## Vertical slice

- **Frontend API**: add typed bridge functions for playlist membership.
- **UI**: add a compact per-track playlist action in the existing track list.
- **Tests**: cover successful add, empty playlist state, and error paths.

## Acceptance criteria

- [x] Track rows expose an add-to-playlist action.
- [x] Opening the action loads existing playlists through `list_playlists`.
- [x] Adding a track calls `add_track_to_playlist` with playlist and track ids.
- [x] The UI confirms the add without leaving the song list.
- [x] No-playlist state is clear and does not call `add_track_to_playlist`.
- [x] Load and add errors are surfaced without closing the picker.
- [x] UI tests cover success, empty, and error behavior.
- [x] Verification covered by the current test suites.

## Out of scope

- Creating playlists inline from the song list.
- Removing tracks from playlists.
- Reordering playlist entries.
- Playing from playlist order.
- Preventing duplicate playlist entries.

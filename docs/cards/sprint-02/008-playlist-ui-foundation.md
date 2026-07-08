- Sprint: 02
- Type: Feature
- Status: Done
- Blocks: -

# Card 008 - Playlist UI foundation

## What to build

Expose the existing playlist backend in the desktop UI. Users should be able to
open a Playlists view, see persisted playlists, and create a new playlist by
name.

This is the first playlist UI slice. Track membership, reordering, playback from
playlist order, cover art, and drag-and-drop stay out of scope.

## What the user can do after this card

1. Open Playlists from the main navigation.
2. See existing playlists and their track counts.
3. Create a playlist with a non-empty name.
4. See the new playlist appear without restarting the app.
5. See clear errors if playlist loading or creation fails.

## Vertical slice

- **Application / commands**: use the existing playlist command handlers.
- **Frontend API**: add typed playlist bridge functions.
- **UI**: add a Playlists view with create and list interactions.

## Acceptance criteria

- [x] Playlists appears in the primary navigation.
- [x] Opening Playlists loads persisted playlists through `list_playlists`.
- [x] Empty playlist state is clear and does not look like an error.
- [x] Creating a playlist calls `create_playlist` with a trimmed name.
- [x] The created playlist appears in the list immediately.
- [x] Blank playlist names are rejected in the UI before calling the backend.
- [x] Load and create errors are surfaced without leaving the view.
- [x] UI tests cover list, create, blank-name, and error behavior.
- [x] Verification covered by the current test suites.

## Out of scope

- Adding tracks to playlists.
- Removing tracks from playlists.
- Reordering playlist entries.
- Playing a playlist as a queue.
- Playlist artwork.

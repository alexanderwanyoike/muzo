- Sprint: 02
- Type: Feature
- Status: Done
- Blocks: -

# Card 011 - Reorder playlist entries

## What to build

Let users change playlist entry order from the Playlists view. The backend
command already exists, so this card adds the typed frontend bridge and UI
controls that call it.

## What the user can do after this card

1. Open Playlists.
2. Move a playlist entry up or down.
3. See the entry order update without restarting.
4. See boundary controls disabled when an entry cannot move further.
5. See a clear error if reordering fails.

## Vertical slice

- **Frontend API**: add a typed bridge for `reorder_playlist_entries`.
- **UI**: render compact move controls per playlist entry.
- **Tests**: cover success, boundary disabling, and failure behavior.

## Acceptance criteria

- [x] Moving an entry calls `reorder_playlist_entries` with playlist id and
      ordered entry ids.
- [x] Successful reordering updates the visible entry order immediately.
- [x] First entries cannot move up and last entries cannot move down.
- [x] Reorder failures are surfaced without changing the local order.
- [x] Tests cover reorder success, boundary disabling, and failure behavior.
- [x] Verification covered by the current test suites.

## Out of scope

- Drag and drop ordering.
- Playlist playback.
- Deleting playlists.

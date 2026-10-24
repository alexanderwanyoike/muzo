- Sprint: 02
- Type: Feature
- Status: Done
- Blocks: -

# Card 016 - Playlist queueing and auto-advance

Start a playlist or a selected playlist entry and continue through its playable
entries. The queue is a snapshot of the playlist at the time playback starts.

## Acceptance criteria

- [x] Play playlist starts all available entries in their visible order.
- [x] Selecting an entry starts at that entry and keeps previous entries reachable.
- [x] Unavailable entries are omitted and empty playlists cannot be started.
- [x] Repeated tracks remain distinct queue entries and each start records a play.
- [x] Track completion advances automatically and stops at the end without looping.
- [x] Previous and next controls respect the queue boundaries.
- [x] Selecting a song directly clears the playlist queue.
- [x] Queued local and Dropbox tracks use the same audio preparation command.
- [x] Source failures stop playback and allow manual skipping without recording a play.
- [x] Old audio pauses during preparation and pending playback is invalidated on cleanup.
- [x] All repository verification commands pass (189 tests; Linux zip and deb).

## Pattern and scope

Use the established array-plus-current-index player pattern, advancing from the
media end event. Howler's MIT-licensed player example demonstrates this pattern:
https://github.com/goldfire/howler.js/blob/master/examples/player/player.js
No implementation code is copied and no new audio framework is introduced.

Muzo deliberately stops at the end of the snapshot. Playlist edits affect the next
queue started, preserving the current listening order. Queue persistence, shuffle,
repeat modes, and a separate queue editor are outside this card.

## Dependencies and tests

Depends on PR #42, which includes the Dropbox audio preparation path, and transitively
on #41 and #16. Merge in that order, then retarget this PR to dev. Tests exercise the
player hook, playlist controls, now-playing controls, and the assembled React app
through its runtime boundary.

### Review corrections

- Start a queue when selecting a playlist entry whose song is already playing directly.
- Clear the queue when selecting its current song from Songs, while retaining explicit player pause/resume behavior.
- Exercise both transitions through the assembled app and retain the legacy standalone playlist control behavior.
- Re-ran the required checks with 216 tests and Linux zip/deb outputs.

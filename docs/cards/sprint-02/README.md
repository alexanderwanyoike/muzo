# Sprint 02 - Dropbox libraries and playlist foundations

Goal: prove the architecture abstraction holds by adding a second library
source (Dropbox), then start layering playlist and history features on top
of sprint 01's playable library.

Sprint 02 also opens with a tech debt card pulled forward from sprint 01
review: formalising SQLite migrations before more tables pile up.

## Cards

| #   | Card | Type | Status | Blocks |
| --- | --- | --- | --- | --- |
| 001 | [Formal SQLite migrations](./001-formal-sqlite-migrations.md) | Tech debt | Done | 002, 003, 004 |
| 002 | [Dropbox OAuth and API client](./002-dropbox-oauth-and-api-client.md) | Feature | Done | 003 |
| 003 | [Dropbox library adapter](./003-dropbox-library-adapter.md) | Feature | Done | - |
| 004 | [Playlists core model](./004-playlists-core-model.md) | Feature | Done | - |
| 005 | [Play history and play counts](./005-play-history-and-play-counts.md) | Feature | Done | - |
| 006 | [Automatic filesystem library sync](./006-automatic-filesystem-library-sync.md) | Fix | Done | 007 |
| 007 | [ID3 metadata and library editing](./007-id3-metadata-and-library-editing.md) | Feature | Done | - |
| 008 | [Playlist UI foundation](./008-playlist-ui-foundation.md) | Feature | Done | - |
| 009 | [Add tracks to playlists from the song list](./009-add-tracks-to-playlists-ui.md) | Feature | Done | - |
| 010 | [Remove playlist entries](./010-remove-playlist-entries-ui.md) | Feature | Done | - |
| 011 | [Reorder playlist entries](./011-reorder-playlist-entries-ui.md) | Feature | Done | - |

## Close-out note

The sprint 02 local-library slices are complete and covered by the current
desktop test suite. Remaining user-facing product work:

- Dropbox account connection, token persistence, polling, downloads, and
  playback.
- Playlist playback.

The Dropbox sync model is decided: **polling on a configurable interval plus
a manual "refresh now" action**. Webhooks were rejected because a desktop
app is not a reliable callback target. To be reconfirmed and ADR'd when
card 002 lands.

## Why playback moved out of sprint 02

Originally playback was sprint 02 card 005. Sprint 01 review surfaced that
without playback, sprint 01 produces a library organiser rather than a music
player - the product's whole reason for existing. Playback was pulled into
sprint 01 as card 006 so sprint 01 is one complete vertical slice.

Sprint 02's playback-adjacent work is now **play history and play counts**
(sprint 02 card 005), which builds on sprint 01's playback by recording
events into a new table. The actual `<audio>` element and now-playing UI
do not need to come back here.

## Out of scope for sprint 02

- Mobile app (sprint 03+)
- Sync conflict resolution beyond last-write-wins
- Smart playlists / rules-based playlists

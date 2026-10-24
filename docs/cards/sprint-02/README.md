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
| 012 | [Play playlist entries](./012-play-playlist-entries-ui.md) | Feature | Done | - |
| 013 | [Dropbox token persistence](./013-dropbox-token-persistence.md) | Feature | Done | 014 |
| 014 | [Dropbox account connection](./014-dropbox-account-connection.md) | Feature | Done | 015 |
| 015 | [Dropbox libraries, sync and playback](./015-dropbox-libraries-sync-and-playback.md) | Feature | Done | - |
| 016 | [Playlist queueing and auto-advance](./016-playlist-playback-queue.md) | Feature | Done | - |
| 017 | [Desktop verification CI](./017-desktop-verification-ci.md) | Tech debt | Done | - |
| 018 | [Enforced code style](./018-enforced-code-style.md) | Tech debt | Done | - |
| 019 | [Playback source retry](./019-playback-source-retry.md) | Fix | Done | - |
| 020 | [Visual Dropbox folder picker](./020-dropbox-folder-picker.md) | Feature | Done | - |
| 021 | [Desktop materials and appearance](./021-desktop-appearance-themes.md) | Feature | Done | - |
| 022 | [Remove library sources](./022-remove-library-sources.md) | Feature | Done | - |
| 023 | [Embedded album artwork](./023-embedded-album-artwork.md) | Feature | Done | - |

| 024 | [Minimal desktop menus](./024-minimal-desktop-menu.md) | Feature | Done | - |
| 025 | [Searchable sources](./025-searchable-sources.md) | Feature | Done | - |

## Close-out note

The sprint 02 local-library slices are complete and covered by the current
desktop test suite. Dropbox account connection, polling, downloads, and playback are implemented.
Playlist queueing and auto-advance are implemented and verified. The completed
work was integrated into dev after explicit approval. The desktop UI follow-up
cards 020 through 023 are delivered in dependent PRs #47 through #50 and should
merge in that order after manual testing.

The Dropbox sync model is decided: **polling on a configurable interval plus
a manual "refresh now" action**. Webhooks were rejected because a desktop
app is not a reliable callback target. See ADR 0005 for the implemented decision.

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

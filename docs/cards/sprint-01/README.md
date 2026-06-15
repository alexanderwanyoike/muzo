# Sprint 01 - A usable local music player

Goal: a user can launch Muzo, register a filesystem folder as a library, see
its tracks scanned in recursively, hear a track play through their speakers,
and have new files picked up automatically. By the end of sprint 01, Muzo is
genuinely usable as a desktop music player for local files.

This sprint is a single vertical slice through every layer of the
architecture: domain -> application -> infrastructure -> commands -> UI.

## Cards

| #   | Card | Status | Blocks |
| --- | --- | --- | --- |
| 001 | [Scaffold desktop shell](./001-scaffold-desktop-shell.md) | Done | 002 |
| 002 | [Add a filesystem library](./002-add-filesystem-library.md) | Done | 003, 004 |
| 003 | [List libraries in the UI](./003-list-libraries.md) | Done | - |
| 004 | [Scan filesystem library recursively](./004-scan-filesystem-library.md) | Done | 005, 006 |
| 005 | [Sync library on filesystem changes](./005-sync-library-on-change.md) | Pending | - |
| 006 | [Play tracks from a library](./006-play-tracks.md) | Pending | - |

## Why Dropbox is in sprint 02, not sprint 01

Dropbox is a first-class library kind in the product; the `LibraryKind::Dropbox`
variant ships in the domain enum as part of card 002 so we do not retrofit it
later. The Dropbox *adapter* lands in sprint 02, deliberately, for two reasons:

1. **It validates the abstraction.** Sprint 01 introduces `FilesystemWalker`,
   `AudioMetadataReader`, and `TrackRepository` as ports. The Dropbox adapter
   in sprint 02 is the first consumer that is not a filesystem - if it cannot
   reuse the same ports cleanly, the abstraction is wrong and we find out
   immediately rather than after the codebase is larger.
2. **It brings its own complexity.** OAuth flow, token storage and refresh,
   and a polling-based sync model are each enough to anchor a card. Mixing
   them into sprint 01 would force a horizontal split (filesystem layer done
   end-to-end, then Dropbox layer done end-to-end) rather than vertical
   slices through every layer.

The sprint 02 sync model is decided: **polling on a configurable interval,
plus a manual "refresh now" action**. Webhooks were rejected because a
desktop app is not reliably addressable as a callback endpoint. That decision
will be reconfirmed and ADR'd when the first Dropbox card lands.

## Why playback was pulled into sprint 01

Originally playback was a sprint 02 card. Sprint 01 review surfaced that
without playback, sprint 01 produces a library organiser, not a music
player - the product's whole reason for existing. Sprint 01 should be one
complete vertical slice that delivers a usable artifact, not a partial one.
Card 006 (play tracks) is now the closing card of sprint 01.

## Out of scope for sprint 01

- Dropbox libraries (sprint 02 - see above)
- Playlists / queue / play counts (sprint 02)
- Mobile app (sprint 03+)

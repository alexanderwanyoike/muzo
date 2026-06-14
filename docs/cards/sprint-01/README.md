# Sprint 01 - Desktop shell and filesystem libraries

Goal: a user can launch Muzo, register a filesystem folder as a library, see
its tracks scanned in recursively, and have new files picked up automatically.
No playback, no playlists, no Dropbox yet - those land in sprint 02.

This sprint proves every layer of the architecture is wired end-to-end with
a real vertical slice, not just the scaffold.

## Cards

| #   | Card | Status | Blocks |
| --- | --- | --- | --- |
| 001 | [Scaffold desktop shell](./001-scaffold-desktop-shell.md) | Done in this PR | 002 |
| 002 | [Add a filesystem library](./002-add-filesystem-library.md) | Pending | 003, 004 |
| 003 | [List libraries in the UI](./003-list-libraries.md) | Pending | - |
| 004 | [Scan filesystem library recursively](./004-scan-filesystem-library.md) | Pending | 005 |
| 005 | [Sync library on filesystem changes](./005-sync-library-on-change.md) | Pending | - |

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

## Out of scope for sprint 01

- Dropbox libraries (sprint 02 - see above)
- Playlists (sprint 02)
- Audio playback (sprint 02)
- Mobile app (sprint 03+)

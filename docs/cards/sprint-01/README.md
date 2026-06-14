# Sprint 01 - Desktop shell and filesystem libraries

Goal: a user can launch Muzo, register a filesystem folder as a library, see
its tracks scanned in recursively, and have new files picked up automatically.
No playback, no playlists, no Dropbox yet - those land in later sprints.

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

## Out of scope for sprint 01

- Dropbox libraries (sprint 02)
- Playlists (sprint 02)
- Audio playback (sprint 02)
- Mobile app (sprint 03+)

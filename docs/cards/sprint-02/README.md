# Sprint 02 - Dropbox libraries and playlist foundations

Goal: prove the architecture abstraction holds by adding a second library
source (Dropbox), then start layering playlist and playback features on top.

Sprint 02 also opens with a tech debt card pulled forward from sprint 01
review: formalising SQLite migrations before more tables pile up.

## Cards

| #   | Card | Type | Blocks |
| --- | --- | --- | --- |
| 001 | [Formal SQLite migrations](./001-formal-sqlite-migrations.md) | Tech debt | 003, 004, 005 |
| 002 | [Dropbox OAuth and API client](./002-dropbox-oauth-and-api-client.md) (planned) | Feature | 003 |
| 003 | [Dropbox library adapter](./003-dropbox-library-adapter.md) (planned) | Feature | - |
| 004 | [Playlists core model](./004-playlists-core-model.md) (planned) | Feature | - |
| 005 | [Audio playback](./005-audio-playback.md) (planned) | Feature | - |

The Dropbox sync model is decided: **polling on a configurable interval plus
a manual "refresh now" action**. Webhooks were rejected because a desktop
app is not a reliable callback target. To be reconfirmed and ADR'd when
card 002 lands.

## Out of scope for sprint 02

- Mobile app (sprint 03+)
- Sync conflict resolution beyond last-write-wins
- Smart playlists / rules-based playlists

# 003: Port playlists, metadata editing and play history

- [x] Create and browse playlists, add tracks, and change the order of entries.
- [x] Remove an entry without removing its track, and play duplicate entries independently.
- [x] Edit and clear metadata overrides while preserving tags in audio files.
- [x] Record successful playback and show existing and new play counts.
- [x] Preserve playlist entry IDs, metadata overrides and play events imported from Electron.
- [x] Exercise playlists, edits and the queue in the running desktop.

The existing SQLite playlist, metadata override and history tables define the
data contract. Keep the order and metadata update rules transactional and use
the existing playback session as the source of playback events.

The packaged comparisons exercised [playlist playback](https://github.com/alexanderwanyoike/muzo/pull/122),
[repeated entries](https://github.com/alexanderwanyoike/muzo/pull/115),
[metadata editing](https://github.com/alexanderwanyoike/muzo/pull/165)
and [entry rollback and retry](https://github.com/alexanderwanyoike/muzo/pull/197).
The installed [profile upgrade checks](https://github.com/alexanderwanyoike/muzo/pull/183)
compare playlist entry and track IDs, metadata and play events across versions.

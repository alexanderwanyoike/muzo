# 003: Port playlists, metadata editing and play history

- [ ] Create and browse playlists, add tracks, and change the order of entries.
- [ ] Remove an entry without removing its track, and play duplicate entries independently.
- [ ] Edit and clear metadata overrides while preserving tags in audio files.
- [ ] Record successful playback and show existing and new play counts.
- [ ] Preserve playlist entry IDs, metadata overrides and play events imported from Electron.
- [ ] Exercise playlists, edits and the queue in the running desktop.

The existing SQLite playlist, metadata override and history tables define the
data contract. Keep the order and metadata update rules transactional and use
the existing playback session as the source of playback events.

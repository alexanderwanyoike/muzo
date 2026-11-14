CREATE TABLE IF NOT EXISTS play_history (
    id TEXT PRIMARY KEY NOT NULL,
    library_id TEXT NOT NULL,
    track_id TEXT NOT NULL,
    played_at_unix_seconds INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_play_history_library_track
ON play_history(library_id, track_id);

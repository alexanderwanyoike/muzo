CREATE TABLE IF NOT EXISTS libraries (
    id       TEXT PRIMARY KEY NOT NULL,
    name     TEXT NOT NULL,
    kind     TEXT NOT NULL,
    location TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS tracks (
    id               TEXT PRIMARY KEY NOT NULL,
    library_id       TEXT NOT NULL,
    title            TEXT NOT NULL,
    artist           TEXT NOT NULL,
    duration_seconds INTEGER NOT NULL,
    file_path        TEXT NOT NULL,
    file_size        INTEGER NOT NULL,
    file_mtime       INTEGER NOT NULL,
    UNIQUE(library_id, file_path)
);

ALTER TABLE tracks ADD COLUMN album TEXT;
ALTER TABLE tracks ADD COLUMN track_number INTEGER;
ALTER TABLE tracks ADD COLUMN disc_number INTEGER;
ALTER TABLE tracks ADD COLUMN genre TEXT;
ALTER TABLE tracks ADD COLUMN year INTEGER;
ALTER TABLE tracks ADD COLUMN override_title TEXT;
ALTER TABLE tracks ADD COLUMN override_artist TEXT;
ALTER TABLE tracks ADD COLUMN override_album TEXT;
ALTER TABLE tracks ADD COLUMN override_track_number INTEGER;
ALTER TABLE tracks ADD COLUMN override_disc_number INTEGER;
ALTER TABLE tracks ADD COLUMN override_genre TEXT;
ALTER TABLE tracks ADD COLUMN override_year INTEGER;

CREATE TABLE IF NOT EXISTS playlists (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS playlist_entries (
    id TEXT PRIMARY KEY NOT NULL,
    playlist_id TEXT NOT NULL,
    track_id TEXT NOT NULL,
    position INTEGER NOT NULL,
    UNIQUE(playlist_id, position),
    FOREIGN KEY(playlist_id) REFERENCES playlists(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS play_history (
    id TEXT PRIMARY KEY NOT NULL,
    library_id TEXT NOT NULL,
    track_id TEXT NOT NULL,
    played_at_unix_seconds INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_play_history_library_track
ON play_history(library_id, track_id);

PRAGMA user_version = 5;

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

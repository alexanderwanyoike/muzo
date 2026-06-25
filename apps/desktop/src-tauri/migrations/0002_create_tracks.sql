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

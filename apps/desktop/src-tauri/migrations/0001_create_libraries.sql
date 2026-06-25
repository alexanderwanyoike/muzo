CREATE TABLE IF NOT EXISTS libraries (
    id       TEXT PRIMARY KEY NOT NULL,
    name     TEXT NOT NULL,
    kind     TEXT NOT NULL,
    location TEXT NOT NULL
);

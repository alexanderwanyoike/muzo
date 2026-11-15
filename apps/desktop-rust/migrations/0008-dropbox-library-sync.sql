CREATE TABLE dropbox_library_accounts (
  library_id TEXT PRIMARY KEY REFERENCES libraries(id) ON DELETE CASCADE,
  account_id TEXT NOT NULL
);
CREATE TABLE dropbox_sync_settings (
  singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
  interval_minutes INTEGER NOT NULL CHECK (interval_minutes BETWEEN 1 AND 60)
);
INSERT INTO dropbox_sync_settings (singleton, interval_minutes) VALUES (1, 5);

CREATE TABLE IF NOT EXISTS dropbox_accounts (
    account_id                           TEXT PRIMARY KEY NOT NULL,
    encrypted_access_token               TEXT NOT NULL,
    encrypted_refresh_token              TEXT NOT NULL,
    access_token_expires_at_unix_seconds INTEGER NOT NULL,
    connected_at_unix_seconds            INTEGER NOT NULL
);

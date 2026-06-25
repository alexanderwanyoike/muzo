//! Versioned SQLite schema migrations.
//!
//! To add a migration, append one `M::up(SQL).comment("YYYY-MM-DD-short-slug")`
//! entry to `MIGRATIONS`.

use lazy_static::lazy_static;
use rusqlite_migration::{Migrations, M};

const CREATE_LIBRARIES_SQL: &str = "CREATE TABLE IF NOT EXISTS libraries (
                    id       TEXT PRIMARY KEY NOT NULL,
                    name     TEXT NOT NULL,
                    kind     TEXT NOT NULL,
                    location TEXT NOT NULL
                );";

const CREATE_TRACKS_SQL: &str = "CREATE TABLE IF NOT EXISTS tracks (
                    id               TEXT PRIMARY KEY NOT NULL,
                    library_id       TEXT NOT NULL,
                    title            TEXT NOT NULL,
                    artist           TEXT NOT NULL,
                    duration_seconds INTEGER NOT NULL,
                    file_path        TEXT NOT NULL,
                    file_size        INTEGER NOT NULL,
                    file_mtime       INTEGER NOT NULL,
                    UNIQUE(library_id, file_path)
                );";

lazy_static! {
    pub static ref MIGRATIONS: Migrations<'static> = Migrations::new(vec![
        M::up(CREATE_LIBRARIES_SQL).comment("2026-06-15-create-libraries"),
        M::up(CREATE_TRACKS_SQL).comment("2026-06-15-create-tracks"),
    ]);
}

pub fn current_schema_sql() -> &'static str {
    concat!(
        "CREATE TABLE IF NOT EXISTS libraries (\n",
        "                    id       TEXT PRIMARY KEY NOT NULL,\n",
        "                    name     TEXT NOT NULL,\n",
        "                    kind     TEXT NOT NULL,\n",
        "                    location TEXT NOT NULL\n",
        "                );\n",
        "CREATE TABLE IF NOT EXISTS tracks (\n",
        "                    id               TEXT PRIMARY KEY NOT NULL,\n",
        "                    library_id       TEXT NOT NULL,\n",
        "                    title            TEXT NOT NULL,\n",
        "                    artist           TEXT NOT NULL,\n",
        "                    duration_seconds INTEGER NOT NULL,\n",
        "                    file_path        TEXT NOT NULL,\n",
        "                    file_size        INTEGER NOT NULL,\n",
        "                    file_mtime       INTEGER NOT NULL,\n",
        "                    UNIQUE(library_id, file_path)\n",
        "                );"
    )
}

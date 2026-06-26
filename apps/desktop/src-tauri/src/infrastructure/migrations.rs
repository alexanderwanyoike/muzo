//! Versioned SQLite schema migrations.
//!
//! To add a migration, append one `M::up(SQL).comment("YYYY-MM-DD-short-slug")`
//! entry to `MIGRATIONS`.

use lazy_static::lazy_static;
use rusqlite_migration::{Migrations, M};

const CREATE_LIBRARIES_SQL: &str = include_str!("../../migrations/0001_create_libraries.sql");
const CREATE_TRACKS_SQL: &str = include_str!("../../migrations/0002_create_tracks.sql");
const TRACK_METADATA_OVERRIDES_SQL: &str =
    include_str!("../../migrations/0003_track_metadata_overrides.sql");
const CREATE_PLAYLISTS_SQL: &str = include_str!("../../migrations/0004_create_playlists.sql");

lazy_static! {
    pub static ref MIGRATIONS: Migrations<'static> = Migrations::new(vec![
        M::up(CREATE_LIBRARIES_SQL).comment("2026-06-15-create-libraries"),
        M::up(CREATE_TRACKS_SQL).comment("2026-06-15-create-tracks"),
        M::up(TRACK_METADATA_OVERRIDES_SQL).comment("2026-06-25-track-metadata-overrides"),
        M::up(CREATE_PLAYLISTS_SQL).comment("2026-06-25-create-playlists"),
    ]);
}

use std::num::NonZeroUsize;

use muzo_desktop_lib::infrastructure::migrations::{current_schema_sql, MIGRATIONS};
use rusqlite_migration::SchemaVersion;

#[test]
fn fresh_database_reaches_the_current_schema() {
    let mut conn = rusqlite::Connection::open_in_memory().unwrap();

    MIGRATIONS.to_latest(&mut conn).unwrap();

    assert_eq!(
        MIGRATIONS.current_version(&conn).unwrap(),
        SchemaVersion::Inside(NonZeroUsize::new(2).unwrap())
    );
    assert_eq!(user_version(&conn), 2);
    assert_eq!(
        table_columns(&conn, "libraries"),
        vec!["id", "name", "kind", "location"]
    );
    assert_eq!(
        table_columns(&conn, "tracks"),
        vec![
            "id",
            "library_id",
            "title",
            "artist",
            "duration_seconds",
            "file_path",
            "file_size",
            "file_mtime"
        ]
    );
}

#[test]
fn rerunning_migrations_against_current_schema_is_a_no_op() {
    let mut conn = rusqlite::Connection::open_in_memory().unwrap();
    MIGRATIONS.to_latest(&mut conn).unwrap();
    conn.execute(
        "INSERT INTO libraries (id, name, kind, location) VALUES (?1, ?2, ?3, ?4)",
        ("lib-1", "Music", "Filesystem", "/music"),
    )
    .unwrap();

    MIGRATIONS.to_latest(&mut conn).unwrap();

    assert_eq!(
        MIGRATIONS.current_version(&conn).unwrap(),
        SchemaVersion::Inside(NonZeroUsize::new(2).unwrap())
    );
    assert_eq!(
        conn.query_row("SELECT COUNT(*) FROM libraries", [], |row| row
            .get::<_, u32>(0))
            .unwrap(),
        1
    );
}

#[test]
fn legacy_inline_schema_database_is_adopted_without_losing_data() {
    let mut conn = rusqlite::Connection::open_in_memory().unwrap();
    conn.execute_batch(current_schema_sql()).unwrap();
    conn.execute(
        "INSERT INTO libraries (id, name, kind, location) VALUES (?1, ?2, ?3, ?4)",
        ("lib-1", "Music", "Filesystem", "/music"),
    )
    .unwrap();
    conn.execute(
        "INSERT INTO tracks (
            id, library_id, title, artist, duration_seconds, file_path, file_size, file_mtime
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        (
            "trk-1",
            "lib-1",
            "Song",
            "Artist",
            120,
            "/music/song.mp3",
            1024,
            42,
        ),
    )
    .unwrap();

    MIGRATIONS.to_latest(&mut conn).unwrap();

    assert_eq!(
        MIGRATIONS.current_version(&conn).unwrap(),
        SchemaVersion::Inside(NonZeroUsize::new(2).unwrap())
    );
    assert_eq!(
        conn.query_row("SELECT COUNT(*) FROM libraries", [], |row| row
            .get::<_, u32>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        conn.query_row("SELECT COUNT(*) FROM tracks", [], |row| row
            .get::<_, u32>(0))
            .unwrap(),
        1
    );
}

fn table_columns(conn: &rusqlite::Connection, table_name: &str) -> Vec<String> {
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({})", table_name))
        .unwrap();
    stmt.query_map([], |row| row.get::<_, String>(1))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
}

fn user_version(conn: &rusqlite::Connection) -> u32 {
    conn.query_row("PRAGMA user_version", [], |row| row.get(0))
        .unwrap()
}

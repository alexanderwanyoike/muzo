use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use rusqlite::Connection;

use crate::catalogue::Catalogue;

#[test]
fn it_imports_a_tauri_catalogue_without_touching_the_legacy_database() {
    let root = PathBuf::from(std::env::temp_dir()).join(format!(
        "muzo-catalog-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&root).unwrap();
    let legacy = root.join("legacy.sqlite");
    let current = root.join("current.sqlite");
    let database = Connection::open(&legacy).unwrap();
    database
        .execute_batch(include_str!(
            "../../desktop/electron/infrastructure/sqlite/fixtures/legacy-tauri-v5.sql"
        ))
        .unwrap();
    database.execute_batch("
        INSERT INTO libraries VALUES ('library-1', 'Records', 'filesystem', '/music');
        INSERT INTO tracks (id, library_id, title, artist, duration_seconds, file_path, file_size, file_mtime)
        VALUES ('track-1', 'library-1', 'Imported song', 'Artist', 60, '/music/a.mp3', 42, 45);
        INSERT INTO playlists VALUES ('playlist-1', 'Collection');
        INSERT INTO playlist_entries VALUES ('entry-1', 'playlist-1', 'track-1', 0);
    ").unwrap();
    drop(database);
    let before = fs::read(&legacy).unwrap();

    let catalog = Catalogue::open(&current, Some(&legacy)).unwrap();
    let library = catalog.libraries().unwrap().remove(0);
    assert_eq!(library.name, "Records");
    let track = catalog.tracks(&library.id).unwrap().remove(0);
    assert_eq!(track.id, "track-1");
    assert_eq!(track.title, "Imported song");
    assert_eq!(fs::read(&legacy).unwrap(), before);
    drop(catalog);
    let database = Connection::open(&current).unwrap();
    let version: u32 = database
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .unwrap();
    assert_eq!(version, 8);
    let entry: String = database
        .query_row("SELECT track_id FROM playlist_entries", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(entry, "track-1");
    drop(database);
    fs::remove_dir_all(root).unwrap();
}

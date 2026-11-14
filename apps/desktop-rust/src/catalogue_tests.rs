use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use rusqlite::Connection;

use crate::catalogue::Catalogue;

#[test]
fn it_imports_a_tauri_catalogue_without_touching_the_legacy_database() {
    let root = std::env::temp_dir().join(format!(
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

#[test]
fn it_reconciles_nested_music_and_removes_a_source_without_touching_audio() {
    let root = std::env::temp_dir().join(format!("muzo-scan-{}", uuid::Uuid::new_v4()));
    let music = root.join("Music");
    fs::create_dir_all(music.join("disc")).unwrap();
    let file = music.join("disc/tone.mp3");
    fs::copy(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tone.mp3"),
        &file,
    )
    .unwrap();
    fs::write(music.join("notes.txt"), "ignored").unwrap();

    let mut catalogue = Catalogue::open(&root.join("muzo.sqlite"), None).unwrap();
    let library = catalogue.add_folder(&music).unwrap();
    let tracks = catalogue.tracks(&library.id).unwrap();
    assert_eq!(tracks.len(), 1);
    assert_eq!(tracks[0].path, file);
    assert!(catalogue.add_folder(&music).is_err());
    catalogue.refresh(&library.id).unwrap();
    assert_eq!(catalogue.tracks(&library.id).unwrap()[0].id, tracks[0].id);

    let moved = music.join("moved.mp3");
    fs::rename(&file, &moved).unwrap();
    catalogue.refresh(&library.id).unwrap();
    let tracks = catalogue.tracks(&library.id).unwrap();
    assert_eq!(tracks.len(), 1);
    assert_eq!(tracks[0].path, moved);
    catalogue.remove(&library.id).unwrap();
    assert!(catalogue.libraries().unwrap().is_empty());
    assert!(moved.is_file());
    drop(catalogue);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn it_reopens_the_same_rust_catalogue_across_launches() {
    let root = std::env::temp_dir().join(format!("muzo-launch-{}", uuid::Uuid::new_v4()));
    let music = root.join("Music");
    fs::create_dir_all(&music).unwrap();
    let paths = crate::desktop::library_sources::CataloguePaths::in_directory(&root);
    let snapshot = crate::desktop::library_sources::run(
        &paths,
        crate::desktop::library_sources::Operation::Add(music),
    )
    .unwrap();
    let id = snapshot.libraries[0].id.clone();
    let restarted = crate::desktop::library_sources::run(
        &paths,
        crate::desktop::library_sources::Operation::Startup,
    )
    .unwrap();
    assert_eq!(restarted.libraries[0].id, id);
    assert_eq!(restarted.selected.as_deref(), Some(id.as_str()));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn it_imports_electron_metadata_overrides_and_play_events() {
    let root = std::env::temp_dir().join(format!("muzo-electron-catalog-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    let legacy = root.join("muzo.sqlite");
    let current = root.join("muzo-rust.sqlite");
    let database = Connection::open(&legacy).unwrap();
    database
        .execute_batch(include_str!(
            "../../desktop/electron/infrastructure/sqlite/fixtures/legacy-tauri-v5.sql"
        ))
        .unwrap();
    database.execute_batch(include_str!("../../desktop/electron/infrastructure/sqlite/migrations/0006-create-dropbox-accounts.sql")).unwrap();
    database
        .execute_batch(include_str!(
            "../../desktop/electron/infrastructure/sqlite/migrations/0007-dropbox-client-id.sql"
        ))
        .unwrap();
    database
        .execute_batch(include_str!(
            "../../desktop/electron/infrastructure/sqlite/migrations/0008-dropbox-library-sync.sql"
        ))
        .unwrap();
    database.execute_batch("PRAGMA user_version = 8; CREATE TABLE electron_migrations (version INTEGER PRIMARY KEY, name TEXT, applied_at_unix_seconds INTEGER);
        INSERT INTO libraries VALUES ('dropbox-1', 'Cloud', 'dropbox', '/Cloud');
        INSERT INTO tracks (id, library_id, title, artist, duration_seconds, file_path, file_size, file_mtime, override_title)
            VALUES ('track-1','dropbox-1','Original','Artist',5,'/Cloud/tone.mp3',100,100,'Edited');
        INSERT INTO play_history VALUES ('play-1','dropbox-1','track-1',123);
    ").unwrap();
    drop(database);
    let original = fs::read(&legacy).unwrap();
    let catalogue = Catalogue::open(&current, Some(&legacy)).unwrap();
    assert_eq!(catalogue.libraries().unwrap()[0].kind, "dropbox");
    assert_eq!(catalogue.tracks("dropbox-1").unwrap()[0].title, "Edited");
    drop(catalogue);
    assert_eq!(fs::read(legacy).unwrap(), original);
    let imported = Connection::open(current).unwrap();
    assert_eq!(
        imported
            .query_row("SELECT track_id FROM play_history", [], |row| row
                .get::<_, String>(0))
            .unwrap(),
        "track-1"
    );
    drop(imported);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn it_reorders_duplicate_playlist_entries_and_edits_track_catalogue_metadata() {
    let root = std::env::temp_dir().join(format!("muzo-playlist-{}", uuid::Uuid::new_v4()));
    let music = root.join("Music");
    fs::create_dir_all(&music).unwrap();
    let audio = music.join("tone.mp3");
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tone.mp3");
    fs::copy(&fixture, &audio).unwrap();
    let mut catalogue = Catalogue::open(&root.join("muzo.sqlite"), None).unwrap();
    let library = catalogue.add_folder(&music).unwrap();
    let track = catalogue.tracks(&library.id).unwrap().remove(0);
    let playlist = catalogue.create_playlist("A set").unwrap();
    let first = catalogue.add_to_playlist(&playlist.id, &track.id).unwrap();
    let second = catalogue.add_to_playlist(&playlist.id, &track.id).unwrap();
    assert_ne!(first, second);
    catalogue
        .move_playlist_entry(&playlist.id, &second, -1)
        .unwrap();
    assert_eq!(
        catalogue.playlist_entries(&playlist.id).unwrap()[0].id,
        second
    );
    catalogue
        .remove_playlist_entry(&playlist.id, &second)
        .unwrap();
    assert_eq!(
        catalogue.playlist_entries(&playlist.id).unwrap()[0].id,
        first
    );

    catalogue
        .override_track(
            &track.id,
            &crate::catalogue::MetadataOverride {
                title: Some("Side A".into()),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(catalogue.tracks(&library.id).unwrap()[0].title, "Side A");
    catalogue.record_play(&track.id).unwrap();
    assert_eq!(catalogue.tracks(&library.id).unwrap()[0].plays, 1);
    catalogue
        .override_track(&track.id, &Default::default())
        .unwrap();
    assert_eq!(catalogue.tracks(&library.id).unwrap()[0].title, "tone");
    assert_eq!(fs::read(&fixture).unwrap(), fs::read(&audio).unwrap());
    drop(catalogue);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn it_browses_the_created_playlist_and_keeps_duplicate_entries_in_order() {
    use crate::desktop::library_sources::{CataloguePaths, Operation, run};

    let root = std::env::temp_dir().join(format!("muzo-playlist-view-{}", uuid::Uuid::new_v4()));
    let music = root.join("Music");
    fs::create_dir_all(&music).unwrap();
    fs::copy(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tone.mp3"),
        music.join("tone.mp3"),
    )
    .unwrap();
    let paths = CataloguePaths::in_directory(&root);
    let library = run(&paths, Operation::Add(music)).unwrap();
    let track = library.tracks[0].id.clone();
    let playlist = run(&paths, Operation::CreatePlaylist("Live Set".into())).unwrap();
    let id = playlist.selected_playlist.unwrap();
    let added = run(&paths, Operation::AddToPlaylist(id.clone(), track.clone())).unwrap();
    let first_entry = added.playlist_entries[0].id.clone();
    let added = run(&paths, Operation::AddToPlaylist(id, track)).unwrap();
    assert_eq!(added.playlist_entries.len(), 2);
    assert_eq!(added.playlist_entries[0].id, first_entry);
    assert_ne!(added.playlist_entries[1].id, first_entry);
    fs::remove_dir_all(root).unwrap();
}

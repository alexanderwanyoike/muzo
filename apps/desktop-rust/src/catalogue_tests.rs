use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use rusqlite::Connection;

use crate::catalogue::Catalogue;

#[test]
fn it_reports_a_missing_artwork_track_as_a_catalogue_error() {
    let root = tempfile::tempdir().unwrap();
    let catalogue = Catalogue::open(&root.path().join("muzo.sqlite"), None).unwrap();

    assert!(matches!(
        catalogue.artwork_revision("library", "missing"),
        Err(crate::catalogue::CatalogueError::UnknownTrack)
    ));
}

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
        .execute_batch(include_str!("../tests/fixtures/legacy-tauri-v5.sql"))
        .unwrap();
    database.execute_batch("
        INSERT INTO libraries VALUES ('library-1', 'Records', 'Filesystem', '/music');
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
    assert_eq!(library.kind, "filesystem");
    let track = catalog.tracks(&library.id).unwrap().remove(0);
    assert_eq!(track.id, "track-1");
    assert_eq!(track.title, "Imported song");
    assert_eq!(fs::read(&legacy).unwrap(), before);
    drop(catalog);
    let database = Connection::open(&current).unwrap();
    let version: u32 = database
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .unwrap();
    assert_eq!(version, 9);
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
    assert_eq!(tracks[0].path, file.canonicalize().unwrap());
    assert!(catalogue.add_folder(&music).is_err());
    catalogue.refresh(&library.id).unwrap();
    assert_eq!(catalogue.tracks(&library.id).unwrap()[0].id, tracks[0].id);

    let moved = music.join("moved.mp3");
    fs::rename(&file, &moved).unwrap();
    catalogue.refresh(&library.id).unwrap();
    let tracks = catalogue.tracks(&library.id).unwrap();
    assert_eq!(tracks.len(), 1);
    assert_eq!(tracks[0].path, moved.canonicalize().unwrap());
    catalogue.remove(&library.id).unwrap();
    assert!(catalogue.libraries().unwrap().is_empty());
    assert!(moved.is_file());
    drop(catalogue);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn it_preserves_the_original_artist_fallback_when_rescanning_migrated_tracks() {
    use crate::desktop::library_sources::{CataloguePaths, Operation, run};

    let root = tempfile::tempdir().unwrap();
    let music = root.path().join("Music");
    fs::create_dir(&music).unwrap();
    fs::copy(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tone.mp3"),
        music.join("tone.mp3"),
    )
    .unwrap();

    let legacy = root.path().join("muzo.sqlite");
    let mut catalogue = Catalogue::open(&legacy, None).unwrap();
    let library = catalogue.add_folder(&music).unwrap();
    let original = catalogue.tracks(&library.id).unwrap().remove(0);
    drop(catalogue);

    let connection = Connection::open(&legacy).unwrap();
    connection
        .execute(
            "UPDATE tracks SET artist = ?1 WHERE id = ?2",
            ["Unknown Artist", &original.id],
        )
        .unwrap();
    drop(connection);
    let legacy_bytes = fs::read(&legacy).unwrap();

    let paths = CataloguePaths::in_directory(root.path());
    run(&paths, Operation::Startup).unwrap();
    let snapshot = run(&paths, Operation::Select(library.id)).unwrap();
    assert_eq!(snapshot.tracks.len(), 1);
    assert_eq!(snapshot.tracks[0].id, original.id);
    assert_eq!(snapshot.tracks[0].artist, "Unknown Artist");
    assert_eq!(fs::read(&legacy).unwrap(), legacy_bytes);
}

#[test]
fn it_reopens_the_same_rust_catalogue_across_launches() {
    let root = std::env::temp_dir().join(format!("muzo-launch-{}", uuid::Uuid::new_v4()));
    let music = root.join("Music");
    fs::create_dir_all(&music).unwrap();
    fs::write(
        music.join("tone.mp3"),
        include_bytes!("../tests/fixtures/tone.mp3"),
    )
    .unwrap();
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
    assert_eq!(restarted.selected, None);
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
        .execute_batch(include_str!("../tests/fixtures/legacy-tauri-v5.sql"))
        .unwrap();
    database
        .execute_batch(include_str!(
            "../migrations/0006-create-dropbox-accounts.sql"
        ))
        .unwrap();
    database
        .execute_batch(include_str!("../migrations/0007-dropbox-client-id.sql"))
        .unwrap();
    database
        .execute_batch(include_str!("../migrations/0008-dropbox-library-sync.sql"))
        .unwrap();
    database.execute_batch("PRAGMA user_version = 8; CREATE TABLE electron_migrations (version INTEGER PRIMARY KEY, name TEXT, applied_at_unix_seconds INTEGER);
        INSERT INTO libraries VALUES ('dropbox-1', 'Cloud', 'Dropbox', '/Cloud');
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

#[test]
fn it_rejects_invalid_legacy_schema_without_publishing_a_catalogue() {
    let root = std::env::temp_dir().join(format!("muzo-invalid-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    let legacy = root.join("muzo.sqlite");
    let current = root.join("muzo-rust.sqlite");
    let database = Connection::open(&legacy).unwrap();
    database.pragma_update(None, "user_version", 8).unwrap();
    drop(database);

    assert!(Catalogue::open(&current, Some(&legacy)).is_err());
    assert!(!current.exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn it_preserves_unchanged_metadata_overrides_in_the_editor() {
    use crate::catalogue::MetadataOverride;
    use crate::desktop::library_sources::{CataloguePaths, Operation, run};

    let root = tempfile::tempdir().unwrap();
    let music = root.path().join("Music");
    fs::create_dir_all(&music).unwrap();
    fs::copy(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tone.mp3"),
        music.join("tone.mp3"),
    )
    .unwrap();
    let paths = CataloguePaths::in_directory(root.path());
    let library = run(&paths, Operation::Add(music)).unwrap();
    let id = &library.tracks[0].id;
    let catalogue = paths.open().unwrap();
    catalogue
        .override_track(
            id,
            &MetadataOverride {
                genre: Some("Ambient".into()),
                track_number: Some(7),
                ..Default::default()
            },
        )
        .unwrap();

    let mut override_values = catalogue.track_overrides(id).unwrap();
    override_values.title = Some("Edited".into());
    catalogue.override_track(id, &override_values).unwrap();
    let overrides = catalogue.track_overrides(id).unwrap();
    assert_eq!(overrides.genre.as_deref(), Some("Ambient"));
    assert_eq!(overrides.track_number, Some(7));
}

#[test]
fn it_opens_the_editor_with_effective_tags_and_saved_overrides() {
    use crate::desktop::library_sources::{CataloguePaths, Operation, run};

    let root = tempfile::tempdir().unwrap();
    let paths = CataloguePaths::in_directory(root.path());
    drop(paths.open().unwrap());
    let database = Connection::open(root.path().join("muzo-rust.sqlite")).unwrap();
    database.execute(
        "INSERT INTO libraries (id, name, kind, location) VALUES ('source','Records','Dropbox','/Records')",
        [],
    ).unwrap();
    database.execute(
        "INSERT INTO tracks (id,library_id,title,artist,album,track_number,disc_number,genre,year,
         duration_seconds,file_path,file_size,file_mtime,override_artist,override_track_number)
         VALUES ('song','source','Silt and Light','Artist','Field Recordings',2,1,'Ambient',2020,
         120,'/Records/track.mp3',100,1,'Mara Vale',7)",
        [],
    ).unwrap();

    let snapshot = run(
        &paths,
        Operation::LoadTrackEditor("song".into(), "source".into()),
    )
    .unwrap();
    let editor = snapshot.editor.unwrap();
    assert!(editor.overridden);
    assert_eq!(
        editor.fields,
        [
            "Silt and Light",
            "Mara Vale",
            "Field Recordings",
            "7",
            "1",
            "Ambient",
            "2020"
        ]
    );
}

#[test]
fn it_opens_and_clears_a_fractional_override_from_an_electron_catalogue() {
    use crate::desktop::library_sources::{CataloguePaths, Operation, run};

    let root = tempfile::tempdir().unwrap();
    let paths = CataloguePaths::in_directory(root.path());
    drop(paths.open().unwrap());
    let database = Connection::open(root.path().join("muzo-rust.sqlite")).unwrap();
    database.execute_batch(
        "INSERT INTO libraries (id, name, kind, location) VALUES ('source','Records','Dropbox','/Records');
         INSERT INTO tracks (id,library_id,title,artist,duration_seconds,file_path,file_size,file_mtime,override_year)
         VALUES ('song','source','Song','Artist',120,'/Records/track.mp3',100,1,1979.5);",
    ).unwrap();
    let snapshot = run(
        &paths,
        Operation::LoadTrackEditor("song".into(), "source".into()),
    )
    .unwrap();
    let editor = snapshot.editor.unwrap();
    assert!(editor.overridden);
    assert_eq!(editor.fields[6], "1979.5");

    run(
        &paths,
        Operation::EditTrack("song".into(), "source".into(), Default::default()),
    )
    .unwrap();
    assert!(
        paths
            .open()
            .unwrap()
            .track_overrides("song")
            .unwrap()
            .year
            .is_none()
    );
}

#[test]
fn it_keeps_the_custom_name_when_indexing_a_filesystem_source() {
    let root = tempfile::tempdir().unwrap();
    let folder = root.path().join("raw-folder-name");
    fs::create_dir(&folder).unwrap();
    fs::write(
        folder.join("tone.mp3"),
        include_bytes!("../tests/fixtures/tone.mp3"),
    )
    .unwrap();
    let path = root.path().join("catalogue.sqlite");
    let mut catalogue = Catalogue::open(&path, None).unwrap();

    let library = catalogue
        .add_named_folder(&folder, "Weekend records")
        .unwrap();
    assert_eq!(library.name, "Weekend records");
    assert_eq!(catalogue.libraries().unwrap(), vec![library]);
    assert!(catalogue.add_named_folder(&folder, "Other").is_err());
}

#[test]
fn it_keeps_a_folder_without_supported_audio_out_of_the_catalogue() {
    let root = tempfile::tempdir().unwrap();
    let folder = root.path().join("Empty Music");
    fs::create_dir(&folder).unwrap();
    fs::write(folder.join("readme.txt"), "not audio").unwrap();
    let mut catalogue = Catalogue::open(&root.path().join("muzo.sqlite"), None).unwrap();

    let result = catalogue.add_named_folder(&folder, "Empty Music");

    assert!(
        result.is_err(),
        "a library with no supported tracks was added"
    );
    assert!(catalogue.libraries().unwrap().is_empty());

    let nested = folder.join("Disc 1");
    fs::create_dir(&nested).unwrap();
    fs::write(
        nested.join("Recording.mp3"),
        include_bytes!("../tests/fixtures/tone.mp3"),
    )
    .unwrap();
    let source = catalogue.add_named_folder(&folder, "Empty Music").unwrap();
    assert_eq!(catalogue.tracks(&source.id).unwrap().len(), 1);
}

use std::path::PathBuf;

use muzo_desktop_lib::domain::library::LibraryId;
use muzo_desktop_lib::domain::track::{
    FileMtime, FileSize, Track, TrackArtist, TrackDuration, TrackFilePath, TrackId,
    TrackRepository, TrackTitle,
};
use muzo_desktop_lib::infrastructure::migrations::MIGRATIONS;
use muzo_desktop_lib::infrastructure::sqlite_track_repository::SqliteTrackRepository;

fn make_track(library: &str, path: &str, title: &str) -> Track {
    Track::new(
        TrackId(format!("trk-{}", path)),
        LibraryId(library.into()),
        TrackTitle(title.into()),
        TrackArtist("Artist".into()),
        TrackDuration(200),
        TrackFilePath(PathBuf::from(path)),
        FileSize(1_000),
        FileMtime(100),
    )
}

fn migrated_connection() -> rusqlite::Connection {
    let mut conn = rusqlite::Connection::open_in_memory().unwrap();
    MIGRATIONS.to_latest(&mut conn).unwrap();
    conn
}

#[test]
fn upsert_inserts_a_new_track_that_can_be_listed_for_its_library() {
    let conn = migrated_connection();
    let repo = SqliteTrackRepository::new(conn);

    repo.upsert(&make_track("lib-1", "/m/a.mp3", "A")).unwrap();

    let listed = repo
        .list_for_library(&LibraryId("lib-1".into()))
        .expect("list should succeed");

    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].title(), &TrackTitle("A".into()));
}

#[test]
fn upsert_on_conflicting_library_and_path_preserves_the_id_and_updates_other_fields() {
    let conn = migrated_connection();
    let repo = SqliteTrackRepository::new(conn);

    let original = make_track("lib-1", "/m/a.mp3", "Old Title");
    let original_id = original.id().0.clone();
    repo.upsert(&original).unwrap();

    let mut updated = make_track("lib-1", "/m/a.mp3", "New Title");
    updated = Track::new(
        TrackId("DIFFERENT_ID".into()),
        updated.library_id().clone(),
        TrackTitle("New Title".into()),
        TrackArtist("New Artist".into()),
        updated.duration(),
        updated.file_path().clone(),
        FileSize(2_000),
        updated.file_mtime(),
    );
    repo.upsert(&updated).unwrap();

    let listed = repo.list_for_library(&LibraryId("lib-1".into())).unwrap();
    assert_eq!(listed.len(), 1, "rescan must not duplicate the row");
    assert_eq!(
        listed[0].id().0,
        original_id,
        "original id must be preserved on update"
    );
    assert_eq!(listed[0].title(), &TrackTitle("New Title".into()));
    assert_eq!(listed[0].artist(), &TrackArtist("New Artist".into()));
    assert_eq!(listed[0].file_size(), FileSize(2_000));
}

#[test]
fn list_for_library_only_returns_tracks_for_the_requested_library() {
    let conn = migrated_connection();
    let repo = SqliteTrackRepository::new(conn);

    repo.upsert(&make_track("lib-1", "/m/a.mp3", "A")).unwrap();
    repo.upsert(&make_track("lib-1", "/m/b.mp3", "B")).unwrap();
    repo.upsert(&make_track("lib-2", "/m/c.mp3", "C")).unwrap();

    let lib1 = repo.list_for_library(&LibraryId("lib-1".into())).unwrap();
    let lib2 = repo.list_for_library(&LibraryId("lib-2".into())).unwrap();

    assert_eq!(lib1.len(), 2);
    assert_eq!(lib2.len(), 1);
}

#[test]
fn delete_by_library_and_path_removes_only_the_matching_track() {
    let conn = migrated_connection();
    let repo = SqliteTrackRepository::new(conn);

    repo.upsert(&make_track("lib-1", "/m/a.mp3", "A")).unwrap();
    repo.upsert(&make_track("lib-1", "/m/b.mp3", "B")).unwrap();

    repo.delete_by_library_and_path(
        &LibraryId("lib-1".into()),
        &TrackFilePath(PathBuf::from("/m/a.mp3")),
    )
    .unwrap();

    let listed = repo.list_for_library(&LibraryId("lib-1".into())).unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].title(), &TrackTitle("B".into()));
}

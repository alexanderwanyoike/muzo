use muzo_desktop_lib::domain::library::LibraryId;
use muzo_desktop_lib::domain::play_history::{
    PlayHistoryEntry, PlayHistoryId, PlayHistoryRepository, PlayedAtUnixSeconds,
};
use muzo_desktop_lib::domain::track::TrackId;
use muzo_desktop_lib::infrastructure::migrations::MIGRATIONS;
use muzo_desktop_lib::infrastructure::sqlite_play_history_repository::SqlitePlayHistoryRepository;

fn migrated_connection(path: &std::path::Path) -> rusqlite::Connection {
    let mut conn = rusqlite::Connection::open(path).unwrap();
    MIGRATIONS.to_latest(&mut conn).unwrap();
    conn
}

#[test]
fn play_counts_survive_a_new_repository_instance_grouped_by_track() {
    let temp = tempfile::tempdir().unwrap();
    let db_path = temp.path().join("muzo.sqlite");
    let writer = SqlitePlayHistoryRepository::new(migrated_connection(&db_path));

    writer
        .record_play(&PlayHistoryEntry::new(
            PlayHistoryId("play-1".into()),
            LibraryId("lib-1".into()),
            TrackId("track-1".into()),
            PlayedAtUnixSeconds(10),
        ))
        .unwrap();
    writer
        .record_play(&PlayHistoryEntry::new(
            PlayHistoryId("play-2".into()),
            LibraryId("lib-1".into()),
            TrackId("track-1".into()),
            PlayedAtUnixSeconds(20),
        ))
        .unwrap();
    writer
        .record_play(&PlayHistoryEntry::new(
            PlayHistoryId("play-3".into()),
            LibraryId("lib-1".into()),
            TrackId("track-2".into()),
            PlayedAtUnixSeconds(15),
        ))
        .unwrap();
    writer
        .record_play(&PlayHistoryEntry::new(
            PlayHistoryId("other-library".into()),
            LibraryId("lib-2".into()),
            TrackId("track-1".into()),
            PlayedAtUnixSeconds(30),
        ))
        .unwrap();
    drop(writer);

    let reader = SqlitePlayHistoryRepository::new(rusqlite::Connection::open(&db_path).unwrap());
    let counts = reader
        .play_counts_for_library(&LibraryId("lib-1".into()))
        .unwrap();

    assert_eq!(counts.len(), 2);
    assert_eq!(counts[0].track_id(), &TrackId("track-1".into()));
    assert_eq!(counts[0].count(), 2);
    assert_eq!(counts[0].last_played_at(), Some(PlayedAtUnixSeconds(20)));
    assert_eq!(counts[1].track_id(), &TrackId("track-2".into()));
    assert_eq!(counts[1].count(), 1);
    assert_eq!(counts[1].last_played_at(), Some(PlayedAtUnixSeconds(15)));
}

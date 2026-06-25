use muzo_desktop_lib::domain::playlist::{
    Playlist, PlaylistEntryId, PlaylistId, PlaylistName, PlaylistRepository,
};
use muzo_desktop_lib::domain::track::TrackId;
use muzo_desktop_lib::infrastructure::migrations::MIGRATIONS;
use muzo_desktop_lib::infrastructure::sqlite_playlist_repository::SqlitePlaylistRepository;

fn migrated_connection(path: &std::path::Path) -> rusqlite::Connection {
    let mut conn = rusqlite::Connection::open(path).unwrap();
    MIGRATIONS.to_latest(&mut conn).unwrap();
    conn
}

#[test]
fn playlist_entries_survive_a_new_repository_instance_in_order() {
    let temp = tempfile::tempdir().unwrap();
    let db_path = temp.path().join("muzo.sqlite");

    let mut playlist = Playlist::new(
        PlaylistId("playlist-1".into()),
        PlaylistName::new("Road Trip".into()).unwrap(),
    );
    playlist.add_track(PlaylistEntryId("entry-1".into()), TrackId("track-1".into()));
    playlist.add_track(PlaylistEntryId("entry-2".into()), TrackId("track-2".into()));
    playlist
        .reorder_entries(&[
            PlaylistEntryId("entry-2".into()),
            PlaylistEntryId("entry-1".into()),
        ])
        .unwrap();

    let writer = SqlitePlaylistRepository::new(migrated_connection(&db_path));
    writer.add(&playlist).unwrap();
    drop(writer);

    let reader = SqlitePlaylistRepository::new(rusqlite::Connection::open(&db_path).unwrap());
    let found = reader
        .find_by_id(playlist.id())
        .unwrap()
        .expect("playlist should be found after reopening the database");

    assert_eq!(found.id(), playlist.id());
    assert_eq!(found.name(), playlist.name());
    assert_eq!(
        found
            .entries()
            .iter()
            .map(|entry| (
                entry.id().0.as_str(),
                entry.track_id().0.as_str(),
                entry.position().0
            ))
            .collect::<Vec<_>>(),
        vec![("entry-2", "track-2", 0), ("entry-1", "track-1", 1)]
    );
}

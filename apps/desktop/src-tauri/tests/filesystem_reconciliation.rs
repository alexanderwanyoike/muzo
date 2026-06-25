use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use muzo_desktop_lib::application::reconcile_filesystem_libraries::reconcile_filesystem_libraries;
use muzo_desktop_lib::application::scan_library::{
    AudioMetadata, AudioMetadataReader, MetadataError,
};
use muzo_desktop_lib::domain::library::{
    Library, LibraryId, LibraryKind, LibraryLocation, LibraryName, LibraryRepository,
};
use muzo_desktop_lib::domain::track::TrackRepository;
use muzo_desktop_lib::infrastructure::migrations::MIGRATIONS;
use muzo_desktop_lib::infrastructure::sqlite_library_repository::SqliteLibraryRepository;
use muzo_desktop_lib::infrastructure::sqlite_track_repository::SqliteTrackRepository;
use muzo_desktop_lib::infrastructure::walkdir_walker::WalkdirWalker;

struct FakeMetadataReader {
    by_path: HashMap<PathBuf, AudioMetadata>,
}

impl FakeMetadataReader {
    fn with(entries: Vec<(PathBuf, AudioMetadata)>) -> Self {
        Self {
            by_path: entries.into_iter().collect(),
        }
    }
}

impl AudioMetadataReader for FakeMetadataReader {
    fn read(&self, path: &Path) -> Result<AudioMetadata, MetadataError> {
        self.by_path
            .get(path)
            .cloned()
            .ok_or_else(|| MetadataError::Io(format!("no fake metadata for {:?}", path)))
    }
}

#[test]
fn startup_reconciliation_inserts_and_removes_filesystem_tracks() {
    let temp = tempfile::tempdir().unwrap();
    let library_root = temp.path().join("library");
    fs::create_dir(&library_root).unwrap();
    let track_path = library_root.join("track.mp3");
    fs::write(&track_path, b"not-real-audio").unwrap();

    let db_dir = tempfile::tempdir().unwrap();
    let db_path = db_dir.path().join("muzo.sqlite");
    let (library_repository, track_repository) = sqlite_repositories(&db_path);
    let library_id = LibraryId("lib-1".into());
    library_repository
        .add(&Library::new(
            library_id.clone(),
            LibraryName("Music".into()),
            LibraryKind::Filesystem,
            LibraryLocation(library_root.display().to_string()),
        ))
        .unwrap();
    let metadata_reader = FakeMetadataReader::with(vec![(
        track_path.clone(),
        AudioMetadata {
            title: "Track".into(),
            artist: "Artist".into(),
            duration_seconds: 120,
        },
    )]);

    let inserted = reconcile_filesystem_libraries(
        &library_repository,
        &track_repository,
        &WalkdirWalker::new(),
        &metadata_reader,
    )
    .unwrap();

    assert_eq!(inserted.libraries_reconciled, 1);
    assert!(inserted.failures.is_empty());
    assert_eq!(
        track_repository
            .list_for_library(&library_id)
            .unwrap()
            .len(),
        1
    );

    fs::remove_file(&track_path).unwrap();
    let removed = reconcile_filesystem_libraries(
        &library_repository,
        &track_repository,
        &WalkdirWalker::new(),
        &metadata_reader,
    )
    .unwrap();

    assert_eq!(removed.libraries_reconciled, 1);
    assert!(removed.failures.is_empty());
    assert!(
        track_repository
            .list_for_library(&library_id)
            .unwrap()
            .is_empty(),
        "startup reconciliation should remove tracks whose files disappeared"
    );
}

fn sqlite_repositories(db_path: &Path) -> (SqliteLibraryRepository, SqliteTrackRepository) {
    let mut library_conn = rusqlite::Connection::open(db_path).unwrap();
    MIGRATIONS.to_latest(&mut library_conn).unwrap();
    let track_conn = rusqlite::Connection::open(db_path).unwrap();
    (
        SqliteLibraryRepository::new(library_conn),
        SqliteTrackRepository::new(track_conn),
    )
}

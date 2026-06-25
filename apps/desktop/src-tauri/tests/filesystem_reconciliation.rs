use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use muzo_desktop_lib::application::reconcile_filesystem_libraries::reconcile_filesystem_libraries;
use muzo_desktop_lib::application::scan_library::{
    scan_library, AudioMetadata, AudioMetadataReader, MetadataError,
};
use muzo_desktop_lib::application::watch_filesystem_libraries::watch_filesystem_libraries;
use muzo_desktop_lib::domain::library::{
    Library, LibraryId, LibraryKind, LibraryLocation, LibraryName, LibraryRepository,
};
use muzo_desktop_lib::domain::track::TrackRepository;
use muzo_desktop_lib::infrastructure::migrations::MIGRATIONS;
use muzo_desktop_lib::infrastructure::notify_filesystem_watcher::NotifyFilesystemLibraryWatcher;
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

struct StemMetadataReader;

impl AudioMetadataReader for StemMetadataReader {
    fn read(&self, path: &Path) -> Result<AudioMetadata, MetadataError> {
        Ok(AudioMetadata {
            title: path
                .file_stem()
                .map(|stem| stem.to_string_lossy().into_owned())
                .unwrap_or_default(),
            artist: "Artist".into(),
            duration_seconds: 120,
        })
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

#[test]
fn running_watcher_reconciles_added_and_deleted_files() {
    let temp = tempfile::tempdir().unwrap();
    let library_root = temp.path().join("library");
    fs::create_dir(&library_root).unwrap();

    let db_dir = tempfile::tempdir().unwrap();
    let db_path = db_dir.path().join("muzo.sqlite");
    let (library_repository, track_repository) = sqlite_repositories(&db_path);
    let library_repository = Arc::new(library_repository);
    let track_repository = Arc::new(track_repository);
    let library_id = LibraryId("lib-1".into());
    library_repository
        .add(&Library::new(
            library_id.clone(),
            LibraryName("Music".into()),
            LibraryKind::Filesystem,
            LibraryLocation(library_root.display().to_string()),
        ))
        .unwrap();

    let walker = Arc::new(WalkdirWalker::new());
    let metadata_reader = Arc::new(StemMetadataReader);
    let on_change = {
        let library_repository = Arc::clone(&library_repository);
        let track_repository = Arc::clone(&track_repository);
        let walker = Arc::clone(&walker);
        let metadata_reader = Arc::clone(&metadata_reader);
        Arc::new(move |library_id: LibraryId| {
            let _ = scan_library(
                &library_id,
                &*library_repository,
                &*track_repository,
                &*walker,
                &*metadata_reader,
            );
        })
    };

    let watcher = NotifyFilesystemLibraryWatcher::new(Duration::from_millis(50));
    let report = watch_filesystem_libraries(&*library_repository, &watcher, on_change).unwrap();
    assert_eq!(report.libraries_watched, 1);
    assert!(report.failures.is_empty());

    let track_path = library_root.join("new-track.mp3");
    fs::write(&track_path, b"not-real-audio").unwrap();

    wait_until(|| {
        track_repository
            .list_for_library(&library_id)
            .map(|tracks| tracks.len() == 1)
            .unwrap_or(false)
    });

    fs::remove_file(&track_path).unwrap();

    wait_until(|| {
        track_repository
            .list_for_library(&library_id)
            .map(|tracks| tracks.is_empty())
            .unwrap_or(false)
    });
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

fn wait_until(condition: impl Fn() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if condition() {
            return;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    panic!("condition did not become true before timeout");
}

use crate::application::scan_library::{AudioMetadataReader, FilesystemWalker, ScanError};
use crate::domain::library::{LibraryId, LibraryKind, LibraryRepository};
use crate::domain::track::TrackRepository;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReconciliationReport {
    pub libraries_reconciled: usize,
    pub failures: Vec<LibraryReconciliationFailure>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryReconciliationFailure {
    pub library_id: LibraryId,
    pub message: String,
}

pub fn reconcile_filesystem_libraries(
    libraries: &dyn LibraryRepository,
    tracks: &dyn TrackRepository,
    walker: &dyn FilesystemWalker,
    reader: &dyn AudioMetadataReader,
) -> Result<ReconciliationReport, ScanError> {
    let mut libraries_reconciled = 0;
    let mut failures = Vec::new();

    for library in libraries.list()? {
        if library.kind() != LibraryKind::Filesystem {
            continue;
        }

        match super::scan_library::scan_library(library.id(), libraries, tracks, walker, reader) {
            Ok(_) => libraries_reconciled += 1,
            Err(error) => failures.push(LibraryReconciliationFailure {
                library_id: library.id().clone(),
                message: error.to_string(),
            }),
        }
    }

    Ok(ReconciliationReport {
        libraries_reconciled,
        failures,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};
    use std::sync::{Arc, Mutex};

    use crate::application::scan_library::{
        AudioMetadata, AudioMetadataReader, FilesystemWalker, MetadataError, WalkError, WalkedFile,
    };
    use crate::domain::library::{
        Library, LibraryId, LibraryKind, LibraryLocation, LibraryName, LibraryRepository,
        RepositoryError,
    };
    use crate::domain::track::{Track, TrackRepository};

    use super::reconcile_filesystem_libraries;

    #[derive(Default, Clone)]
    struct InMemoryLibraryRepository {
        libraries: Arc<Mutex<Vec<Library>>>,
    }

    impl InMemoryLibraryRepository {
        fn store(&self, library: Library) {
            self.libraries.lock().unwrap().push(library);
        }
    }

    impl LibraryRepository for InMemoryLibraryRepository {
        fn add(&self, library: &Library) -> Result<(), RepositoryError> {
            self.libraries.lock().unwrap().push(library.clone());
            Ok(())
        }

        fn find_by_id(&self, id: &LibraryId) -> Result<Option<Library>, RepositoryError> {
            Ok(self
                .libraries
                .lock()
                .unwrap()
                .iter()
                .find(|library| library.id() == id)
                .cloned())
        }

        fn list(&self) -> Result<Vec<Library>, RepositoryError> {
            Ok(self.libraries.lock().unwrap().clone())
        }
    }

    #[derive(Default, Clone)]
    struct FakeTrackRepository {
        tracks: Arc<Mutex<Vec<Track>>>,
    }

    impl FakeTrackRepository {
        fn stored(&self) -> Vec<Track> {
            self.tracks.lock().unwrap().clone()
        }
    }

    impl TrackRepository for FakeTrackRepository {
        fn upsert(&self, track: &Track) -> Result<(), RepositoryError> {
            let mut guard = self.tracks.lock().unwrap();
            if let Some(existing) = guard.iter_mut().find(|stored| {
                stored.library_id() == track.library_id() && stored.file_path() == track.file_path()
            }) {
                *existing = track.clone();
            } else {
                guard.push(track.clone());
            }
            Ok(())
        }

        fn list_for_library(&self, library_id: &LibraryId) -> Result<Vec<Track>, RepositoryError> {
            Ok(self
                .tracks
                .lock()
                .unwrap()
                .iter()
                .filter(|track| track.library_id() == library_id)
                .cloned()
                .collect())
        }

        fn delete_by_library_and_path(
            &self,
            library_id: &LibraryId,
            file_path: &crate::domain::track::TrackFilePath,
        ) -> Result<(), RepositoryError> {
            self.tracks
                .lock()
                .unwrap()
                .retain(|track| track.library_id() != library_id || track.file_path() != file_path);
            Ok(())
        }

        fn update_metadata_override(
            &self,
            _library_id: &LibraryId,
            _track_id: &crate::domain::track::TrackId,
            _metadata_override: &crate::domain::track::TrackMetadataOverride,
        ) -> Result<(), RepositoryError> {
            Ok(())
        }
    }

    struct FakeWalker {
        files_by_root: HashMap<PathBuf, Vec<WalkedFile>>,
    }

    impl FakeWalker {
        fn with(files_by_root: Vec<(PathBuf, Vec<WalkedFile>)>) -> Self {
            Self {
                files_by_root: files_by_root.into_iter().collect(),
            }
        }
    }

    impl FilesystemWalker for FakeWalker {
        fn walk_audio_files(&self, root: &Path) -> Result<Vec<WalkedFile>, WalkError> {
            self.files_by_root
                .get(root)
                .cloned()
                .ok_or_else(|| WalkError::NotFound(root.display().to_string()))
        }
    }

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
                .ok_or_else(|| MetadataError::Io(format!("no fake entry for {:?}", path)))
        }
    }

    fn library_at(id: &str, kind: LibraryKind, location: &str) -> Library {
        Library::new(
            LibraryId(id.into()),
            LibraryName("Test".into()),
            kind,
            LibraryLocation(location.into()),
        )
    }

    #[test]
    fn startup_reconciliation_scans_persisted_filesystem_libraries() {
        let libraries = InMemoryLibraryRepository::default();
        libraries.store(library_at("fs-1", LibraryKind::Filesystem, "/music"));
        libraries.store(library_at("dropbox-1", LibraryKind::Dropbox, "/dropbox"));
        let tracks = FakeTrackRepository::default();
        let file_path = PathBuf::from("/music/track.mp3");
        let walker = FakeWalker::with(vec![(
            PathBuf::from("/music"),
            vec![WalkedFile {
                path: file_path.clone(),
                size: 1_000,
                mtime: 100,
            }],
        )]);
        let reader = FakeMetadataReader::with(vec![(
            file_path,
            AudioMetadata {
                title: "Title".into(),
                artist: "Artist".into(),
                album: None,
                track_number: None,
                disc_number: None,
                genre: None,
                year: None,
                duration_seconds: 200,
            },
        )]);

        let report = reconcile_filesystem_libraries(&libraries, &tracks, &walker, &reader).unwrap();

        assert_eq!(report.libraries_reconciled, 1);
        assert!(report.failures.is_empty());
        let stored = tracks.stored();
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].library_id(), &LibraryId("fs-1".into()));
    }

    #[test]
    fn startup_reconciliation_reports_library_failures_and_continues() {
        let libraries = InMemoryLibraryRepository::default();
        libraries.store(library_at("bad", LibraryKind::Filesystem, "/missing"));
        libraries.store(library_at("good", LibraryKind::Filesystem, "/music"));
        let tracks = FakeTrackRepository::default();
        let file_path = PathBuf::from("/music/track.mp3");
        let walker = FakeWalker::with(vec![(
            PathBuf::from("/music"),
            vec![WalkedFile {
                path: file_path.clone(),
                size: 1_000,
                mtime: 100,
            }],
        )]);
        let reader = FakeMetadataReader::with(vec![(
            file_path,
            AudioMetadata {
                title: "Title".into(),
                artist: "Artist".into(),
                album: None,
                track_number: None,
                disc_number: None,
                genre: None,
                year: None,
                duration_seconds: 200,
            },
        )]);

        let report = reconcile_filesystem_libraries(&libraries, &tracks, &walker, &reader).unwrap();

        assert_eq!(report.libraries_reconciled, 1);
        assert_eq!(report.failures.len(), 1);
        assert_eq!(report.failures[0].library_id, LibraryId("bad".into()));
        let stored = tracks.stored();
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].library_id(), &LibraryId("good".into()));
    }
}

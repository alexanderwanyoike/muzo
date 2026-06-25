use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::domain::library::{LibraryId, LibraryKind, LibraryRepository, RepositoryError};
use crate::domain::track::{
    generate_track_id, FileMtime, FileSize, Track, TrackArtist, TrackDuration, TrackFilePath,
    TrackMetadata, TrackMetadataOverride, TrackRepository, TrackTitle,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DropboxScanInput {
    pub library_id: LibraryId,
    pub access_token: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DropboxSourceFile {
    pub path: String,
    pub size: u64,
    pub modified_epoch_seconds: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DropboxScanReport {
    pub tracks_scanned: usize,
}

pub trait DropboxLibraryCatalog {
    fn list_audio_files(
        &self,
        access_token: &str,
        root: &str,
    ) -> Result<Vec<DropboxSourceFile>, DropboxCatalogError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DropboxCatalogError {
    Io(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DropboxScanError {
    LibraryNotFound(LibraryId),
    NotDropboxLibrary(LibraryId),
    Catalog(DropboxCatalogError),
    Repository(RepositoryError),
}

impl From<RepositoryError> for DropboxScanError {
    fn from(error: RepositoryError) -> Self {
        Self::Repository(error)
    }
}

impl From<DropboxCatalogError> for DropboxScanError {
    fn from(error: DropboxCatalogError) -> Self {
        Self::Catalog(error)
    }
}

pub fn scan_dropbox_library(
    input: DropboxScanInput,
    libraries: &dyn LibraryRepository,
    tracks: &dyn TrackRepository,
    catalog: &dyn DropboxLibraryCatalog,
) -> Result<DropboxScanReport, DropboxScanError> {
    let library = libraries
        .find_by_id(&input.library_id)?
        .ok_or_else(|| DropboxScanError::LibraryNotFound(input.library_id.clone()))?;

    if library.kind() != LibraryKind::Dropbox {
        return Err(DropboxScanError::NotDropboxLibrary(input.library_id));
    }

    let files = catalog.list_audio_files(&input.access_token, &library.location().0)?;
    let remote_paths = files
        .iter()
        .map(|file| PathBuf::from(file.path.as_str()))
        .collect::<HashSet<_>>();

    let mut scanned = 0;
    for file in files {
        let track = Track::new_with_metadata(
            generate_track_id(),
            input.library_id.clone(),
            TrackMetadata::new(
                TrackTitle(title_from_remote_path(&file.path)),
                TrackArtist("Unknown Artist".into()),
                None,
                None,
                None,
                None,
                None,
            ),
            TrackMetadataOverride::empty(),
            TrackDuration(0),
            TrackFilePath(PathBuf::from(file.path)),
            FileSize(file.size),
            FileMtime(file.modified_epoch_seconds),
        );
        tracks.upsert(&track)?;
        scanned += 1;
    }

    for track in tracks.list_for_library(&input.library_id)? {
        if !remote_paths.contains(&track.file_path().0) {
            tracks.delete_by_library_and_path(&input.library_id, track.file_path())?;
        }
    }

    Ok(DropboxScanReport {
        tracks_scanned: scanned,
    })
}

fn title_from_remote_path(path: &str) -> String {
    Path::new(path)
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};

    use super::{
        scan_dropbox_library, DropboxCatalogError, DropboxLibraryCatalog, DropboxScanError,
        DropboxScanInput, DropboxSourceFile,
    };
    use crate::domain::library::{
        Library, LibraryId, LibraryKind, LibraryLocation, LibraryName, LibraryRepository,
        RepositoryError,
    };
    use crate::domain::track::{
        FileMtime, FileSize, Track, TrackArtist, TrackDuration, TrackFilePath, TrackId,
        TrackMetadataOverride, TrackRepository, TrackTitle,
    };

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
            self.store(library.clone());
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
        fn store(&self, track: Track) {
            self.tracks.lock().unwrap().push(track);
        }

        fn stored(&self) -> Vec<Track> {
            self.tracks.lock().unwrap().clone()
        }
    }

    impl TrackRepository for FakeTrackRepository {
        fn upsert(&self, track: &Track) -> Result<(), RepositoryError> {
            self.tracks.lock().unwrap().push(track.clone());
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
            file_path: &TrackFilePath,
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
            _metadata_override: &TrackMetadataOverride,
        ) -> Result<(), RepositoryError> {
            Ok(())
        }
    }

    #[derive(Default)]
    struct FakeDropboxCatalog {
        calls: Mutex<Vec<(String, String)>>,
        files: Mutex<Vec<DropboxSourceFile>>,
    }

    impl FakeDropboxCatalog {
        fn with(files: Vec<DropboxSourceFile>) -> Self {
            Self {
                calls: Mutex::new(vec![]),
                files: Mutex::new(files),
            }
        }
    }

    impl DropboxLibraryCatalog for FakeDropboxCatalog {
        fn list_audio_files(
            &self,
            access_token: &str,
            root: &str,
        ) -> Result<Vec<DropboxSourceFile>, DropboxCatalogError> {
            self.calls
                .lock()
                .unwrap()
                .push((access_token.into(), root.into()));
            Ok(self.files.lock().unwrap().clone())
        }
    }

    #[test]
    fn scans_a_dropbox_library_and_persists_remote_audio_files() {
        let libraries = InMemoryLibraryRepository::default();
        libraries.store(Library::new(
            LibraryId("lib-1".into()),
            LibraryName("Dropbox Music".into()),
            LibraryKind::Dropbox,
            LibraryLocation("/Music".into()),
        ));
        let tracks = FakeTrackRepository::default();
        let catalog = FakeDropboxCatalog::with(vec![DropboxSourceFile {
            path: "/Music/Album/Song.mp3".into(),
            size: 12_345,
            modified_epoch_seconds: 1_700_000_000,
        }]);

        let report = scan_dropbox_library(
            DropboxScanInput {
                library_id: LibraryId("lib-1".into()),
                access_token: "access-123".into(),
            },
            &libraries,
            &tracks,
            &catalog,
        )
        .expect("Dropbox scan should succeed");

        assert_eq!(report.tracks_scanned, 1);
        assert_eq!(
            catalog.calls.lock().unwrap().as_slice(),
            &[("access-123".into(), "/Music".into())]
        );
        let stored = tracks.stored();
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].library_id(), &LibraryId("lib-1".into()));
        assert_eq!(stored[0].title().0, "Song");
        assert_eq!(stored[0].artist().0, "Unknown Artist");
        assert_eq!(
            stored[0].file_path().0,
            PathBuf::from("/Music/Album/Song.mp3")
        );
        assert_eq!(stored[0].file_size().0, 12_345);
        assert_eq!(stored[0].file_mtime().0, 1_700_000_000);
    }

    #[test]
    fn rejects_a_non_dropbox_library() {
        let libraries = InMemoryLibraryRepository::default();
        libraries.store(Library::new(
            LibraryId("lib-1".into()),
            LibraryName("Local Music".into()),
            LibraryKind::Filesystem,
            LibraryLocation("/Music".into()),
        ));
        let tracks = FakeTrackRepository::default();
        let catalog = FakeDropboxCatalog::default();

        let result = scan_dropbox_library(
            DropboxScanInput {
                library_id: LibraryId("lib-1".into()),
                access_token: "access-123".into(),
            },
            &libraries,
            &tracks,
            &catalog,
        );

        assert_eq!(
            result,
            Err(DropboxScanError::NotDropboxLibrary(LibraryId(
                "lib-1".into()
            )))
        );
        assert!(catalog.calls.lock().unwrap().is_empty());
        assert!(tracks.stored().is_empty());
    }

    #[test]
    fn rejects_a_missing_dropbox_library() {
        let libraries = InMemoryLibraryRepository::default();
        let tracks = FakeTrackRepository::default();
        let catalog = FakeDropboxCatalog::default();

        let result = scan_dropbox_library(
            DropboxScanInput {
                library_id: LibraryId("missing".into()),
                access_token: "access-123".into(),
            },
            &libraries,
            &tracks,
            &catalog,
        );

        assert_eq!(
            result,
            Err(DropboxScanError::LibraryNotFound(LibraryId(
                "missing".into()
            )))
        );
        assert!(catalog.calls.lock().unwrap().is_empty());
        assert!(tracks.stored().is_empty());
    }

    #[test]
    fn removes_tracks_missing_from_the_latest_dropbox_listing() {
        let libraries = InMemoryLibraryRepository::default();
        libraries.store(Library::new(
            LibraryId("lib-1".into()),
            LibraryName("Dropbox Music".into()),
            LibraryKind::Dropbox,
            LibraryLocation("/Music".into()),
        ));
        let tracks = FakeTrackRepository::default();
        tracks.store(Track::new(
            TrackId("old-track".into()),
            LibraryId("lib-1".into()),
            TrackTitle("Old Song".into()),
            TrackArtist("Unknown Artist".into()),
            TrackDuration(0),
            TrackFilePath(PathBuf::from("/Music/Old Song.mp3")),
            FileSize(1),
            FileMtime(1),
        ));
        let catalog = FakeDropboxCatalog::with(vec![DropboxSourceFile {
            path: "/Music/New Song.mp3".into(),
            size: 2,
            modified_epoch_seconds: 2,
        }]);

        scan_dropbox_library(
            DropboxScanInput {
                library_id: LibraryId("lib-1".into()),
                access_token: "access-123".into(),
            },
            &libraries,
            &tracks,
            &catalog,
        )
        .expect("Dropbox scan should succeed");

        let stored = tracks.stored();
        assert_eq!(stored.len(), 1);
        assert_eq!(
            stored[0].file_path().0,
            PathBuf::from("/Music/New Song.mp3")
        );
    }
}

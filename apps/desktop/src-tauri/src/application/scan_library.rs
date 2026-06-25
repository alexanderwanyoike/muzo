use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::domain::library::{LibraryId, LibraryRepository, RepositoryError};
use crate::domain::track::{Track, TrackRepository};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalkedFile {
    pub path: PathBuf,
    pub size: u64,
    pub mtime: u64,
}

#[derive(Debug, Clone)]
pub struct AudioMetadata {
    pub title: String,
    pub artist: String,
    pub duration_seconds: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WalkError {
    Io(String),
    NotFound(String),
}

impl std::fmt::Display for WalkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WalkError::Io(msg) => write!(f, "walk io error: {}", msg),
            WalkError::NotFound(p) => write!(f, "walk root not found: {}", p),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetadataError {
    Io(String),
    Parse(String),
}

impl std::fmt::Display for MetadataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetadataError::Io(msg) => write!(f, "metadata io error: {}", msg),
            MetadataError::Parse(msg) => write!(f, "metadata parse error: {}", msg),
        }
    }
}

pub trait FilesystemWalker {
    fn walk_audio_files(&self, root: &Path) -> Result<Vec<WalkedFile>, WalkError>;
}

pub trait AudioMetadataReader {
    fn read(&self, path: &Path) -> Result<AudioMetadata, MetadataError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanReport {
    pub tracks_scanned: usize,
}

#[derive(Debug, Clone)]
pub enum ScanError {
    LibraryNotFound(LibraryId),
    Walk(WalkError),
    Metadata(MetadataError),
    Repository(RepositoryError),
}

impl std::fmt::Display for ScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScanError::LibraryNotFound(id) => write!(f, "library not found: {}", id),
            ScanError::Walk(e) => write!(f, "walk failed: {}", e),
            ScanError::Metadata(e) => write!(f, "metadata read failed: {}", e),
            ScanError::Repository(e) => write!(f, "repository error: {}", e),
        }
    }
}

impl std::error::Error for ScanError {}

impl From<WalkError> for ScanError {
    fn from(e: WalkError) -> Self {
        ScanError::Walk(e)
    }
}

impl From<MetadataError> for ScanError {
    fn from(e: MetadataError) -> Self {
        ScanError::Metadata(e)
    }
}

impl From<RepositoryError> for ScanError {
    fn from(e: RepositoryError) -> Self {
        ScanError::Repository(e)
    }
}

pub fn scan_library(
    library_id: &LibraryId,
    libraries: &dyn LibraryRepository,
    tracks: &dyn TrackRepository,
    walker: &dyn FilesystemWalker,
    reader: &dyn AudioMetadataReader,
) -> Result<ScanReport, ScanError> {
    let library = libraries
        .find_by_id(library_id)?
        .ok_or_else(|| ScanError::LibraryNotFound(library_id.clone()))?;

    let library_root = Path::new(library.location().0.as_str());
    let walked = walker.walk_audio_files(library_root)?;

    let walked_paths = walked
        .iter()
        .map(|file| file.path.clone())
        .collect::<HashSet<_>>();

    let mut scanned = 0;
    for file in walked {
        let metadata = reader.read(&file.path)?;
        let track = Track::new(
            crate::domain::track::generate_track_id(),
            library_id.clone(),
            crate::domain::track::TrackTitle(metadata.title),
            crate::domain::track::TrackArtist(metadata.artist),
            crate::domain::track::TrackDuration(metadata.duration_seconds),
            crate::domain::track::TrackFilePath(file.path),
            crate::domain::track::FileSize(file.size),
            crate::domain::track::FileMtime(file.mtime),
        );
        tracks.upsert(&track)?;
        scanned += 1;
    }

    for track in tracks.list_for_library(library_id)? {
        if !walked_paths.contains(&track.file_path().0) {
            tracks.delete_by_library_and_path(library_id, track.file_path())?;
        }
    }

    Ok(ScanReport {
        tracks_scanned: scanned,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};
    use std::sync::{Arc, Mutex};

    use crate::domain::library::{
        Library, LibraryId, LibraryKind, LibraryLocation, LibraryName, LibraryRepository,
        RepositoryError,
    };
    use crate::domain::track::{Track, TrackRepository};

    use super::{
        scan_library, AudioMetadata, AudioMetadataReader, FilesystemWalker, MetadataError,
        WalkError, WalkedFile,
    };

    // -- Test fakes ---------------------------------------------------------

    #[derive(Default, Clone)]
    struct InMemoryLibraryRepository {
        libraries: Arc<Mutex<Vec<Library>>>,
    }

    impl InMemoryLibraryRepository {
        fn new() -> Self {
            Self::default()
        }

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
                .find(|l| l.id() == id)
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
        fn new() -> Self {
            Self::default()
        }

        fn store(&self, track: Track) {
            self.tracks.lock().unwrap().push(track);
        }

        fn stored(&self) -> Vec<Track> {
            self.tracks.lock().unwrap().clone()
        }
    }

    impl TrackRepository for FakeTrackRepository {
        fn upsert(&self, track: &Track) -> Result<(), RepositoryError> {
            let mut guard = self.tracks.lock().unwrap();
            if let Some(existing) = guard.iter_mut().find(|t| {
                t.library_id() == track.library_id() && t.file_path() == track.file_path()
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
                .filter(|t| t.library_id() == library_id)
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
                .retain(|t| t.library_id() != library_id || t.file_path() != file_path);
            Ok(())
        }
    }

    struct FakeWalker {
        files: Vec<WalkedFile>,
    }

    impl FakeWalker {
        fn with(files: Vec<WalkedFile>) -> Self {
            Self { files }
        }
    }

    impl FilesystemWalker for FakeWalker {
        fn walk_audio_files(&self, _root: &Path) -> Result<Vec<WalkedFile>, WalkError> {
            Ok(self.files.clone())
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

    // -- Tests --------------------------------------------------------------

    fn library_at(id: &str, location: &str) -> Library {
        Library::new(
            LibraryId(id.into()),
            LibraryName("Test".into()),
            LibraryKind::Filesystem,
            LibraryLocation(location.into()),
        )
    }

    #[test]
    fn scan_library_walks_the_library_root_and_persists_one_track_per_file() {
        let libraries = InMemoryLibraryRepository::new();
        let library = library_at("lib-1", "/music");
        libraries.store(library.clone());
        let tracks = FakeTrackRepository::new();
        let walker = FakeWalker::with(vec![WalkedFile {
            path: PathBuf::from("/music/track.mp3"),
            size: 1_000,
            mtime: 100,
        }]);
        let reader = FakeMetadataReader::with(vec![(
            PathBuf::from("/music/track.mp3"),
            AudioMetadata {
                title: "Title".into(),
                artist: "Artist".into(),
                duration_seconds: 200,
            },
        )]);

        let report = scan_library(
            &LibraryId("lib-1".into()),
            &libraries,
            &tracks,
            &walker,
            &reader,
        )
        .expect("scan should succeed");

        assert_eq!(report.tracks_scanned, 1);
        let stored = tracks.stored();
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].library_id(), &LibraryId("lib-1".into()));
        assert_eq!(
            stored[0].title(),
            &crate::domain::track::TrackTitle("Title".into())
        );
        assert_eq!(
            stored[0].artist(),
            &crate::domain::track::TrackArtist("Artist".into())
        );
        assert_eq!(
            stored[0].duration(),
            crate::domain::track::TrackDuration(200)
        );
    }

    #[test]
    fn scan_library_with_no_files_reports_zero_tracks_and_persists_nothing() {
        let libraries = InMemoryLibraryRepository::new();
        libraries.store(library_at("lib-1", "/empty"));
        let tracks = FakeTrackRepository::new();
        let walker = FakeWalker::with(vec![]);
        let reader = FakeMetadataReader::with(vec![]);

        let report = scan_library(
            &LibraryId("lib-1".into()),
            &libraries,
            &tracks,
            &walker,
            &reader,
        )
        .expect("scan should succeed");

        assert_eq!(report.tracks_scanned, 0);
        assert!(tracks.stored().is_empty());
    }

    #[test]
    fn scanning_the_same_library_twice_does_not_duplicate_tracks() {
        let libraries = InMemoryLibraryRepository::new();
        libraries.store(library_at("lib-1", "/music"));
        let tracks = FakeTrackRepository::new();
        let file_path = PathBuf::from("/music/track.mp3");
        let walker = FakeWalker::with(vec![WalkedFile {
            path: file_path.clone(),
            size: 1_000,
            mtime: 100,
        }]);
        let reader = FakeMetadataReader::with(vec![(
            file_path.clone(),
            AudioMetadata {
                title: "Title".into(),
                artist: "Artist".into(),
                duration_seconds: 200,
            },
        )]);

        let first = scan_library(
            &LibraryId("lib-1".into()),
            &libraries,
            &tracks,
            &walker,
            &reader,
        )
        .expect("first scan should succeed");
        let second = scan_library(
            &LibraryId("lib-1".into()),
            &libraries,
            &tracks,
            &walker,
            &reader,
        )
        .expect("second scan should succeed");

        assert_eq!(first.tracks_scanned, 1);
        assert_eq!(second.tracks_scanned, 1);
        assert_eq!(
            tracks.stored().len(),
            1,
            "rescan must not duplicate the row"
        );
    }

    #[test]
    fn scan_library_removes_tracks_whose_files_are_no_longer_present() {
        let libraries = InMemoryLibraryRepository::new();
        libraries.store(library_at("lib-1", "/music"));
        let tracks = FakeTrackRepository::new();
        tracks.store(Track::new(
            crate::domain::track::TrackId("trk-missing".into()),
            LibraryId("lib-1".into()),
            crate::domain::track::TrackTitle("Missing".into()),
            crate::domain::track::TrackArtist("Artist".into()),
            crate::domain::track::TrackDuration(200),
            crate::domain::track::TrackFilePath(PathBuf::from("/music/missing.mp3")),
            crate::domain::track::FileSize(1_000),
            crate::domain::track::FileMtime(100),
        ));
        let walker = FakeWalker::with(vec![]);
        let reader = FakeMetadataReader::with(vec![]);

        let report = scan_library(
            &LibraryId("lib-1".into()),
            &libraries,
            &tracks,
            &walker,
            &reader,
        )
        .expect("scan should succeed");

        assert_eq!(report.tracks_scanned, 0);
        assert!(
            tracks.stored().is_empty(),
            "tracks missing from the filesystem should be removed"
        );
    }

    #[test]
    fn scan_library_returns_library_not_found_when_the_id_does_not_exist() {
        let libraries = InMemoryLibraryRepository::new();
        let tracks = FakeTrackRepository::new();
        let walker = FakeWalker::with(vec![]);
        let reader = FakeMetadataReader::with(vec![]);

        let result = scan_library(
            &LibraryId("does-not-exist".into()),
            &libraries,
            &tracks,
            &walker,
            &reader,
        );

        assert!(matches!(result, Err(ScanError::LibraryNotFound(_))));
        assert!(tracks.stored().is_empty());
    }

    use super::ScanError;

    #[test]
    fn scan_library_propagates_walk_errors() {
        let libraries = InMemoryLibraryRepository::new();
        libraries.store(library_at("lib-1", "/missing"));
        let tracks = FakeTrackRepository::new();

        struct FailingWalker;
        impl FilesystemWalker for FailingWalker {
            fn walk_audio_files(&self, _root: &Path) -> Result<Vec<WalkedFile>, WalkError> {
                Err(WalkError::NotFound("/missing".into()))
            }
        }

        let result = scan_library(
            &LibraryId("lib-1".into()),
            &libraries,
            &tracks,
            &FailingWalker,
            &FakeMetadataReader::with(vec![]),
        );

        assert!(matches!(result, Err(ScanError::Walk(_))));
    }
}

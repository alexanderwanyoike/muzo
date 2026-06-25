use std::path::PathBuf;
use std::sync::Arc;

use crate::domain::library::{Library, LibraryId, LibraryKind, LibraryRepository};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WatchError {
    Io(String),
}

impl std::fmt::Display for WatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WatchError::Io(message) => write!(f, "watch io error: {}", message),
        }
    }
}

impl std::error::Error for WatchError {}

pub type FilesystemLibraryChangeHandler = Arc<dyn Fn(LibraryId) + Send + Sync>;

pub trait FilesystemLibraryWatcher {
    fn watch(
        &self,
        library_id: LibraryId,
        root: PathBuf,
        on_change: FilesystemLibraryChangeHandler,
    ) -> Result<(), WatchError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchFilesystemLibrariesReport {
    pub libraries_watched: usize,
    pub failures: Vec<WatchFilesystemLibraryFailure>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchFilesystemLibraryFailure {
    pub library_id: LibraryId,
    pub message: String,
}

pub fn watch_filesystem_libraries(
    libraries: &dyn LibraryRepository,
    watcher: &dyn FilesystemLibraryWatcher,
    on_change: FilesystemLibraryChangeHandler,
) -> Result<WatchFilesystemLibrariesReport, crate::domain::library::RepositoryError> {
    let mut libraries_watched = 0;
    let mut failures = Vec::new();

    for library in libraries.list()? {
        if library.kind() != LibraryKind::Filesystem {
            continue;
        }

        match watcher.watch(
            library.id().clone(),
            PathBuf::from(library.location().0.as_str()),
            on_change.clone(),
        ) {
            Ok(()) => libraries_watched += 1,
            Err(error) => failures.push(WatchFilesystemLibraryFailure {
                library_id: library.id().clone(),
                message: error.to_string(),
            }),
        }
    }

    Ok(WatchFilesystemLibrariesReport {
        libraries_watched,
        failures,
    })
}

pub fn watch_filesystem_library(
    library: &Library,
    watcher: &dyn FilesystemLibraryWatcher,
    on_change: FilesystemLibraryChangeHandler,
) -> Result<bool, WatchError> {
    if library.kind() != LibraryKind::Filesystem {
        return Ok(false);
    }

    watcher.watch(
        library.id().clone(),
        PathBuf::from(library.location().0.as_str()),
        on_change,
    )?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};

    use crate::domain::library::{
        Library, LibraryId, LibraryKind, LibraryLocation, LibraryName, LibraryRepository,
        RepositoryError,
    };

    use super::{
        watch_filesystem_libraries, watch_filesystem_library, FilesystemLibraryChangeHandler,
        FilesystemLibraryWatcher, WatchError,
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

    #[derive(Default)]
    struct FakeWatcher {
        watched: Mutex<Vec<(LibraryId, PathBuf)>>,
    }

    impl FakeWatcher {
        fn watched(&self) -> Vec<(LibraryId, PathBuf)> {
            self.watched.lock().unwrap().clone()
        }
    }

    impl FilesystemLibraryWatcher for FakeWatcher {
        fn watch(
            &self,
            library_id: LibraryId,
            root: PathBuf,
            _on_change: FilesystemLibraryChangeHandler,
        ) -> Result<(), WatchError> {
            self.watched.lock().unwrap().push((library_id, root));
            Ok(())
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
    fn watches_persisted_filesystem_libraries() {
        let libraries = InMemoryLibraryRepository::default();
        libraries.store(library_at("fs-1", LibraryKind::Filesystem, "/music"));
        libraries.store(library_at("dropbox-1", LibraryKind::Dropbox, "/dropbox"));
        let watcher = FakeWatcher::default();

        let report = watch_filesystem_libraries(&libraries, &watcher, Arc::new(|_| {})).unwrap();

        assert_eq!(report.libraries_watched, 1);
        assert!(report.failures.is_empty());
        assert_eq!(
            watcher.watched(),
            vec![(LibraryId("fs-1".into()), PathBuf::from("/music"))]
        );
    }

    #[test]
    fn watches_a_newly_added_filesystem_library() {
        let library = library_at("fs-1", LibraryKind::Filesystem, "/music");
        let watcher = FakeWatcher::default();

        let watched =
            watch_filesystem_library(&library, &watcher, Arc::new(|_| {})).expect("watch succeeds");

        assert!(watched);
        assert_eq!(
            watcher.watched(),
            vec![(LibraryId("fs-1".into()), PathBuf::from("/music"))]
        );
    }
}

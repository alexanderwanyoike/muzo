use crate::domain::library::{Library, LibraryRepository, RepositoryError};

pub fn list_libraries(repository: &dyn LibraryRepository) -> Result<Vec<Library>, RepositoryError> {
    repository.list()
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use crate::domain::library::{
        Library, LibraryId, LibraryKind, LibraryLocation, LibraryName, LibraryRepository,
        RepositoryError,
    };

    use super::list_libraries;

    #[derive(Default, Clone)]
    struct FakeLibraryRepository {
        libraries: Arc<Mutex<Vec<Library>>>,
    }

    impl FakeLibraryRepository {
        fn new() -> Self {
            Self::default()
        }
    }

    impl LibraryRepository for FakeLibraryRepository {
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

    fn make_library(name: &str, location: &str) -> Library {
        Library::new(
            LibraryId(name.into()),
            LibraryName(name.into()),
            LibraryKind::Filesystem,
            LibraryLocation(location.into()),
        )
    }

    #[test]
    fn list_libraries_returns_every_library_in_the_repository() {
        let repo = FakeLibraryRepository::new();
        repo.add(&make_library("First", "/a")).unwrap();
        repo.add(&make_library("Second", "/b")).unwrap();

        let result = list_libraries(&repo).expect("list should succeed");

        assert_eq!(result.len(), 2, "both libraries should be returned");
        assert_eq!(result[0].name(), &LibraryName("First".into()));
        assert_eq!(result[1].name(), &LibraryName("Second".into()));
    }

    #[test]
    fn list_libraries_returns_an_empty_vec_when_the_repository_is_empty() {
        let repo = FakeLibraryRepository::new();

        let result = list_libraries(&repo).expect("list should succeed");

        assert!(
            result.is_empty(),
            "empty repository should yield empty list"
        );
    }

    #[test]
    fn list_libraries_propagates_repository_errors() {
        struct AlwaysFails;
        impl LibraryRepository for AlwaysFails {
            fn add(&self, _library: &Library) -> Result<(), RepositoryError> {
                Ok(())
            }
            fn find_by_id(&self, _id: &LibraryId) -> Result<Option<Library>, RepositoryError> {
                Ok(None)
            }
            fn list(&self) -> Result<Vec<Library>, RepositoryError> {
                Err(RepositoryError::Io("disk read failed".into()))
            }
        }

        let result = list_libraries(&AlwaysFails);

        assert!(result.is_err());
    }
}

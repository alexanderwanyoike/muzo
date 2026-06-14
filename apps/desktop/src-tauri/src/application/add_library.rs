use crate::domain::library::{
    generate_library_id, Library, LibraryKind, LibraryLocation, LibraryName, LibraryRepository,
};

use super::error::AddLibraryError;

pub struct AddLibraryInput {
    pub name: String,
    pub kind: LibraryKind,
    pub location: String,
}

pub fn add_library(
    repository: &dyn LibraryRepository,
    input: AddLibraryInput,
) -> Result<Library, AddLibraryError> {
    if input.name.trim().is_empty() {
        return Err(AddLibraryError::EmptyName);
    }
    if input.location.trim().is_empty() {
        return Err(AddLibraryError::EmptyLocation);
    }

    let library = Library::new(
        generate_library_id(),
        LibraryName(input.name),
        input.kind,
        LibraryLocation(input.location),
    );

    repository.add(&library)?;

    Ok(library)
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use crate::domain::library::{
        Library, LibraryId, LibraryKind, LibraryLocation, LibraryName, LibraryRepository,
        RepositoryError,
    };

    use super::{add_library, AddLibraryError, AddLibraryInput};

    #[derive(Default, Clone)]
    struct FakeLibraryRepository {
        libraries: Arc<Mutex<Vec<Library>>>,
    }

    impl FakeLibraryRepository {
        fn new() -> Self {
            Self::default()
        }

        fn stored(&self) -> Vec<Library> {
            self.libraries.lock().unwrap().clone()
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
    }

    #[test]
    fn add_library_persists_a_new_library_and_returns_it() {
        let repo = FakeLibraryRepository::new();

        let result = add_library(
            &repo,
            AddLibraryInput {
                name: "My Music".to_string(),
                kind: LibraryKind::Filesystem,
                location: "/home/user/Music".to_string(),
            },
        )
        .expect("add should succeed with valid inputs");

        assert_eq!(result.name(), &LibraryName("My Music".into()));
        assert_eq!(result.kind(), LibraryKind::Filesystem);
        assert_eq!(
            result.location(),
            &LibraryLocation("/home/user/Music".into())
        );
        assert!(!result.id().0.is_empty(), "id must not be empty");

        let stored = &repo.stored();
        assert_eq!(stored.len(), 1, "exactly one library should be persisted");
        assert_eq!(stored[0], result);
    }

    #[test]
    fn add_library_rejects_a_whitespace_only_name() {
        let repo = FakeLibraryRepository::new();

        let result = add_library(
            &repo,
            AddLibraryInput {
                name: "   ".to_string(),
                kind: LibraryKind::Filesystem,
                location: "/home/user/Music".to_string(),
            },
        );

        assert!(matches!(result, Err(AddLibraryError::EmptyName)));
        assert!(
            repo.stored().is_empty(),
            "nothing should be persisted on error"
        );
    }

    #[test]
    fn add_library_rejects_an_empty_location() {
        let repo = FakeLibraryRepository::new();

        let result = add_library(
            &repo,
            AddLibraryInput {
                name: "My Music".to_string(),
                kind: LibraryKind::Filesystem,
                location: "".to_string(),
            },
        );

        assert!(matches!(result, Err(AddLibraryError::EmptyLocation)));
        assert!(
            repo.stored().is_empty(),
            "nothing should be persisted on error"
        );
    }

    #[test]
    fn add_library_propagates_repository_errors() {
        struct AlwaysFails;
        impl LibraryRepository for AlwaysFails {
            fn add(&self, _library: &Library) -> Result<(), RepositoryError> {
                Err(RepositoryError::Io("disk full".into()))
            }
            fn find_by_id(&self, _id: &LibraryId) -> Result<Option<Library>, RepositoryError> {
                Ok(None)
            }
        }

        let repo = AlwaysFails;

        let result = add_library(
            &repo,
            AddLibraryInput {
                name: "My Music".to_string(),
                kind: LibraryKind::Filesystem,
                location: "/home/user/Music".to_string(),
            },
        );

        assert!(matches!(
            result,
            Err(AddLibraryError::Repository(RepositoryError::Io(_)))
        ));
    }
}

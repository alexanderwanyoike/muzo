//! Library aggregate root and related value objects.
//!
//! Pure domain types. No framework imports, no serde on the wire, no
//! filesystem, no Tokio. Anything that crosses a layer boundary is mapped
//! at the boundary.

use std::fmt;

/// Identifies a library uniquely within Muzo.
///
/// Newtype over `String` so a `LibraryId` cannot be confused with any other
/// identifier or bare string in the system.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LibraryId(pub String);

impl fmt::Display for LibraryId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Human-readable name for a library. Not unique - display only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryName(pub String);

/// What a library is backed by. Determines which adapter is responsible
/// for scanning and syncing it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LibraryKind {
    /// A folder on the local filesystem, scanned recursively.
    Filesystem,
    /// A folder inside a Dropbox account, synced via the Dropbox API.
    /// The adapter lands in sprint 02; the variant exists now so the domain
    /// does not need to be retrofitted later.
    Dropbox,
}

/// Where a library lives on its source. Opaque to the domain - the
/// infrastructure adapter for the library's [`LibraryKind`] knows how to
/// interpret it (a path for Filesystem, a Dropbox path for Dropbox).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryLocation(pub String);

/// Persistence errors raised by a [`LibraryRepository`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepositoryError {
    Io(String),
    NotFound(LibraryId),
}

impl fmt::Display for RepositoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RepositoryError::Io(msg) => write!(f, "repository io error: {}", msg),
            RepositoryError::NotFound(id) => write!(f, "library not found: {}", id),
        }
    }
}

impl std::error::Error for RepositoryError {}

/// Persists libraries. Implemented by infrastructure adapters (SQLite in
/// sprint 01, possibly more later). Defined in the domain so the application
/// layer can depend on the abstraction without leaking infrastructure.
pub trait LibraryRepository {
    fn add(&self, library: &Library) -> Result<(), RepositoryError>;
    fn find_by_id(&self, id: &LibraryId) -> Result<Option<Library>, RepositoryError>;
}

/// Generate a fresh, stable, unique [`LibraryId`] for a new library.
///
/// ULID format: 26-char lexicographically sortable string. Stable across
/// renames of the human-friendly name; the on-disk identity of a library
/// never changes after creation.
pub fn generate_library_id() -> LibraryId {
    LibraryId(ulid::Ulid::new().to_string())
}

/// The library aggregate root. Holds the invariants; creation and mutation
/// go through constructors and methods on this type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Library {
    id: LibraryId,
    name: LibraryName,
    kind: LibraryKind,
    location: LibraryLocation,
}

impl Library {
    /// Construct a new library from its constituent values.
    pub fn new(
        id: LibraryId,
        name: LibraryName,
        kind: LibraryKind,
        location: LibraryLocation,
    ) -> Self {
        Self {
            id,
            name,
            kind,
            location,
        }
    }

    pub fn id(&self) -> &LibraryId {
        &self.id
    }

    pub fn name(&self) -> &LibraryName {
        &self.name
    }

    pub fn kind(&self) -> LibraryKind {
        self.kind
    }

    pub fn location(&self) -> &LibraryLocation {
        &self.location
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_library_ids_with_the_same_value_are_equal() {
        assert_eq!(LibraryId("lib-1".into()), LibraryId("lib-1".into()));
    }

    #[test]
    fn two_library_ids_with_different_values_are_not_equal() {
        assert_ne!(LibraryId("lib-1".into()), LibraryId("lib-2".into()));
    }

    #[test]
    fn library_id_displays_as_its_inner_value() {
        let id = LibraryId("lib-42".into());
        assert_eq!(id.to_string(), "lib-42");
    }

    #[test]
    fn a_library_exposes_the_values_it_was_constructed_with() {
        let library = Library::new(
            LibraryId("lib-1".into()),
            LibraryName("My Music".into()),
            LibraryKind::Filesystem,
            LibraryLocation("/home/user/Music".into()),
        );

        assert_eq!(library.id(), &LibraryId("lib-1".into()));
        assert_eq!(library.name(), &LibraryName("My Music".into()));
        assert_eq!(library.kind(), LibraryKind::Filesystem);
        assert_eq!(
            library.location(),
            &LibraryLocation("/home/user/Music".into())
        );
    }

    #[test]
    fn filesystem_and_dropbox_are_distinct_library_kinds() {
        assert_ne!(LibraryKind::Filesystem, LibraryKind::Dropbox);
    }
}

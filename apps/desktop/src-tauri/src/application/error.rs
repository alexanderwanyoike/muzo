use std::fmt;

use crate::domain::library::RepositoryError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddLibraryError {
    EmptyName,
    EmptyLocation,
    Repository(RepositoryError),
}

impl fmt::Display for AddLibraryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AddLibraryError::EmptyName => write!(f, "library name must not be empty"),
            AddLibraryError::EmptyLocation => write!(f, "library location must not be empty"),
            AddLibraryError::Repository(err) => write!(f, "repository error: {}", err),
        }
    }
}

impl std::error::Error for AddLibraryError {}

impl From<RepositoryError> for AddLibraryError {
    fn from(err: RepositoryError) -> Self {
        AddLibraryError::Repository(err)
    }
}

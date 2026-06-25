use serde::{Deserialize, Serialize};

use crate::application::add_library::{add_library as run_add_library, AddLibraryInput};
use crate::application::error::AddLibraryError;
use crate::application::watch_filesystem_libraries::watch_filesystem_library;
use crate::domain::library::{Library, LibraryKind};
use crate::{filesystem_library_change_handler, AppState};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddLibraryInputDto {
    pub name: String,
    pub kind: LibraryKindDto,
    pub location: String,
}

#[derive(Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub enum LibraryKindDto {
    Filesystem,
    Dropbox,
}

#[derive(Serialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LibraryDto {
    pub id: String,
    pub name: String,
    pub kind: LibraryKindDto,
    pub location: String,
}

#[derive(Serialize, PartialEq, Eq, Debug)]
#[serde(tag = "kind", content = "message", rename_all = "camelCase")]
pub enum AddLibraryErrorDto {
    EmptyName,
    EmptyLocation,
    Repository(String),
}

impl From<LibraryKind> for LibraryKindDto {
    fn from(kind: LibraryKind) -> Self {
        match kind {
            LibraryKind::Filesystem => LibraryKindDto::Filesystem,
            LibraryKind::Dropbox => LibraryKindDto::Dropbox,
        }
    }
}

impl From<LibraryKindDto> for LibraryKind {
    fn from(dto: LibraryKindDto) -> Self {
        match dto {
            LibraryKindDto::Filesystem => LibraryKind::Filesystem,
            LibraryKindDto::Dropbox => LibraryKind::Dropbox,
        }
    }
}

impl From<Library> for LibraryDto {
    fn from(library: Library) -> Self {
        LibraryDto {
            id: library.id().0.clone(),
            name: library.name().0.clone(),
            kind: library.kind().into(),
            location: library.location().0.clone(),
        }
    }
}

impl From<AddLibraryError> for AddLibraryErrorDto {
    fn from(err: AddLibraryError) -> Self {
        match err {
            AddLibraryError::EmptyName => AddLibraryErrorDto::EmptyName,
            AddLibraryError::EmptyLocation => AddLibraryErrorDto::EmptyLocation,
            AddLibraryError::Repository(e) => AddLibraryErrorDto::Repository(e.to_string()),
        }
    }
}

#[tauri::command]
pub fn add_library(
    state: tauri::State<'_, AppState>,
    input: AddLibraryInputDto,
) -> Result<LibraryDto, AddLibraryErrorDto> {
    let input = AddLibraryInput {
        name: input.name,
        kind: input.kind.into(),
        location: input.location,
    };

    let library =
        run_add_library(&*state.library_repository, input).map_err(AddLibraryErrorDto::from)?;

    if let Err(error) = watch_filesystem_library(
        &library,
        &*state.filesystem_watcher,
        filesystem_library_change_handler(
            state.library_repository.clone(),
            state.track_repository.clone(),
            state.walker.clone(),
            state.metadata_reader.clone(),
        ),
    ) {
        eprintln!(
            "filesystem library watcher failed for {}: {}",
            library.id(),
            error
        );
    }

    Ok(LibraryDto::from(library))
}

#[cfg(test)]
mod tests {
    use crate::domain::library::{Library, LibraryId, LibraryKind, LibraryLocation, LibraryName};

    use super::{AddLibraryError, AddLibraryErrorDto, LibraryDto, LibraryKindDto};

    #[test]
    fn library_dto_round_trips_every_field() {
        let library = Library::new(
            LibraryId("lib-1".into()),
            LibraryName("My Music".into()),
            LibraryKind::Filesystem,
            LibraryLocation("/home/user/Music".into()),
        );

        let dto = LibraryDto::from(library);

        assert_eq!(dto.id, "lib-1");
        assert_eq!(dto.name, "My Music");
        assert_eq!(dto.kind, LibraryKindDto::Filesystem);
        assert_eq!(dto.location, "/home/user/Music");
    }

    #[test]
    fn library_kind_dto_round_trips_both_variants() {
        assert_eq!(
            LibraryKindDto::from(LibraryKind::Filesystem),
            LibraryKindDto::Filesystem
        );
        assert_eq!(
            LibraryKindDto::from(LibraryKind::Dropbox),
            LibraryKindDto::Dropbox
        );

        assert_eq!(
            LibraryKind::from(LibraryKindDto::Filesystem),
            LibraryKind::Filesystem
        );
        assert_eq!(
            LibraryKind::from(LibraryKindDto::Dropbox),
            LibraryKind::Dropbox
        );
    }

    #[test]
    fn add_library_error_dto_preserves_validation_kinds() {
        assert_eq!(
            AddLibraryErrorDto::from(AddLibraryError::EmptyName),
            AddLibraryErrorDto::EmptyName
        );
        assert_eq!(
            AddLibraryErrorDto::from(AddLibraryError::EmptyLocation),
            AddLibraryErrorDto::EmptyLocation
        );
    }
}

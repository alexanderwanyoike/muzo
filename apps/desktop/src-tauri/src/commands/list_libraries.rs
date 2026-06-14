use serde::Serialize;

use crate::application::list_libraries::list_libraries as run_list_libraries;
use crate::commands::add_library::LibraryDto;
use crate::domain::library::RepositoryError;
use crate::AppState;

#[derive(Serialize, PartialEq, Eq, Debug)]
#[serde(tag = "kind", content = "message", rename_all = "camelCase")]
pub enum ListLibrariesErrorDto {
    Repository(String),
}

impl From<RepositoryError> for ListLibrariesErrorDto {
    fn from(err: RepositoryError) -> Self {
        ListLibrariesErrorDto::Repository(err.to_string())
    }
}

#[tauri::command]
pub fn list_libraries(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<LibraryDto>, ListLibrariesErrorDto> {
    run_list_libraries(&*state.library_repository)
        .map(|libraries| libraries.into_iter().map(LibraryDto::from).collect())
        .map_err(ListLibrariesErrorDto::from)
}

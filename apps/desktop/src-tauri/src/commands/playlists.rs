use serde::{Deserialize, Serialize};

use crate::application::playlists::{
    add_track_to_playlist as run_add_track_to_playlist, create_playlist as run_create_playlist,
    list_playlists as run_list_playlists, remove_playlist_entry as run_remove_playlist_entry,
    reorder_playlist_entries as run_reorder_playlist_entries, AddTrackToPlaylistInput,
    CreatePlaylistInput, PlaylistApplicationError, RemovePlaylistEntryInput,
    ReorderPlaylistEntriesInput,
};
use crate::domain::playlist::{Playlist, PlaylistEntryId, PlaylistId};
use crate::domain::track::TrackId;
use crate::AppState;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePlaylistInputDto {
    pub name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddTrackToPlaylistInputDto {
    pub playlist_id: String,
    pub track_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemovePlaylistEntryInputDto {
    pub playlist_id: String,
    pub entry_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReorderPlaylistEntriesInputDto {
    pub playlist_id: String,
    pub ordered_entry_ids: Vec<String>,
}

#[derive(Serialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistDto {
    pub id: String,
    pub name: String,
    pub entries: Vec<PlaylistEntryDto>,
}

#[derive(Serialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistEntryDto {
    pub id: String,
    pub track_id: String,
    pub position: u32,
}

#[derive(Serialize, PartialEq, Eq, Debug)]
#[serde(tag = "kind", content = "message", rename_all = "camelCase")]
pub enum PlaylistErrorDto {
    EmptyName,
    PlaylistNotFound,
    InvalidEntryOrder,
    Repository(String),
}

impl From<Playlist> for PlaylistDto {
    fn from(playlist: Playlist) -> Self {
        PlaylistDto {
            id: playlist.id().0.clone(),
            name: playlist.name().as_str().to_string(),
            entries: playlist
                .entries()
                .iter()
                .map(|entry| PlaylistEntryDto {
                    id: entry.id().0.clone(),
                    track_id: entry.track_id().0.clone(),
                    position: entry.position().0,
                })
                .collect(),
        }
    }
}

impl From<PlaylistApplicationError> for PlaylistErrorDto {
    fn from(error: PlaylistApplicationError) -> Self {
        match error {
            PlaylistApplicationError::EmptyName => PlaylistErrorDto::EmptyName,
            PlaylistApplicationError::PlaylistNotFound(_) => PlaylistErrorDto::PlaylistNotFound,
            PlaylistApplicationError::InvalidEntryOrder => PlaylistErrorDto::InvalidEntryOrder,
            PlaylistApplicationError::Repository(error) => {
                PlaylistErrorDto::Repository(error.to_string())
            }
        }
    }
}

#[tauri::command]
pub fn create_playlist(
    state: tauri::State<'_, AppState>,
    input: CreatePlaylistInputDto,
) -> Result<PlaylistDto, PlaylistErrorDto> {
    run_create_playlist(
        &*state.playlist_repository,
        CreatePlaylistInput { name: input.name },
    )
    .map(PlaylistDto::from)
    .map_err(PlaylistErrorDto::from)
}

#[tauri::command]
pub fn list_playlists(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<PlaylistDto>, PlaylistErrorDto> {
    run_list_playlists(&*state.playlist_repository)
        .map(|playlists| playlists.into_iter().map(PlaylistDto::from).collect())
        .map_err(PlaylistErrorDto::from)
}

#[tauri::command]
pub fn add_track_to_playlist(
    state: tauri::State<'_, AppState>,
    input: AddTrackToPlaylistInputDto,
) -> Result<PlaylistDto, PlaylistErrorDto> {
    run_add_track_to_playlist(
        &*state.playlist_repository,
        AddTrackToPlaylistInput {
            playlist_id: PlaylistId(input.playlist_id),
            track_id: TrackId(input.track_id),
        },
    )
    .map(PlaylistDto::from)
    .map_err(PlaylistErrorDto::from)
}

#[tauri::command]
pub fn remove_playlist_entry(
    state: tauri::State<'_, AppState>,
    input: RemovePlaylistEntryInputDto,
) -> Result<PlaylistDto, PlaylistErrorDto> {
    run_remove_playlist_entry(
        &*state.playlist_repository,
        RemovePlaylistEntryInput {
            playlist_id: PlaylistId(input.playlist_id),
            entry_id: PlaylistEntryId(input.entry_id),
        },
    )
    .map(PlaylistDto::from)
    .map_err(PlaylistErrorDto::from)
}

#[tauri::command]
pub fn reorder_playlist_entries(
    state: tauri::State<'_, AppState>,
    input: ReorderPlaylistEntriesInputDto,
) -> Result<PlaylistDto, PlaylistErrorDto> {
    run_reorder_playlist_entries(
        &*state.playlist_repository,
        ReorderPlaylistEntriesInput {
            playlist_id: PlaylistId(input.playlist_id),
            ordered_entry_ids: input
                .ordered_entry_ids
                .into_iter()
                .map(PlaylistEntryId)
                .collect(),
        },
    )
    .map(PlaylistDto::from)
    .map_err(PlaylistErrorDto::from)
}

#[cfg(test)]
mod tests {
    use crate::application::playlists::PlaylistApplicationError;
    use crate::domain::library::RepositoryError;
    use crate::domain::playlist::{
        Playlist, PlaylistEntryId, PlaylistId, PlaylistName, PlaylistPosition,
    };
    use crate::domain::track::TrackId;

    use super::{PlaylistDto, PlaylistErrorDto};

    #[test]
    fn playlist_dto_preserves_name_and_ordered_entries() {
        let mut playlist = Playlist::new(
            PlaylistId("playlist-1".into()),
            PlaylistName::new("Road Trip".into()).unwrap(),
        );
        playlist.add_track(PlaylistEntryId("entry-1".into()), TrackId("track-1".into()));
        playlist.add_track(PlaylistEntryId("entry-2".into()), TrackId("track-2".into()));

        let dto = PlaylistDto::from(playlist);

        assert_eq!(dto.id, "playlist-1");
        assert_eq!(dto.name, "Road Trip");
        assert_eq!(dto.entries.len(), 2);
        assert_eq!(dto.entries[0].id, "entry-1");
        assert_eq!(dto.entries[0].track_id, "track-1");
        assert_eq!(dto.entries[0].position, PlaylistPosition(0).0);
        assert_eq!(dto.entries[1].id, "entry-2");
        assert_eq!(dto.entries[1].track_id, "track-2");
        assert_eq!(dto.entries[1].position, PlaylistPosition(1).0);
    }

    #[test]
    fn playlist_error_dto_preserves_validation_and_lookup_kinds() {
        assert_eq!(
            PlaylistErrorDto::from(PlaylistApplicationError::EmptyName),
            PlaylistErrorDto::EmptyName
        );
        assert_eq!(
            PlaylistErrorDto::from(PlaylistApplicationError::PlaylistNotFound(PlaylistId(
                "playlist-1".into()
            ))),
            PlaylistErrorDto::PlaylistNotFound
        );
        assert_eq!(
            PlaylistErrorDto::from(PlaylistApplicationError::InvalidEntryOrder),
            PlaylistErrorDto::InvalidEntryOrder
        );
        assert_eq!(
            PlaylistErrorDto::from(PlaylistApplicationError::Repository(RepositoryError::Io(
                "database locked".into()
            ))),
            PlaylistErrorDto::Repository("repository io error: database locked".into())
        );
    }
}

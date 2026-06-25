use serde::Deserialize;

use crate::application::edit_track_metadata::{
    clear_track_metadata_override as run_clear_track_metadata_override,
    edit_track_metadata as run_edit_track_metadata, EditTrackMetadataInput,
};
use crate::domain::library::{LibraryId, RepositoryError};
use crate::domain::track::{
    DiscNumber, TrackAlbum, TrackArtist, TrackGenre, TrackId, TrackMetadataOverride, TrackNumber,
    TrackTitle, TrackYear,
};
use crate::AppState;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditTrackMetadataInputDto {
    pub library_id: String,
    pub track_id: String,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub track_number: Option<u32>,
    pub disc_number: Option<u32>,
    pub genre: Option<String>,
    pub year: Option<i32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClearTrackMetadataOverrideInputDto {
    pub library_id: String,
    pub track_id: String,
}

#[derive(serde::Serialize, PartialEq, Eq, Debug)]
#[serde(tag = "kind", content = "message", rename_all = "camelCase")]
pub enum TrackMetadataErrorDto {
    Repository(String),
}

impl From<RepositoryError> for TrackMetadataErrorDto {
    fn from(error: RepositoryError) -> Self {
        TrackMetadataErrorDto::Repository(error.to_string())
    }
}

#[tauri::command]
pub fn edit_track_metadata(
    state: tauri::State<'_, AppState>,
    input: EditTrackMetadataInputDto,
) -> Result<(), TrackMetadataErrorDto> {
    let library_id = LibraryId(input.library_id.clone());
    let track_id = TrackId(input.track_id.clone());
    let metadata_override = metadata_override_from_dto(input);

    run_edit_track_metadata(
        &*state.track_repository,
        EditTrackMetadataInput {
            library_id,
            track_id,
            metadata_override,
        },
    )
    .map_err(TrackMetadataErrorDto::from)
}

#[tauri::command]
pub fn clear_track_metadata_override(
    state: tauri::State<'_, AppState>,
    input: ClearTrackMetadataOverrideInputDto,
) -> Result<(), TrackMetadataErrorDto> {
    run_clear_track_metadata_override(
        &*state.track_repository,
        &LibraryId(input.library_id),
        &TrackId(input.track_id),
    )
    .map_err(TrackMetadataErrorDto::from)
}

fn metadata_override_from_dto(input: EditTrackMetadataInputDto) -> TrackMetadataOverride {
    TrackMetadataOverride::new(
        input.title.map(TrackTitle),
        input.artist.map(TrackArtist),
        input.album.map(TrackAlbum),
        input.track_number.map(TrackNumber),
        input.disc_number.map(DiscNumber),
        input.genre.map(TrackGenre),
        input.year.map(TrackYear),
    )
}

#[cfg(test)]
mod tests {
    use super::{metadata_override_from_dto, EditTrackMetadataInputDto};

    use crate::domain::track::{TrackAlbum, TrackMetadataOverride, TrackTitle};

    #[test]
    fn edit_track_metadata_dto_maps_to_metadata_override() {
        let metadata_override = metadata_override_from_dto(EditTrackMetadataInputDto {
            library_id: "lib-1".into(),
            track_id: "trk-1".into(),
            title: Some("Edited".into()),
            artist: None,
            album: Some("Edited Album".into()),
            track_number: None,
            disc_number: None,
            genre: None,
            year: None,
        });

        assert_eq!(
            metadata_override,
            TrackMetadataOverride::new(
                Some(TrackTitle("Edited".into())),
                None,
                Some(TrackAlbum("Edited Album".into())),
                None,
                None,
                None,
                None,
            )
        );
    }
}

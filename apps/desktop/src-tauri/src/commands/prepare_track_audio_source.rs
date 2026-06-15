use serde::{Deserialize, Serialize};

use crate::application::resolve_track_audio_source::{
    resolve_track_audio_source, ResolveTrackAudioSourceError,
};
use crate::domain::library::LibraryId;
use crate::domain::track::TrackId;
use crate::AppState;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrepareTrackAudioSourceInputDto {
    pub library_id: String,
    pub track_id: String,
}

#[derive(Serialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PreparedTrackAudioSourceDto {
    pub mime_type: String,
    pub url: String,
}

#[derive(Serialize, PartialEq, Eq, Debug)]
#[serde(tag = "kind", content = "message", rename_all = "camelCase")]
pub enum PrepareTrackAudioSourceErrorDto {
    TrackNotFound(String),
    UnsupportedFileType(String),
    Repository(String),
}

impl From<ResolveTrackAudioSourceError> for PrepareTrackAudioSourceErrorDto {
    fn from(err: ResolveTrackAudioSourceError) -> Self {
        match err {
            ResolveTrackAudioSourceError::TrackNotFound(id) => {
                PrepareTrackAudioSourceErrorDto::TrackNotFound(id.0)
            }
            ResolveTrackAudioSourceError::UnsupportedFileType(path) => {
                PrepareTrackAudioSourceErrorDto::UnsupportedFileType(path)
            }
            ResolveTrackAudioSourceError::Repository(err) => {
                PrepareTrackAudioSourceErrorDto::Repository(err.to_string())
            }
        }
    }
}

#[tauri::command]
pub fn prepare_track_audio_source(
    state: tauri::State<'_, AppState>,
    input: PrepareTrackAudioSourceInputDto,
) -> Result<PreparedTrackAudioSourceDto, PrepareTrackAudioSourceErrorDto> {
    let source = resolve_track_audio_source(
        &LibraryId(input.library_id),
        &TrackId(input.track_id),
        &*state.track_repository,
    )?;
    let url = state
        .audio_stream_server
        .register(&source.file_path, &source.mime_type);

    Ok(PreparedTrackAudioSourceDto {
        mime_type: source.mime_type,
        url,
    })
}

#[cfg(test)]
mod tests {
    use crate::application::resolve_track_audio_source::ResolveTrackAudioSourceError;
    use crate::domain::track::TrackId;

    use super::PrepareTrackAudioSourceErrorDto;

    #[test]
    fn prepare_track_audio_source_error_dto_maps_track_not_found() {
        let dto = PrepareTrackAudioSourceErrorDto::from(
            ResolveTrackAudioSourceError::TrackNotFound(TrackId("trk-1".into())),
        );

        assert_eq!(
            dto,
            PrepareTrackAudioSourceErrorDto::TrackNotFound("trk-1".into())
        );
    }
}

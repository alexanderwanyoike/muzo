use serde::{Deserialize, Serialize};

use crate::application::load_track_audio_source::{
    load_track_audio_source as run_load_track_audio_source, LoadTrackAudioSourceError,
    TrackAudioSource,
};
use crate::domain::library::LibraryId;
use crate::domain::track::TrackId;
use crate::AppState;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadTrackAudioSourceInputDto {
    pub library_id: String,
    pub track_id: String,
}

#[derive(Serialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TrackAudioSourceDto {
    pub mime_type: String,
    pub bytes: Vec<u8>,
}

impl From<TrackAudioSource> for TrackAudioSourceDto {
    fn from(source: TrackAudioSource) -> Self {
        TrackAudioSourceDto {
            mime_type: source.mime_type,
            bytes: source.bytes,
        }
    }
}

#[derive(Serialize, PartialEq, Eq, Debug)]
#[serde(tag = "kind", content = "message", rename_all = "camelCase")]
pub enum LoadTrackAudioSourceErrorDto {
    TrackNotFound(String),
    UnsupportedFileType(String),
    Repository(String),
    Read(String),
}

impl From<LoadTrackAudioSourceError> for LoadTrackAudioSourceErrorDto {
    fn from(err: LoadTrackAudioSourceError) -> Self {
        match err {
            LoadTrackAudioSourceError::TrackNotFound(id) => {
                LoadTrackAudioSourceErrorDto::TrackNotFound(id.0)
            }
            LoadTrackAudioSourceError::UnsupportedFileType(path) => {
                LoadTrackAudioSourceErrorDto::UnsupportedFileType(path)
            }
            LoadTrackAudioSourceError::Repository(err) => {
                LoadTrackAudioSourceErrorDto::Repository(err.to_string())
            }
            LoadTrackAudioSourceError::Read(err) => {
                LoadTrackAudioSourceErrorDto::Read(err.to_string())
            }
        }
    }
}

#[tauri::command]
pub fn load_track_audio_source(
    state: tauri::State<'_, AppState>,
    input: LoadTrackAudioSourceInputDto,
) -> Result<TrackAudioSourceDto, LoadTrackAudioSourceErrorDto> {
    let source = run_load_track_audio_source(
        &LibraryId(input.library_id),
        &TrackId(input.track_id),
        &*state.track_repository,
        &*state.audio_reader,
    )?;

    Ok(source.into())
}

#[cfg(test)]
mod tests {
    use crate::application::load_track_audio_source::{
        LoadTrackAudioSourceError, TrackAudioReadError, TrackAudioSource,
    };

    use super::{LoadTrackAudioSourceErrorDto, TrackAudioSourceDto};

    #[test]
    fn track_audio_source_dto_preserves_mime_type_and_bytes() {
        let dto = TrackAudioSourceDto::from(TrackAudioSource {
            mime_type: "audio/mpeg".into(),
            bytes: vec![1, 2, 3],
        });

        assert_eq!(dto.mime_type, "audio/mpeg");
        assert_eq!(dto.bytes, vec![1, 2, 3]);
    }

    #[test]
    fn load_track_audio_source_error_dto_maps_read_errors() {
        let dto = LoadTrackAudioSourceErrorDto::from(LoadTrackAudioSourceError::Read(
            TrackAudioReadError::Io("permission denied".into()),
        ));

        assert_eq!(
            dto,
            LoadTrackAudioSourceErrorDto::Read("audio file read error: permission denied".into())
        );
    }
}

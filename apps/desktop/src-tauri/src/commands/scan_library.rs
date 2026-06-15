use serde::{Deserialize, Serialize};

use crate::application::scan_library::{scan_library as run_scan_library, ScanError};
use crate::domain::library::LibraryId;
use crate::domain::track::{Track, TrackRepository};
use crate::AppState;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanLibraryInputDto {
    pub library_id: String,
}

#[derive(Serialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ScanReportDto {
    pub tracks_scanned: usize,
}

#[derive(Serialize, PartialEq, Eq, Debug)]
#[serde(tag = "kind", content = "message", rename_all = "camelCase")]
pub enum ScanErrorDto {
    LibraryNotFound(String),
    Walk(String),
    Metadata(String),
    Repository(String),
}

impl From<ScanError> for ScanErrorDto {
    fn from(err: ScanError) -> Self {
        match err {
            ScanError::LibraryNotFound(id) => ScanErrorDto::LibraryNotFound(id.0),
            ScanError::Walk(e) => ScanErrorDto::Walk(e.to_string()),
            ScanError::Metadata(e) => ScanErrorDto::Metadata(e.to_string()),
            ScanError::Repository(e) => ScanErrorDto::Repository(e.to_string()),
        }
    }
}

#[tauri::command]
pub fn scan_library(
    state: tauri::State<'_, AppState>,
    input: ScanLibraryInputDto,
) -> Result<ScanReportDto, ScanErrorDto> {
    let library_id = LibraryId(input.library_id);
    let report = run_scan_library(
        &library_id,
        &*state.library_repository,
        &*state.track_repository,
        &*state.walker,
        &*state.metadata_reader,
    )?;

    Ok(ScanReportDto {
        tracks_scanned: report.tracks_scanned,
    })
}

#[derive(Serialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TrackDto {
    pub id: String,
    pub library_id: String,
    pub title: String,
    pub artist: String,
    pub duration_seconds: u64,
    pub file_path: String,
}

impl From<&Track> for TrackDto {
    fn from(track: &Track) -> Self {
        TrackDto {
            id: track.id().0.clone(),
            library_id: track.library_id().0.clone(),
            title: track.title().0.clone(),
            artist: track.artist().0.clone(),
            duration_seconds: track.duration().0,
            file_path: track.file_path().0.display().to_string(),
        }
    }
}

#[derive(Serialize, PartialEq, Eq, Debug)]
#[serde(tag = "kind", content = "message", rename_all = "camelCase")]
pub enum ListTracksErrorDto {
    Repository(String),
}

#[tauri::command]
pub fn list_tracks(
    state: tauri::State<'_, AppState>,
    library_id: String,
) -> Result<Vec<TrackDto>, ListTracksErrorDto> {
    let tracks = (*state.track_repository)
        .list_for_library(&LibraryId(library_id))
        .map_err(|e| ListTracksErrorDto::Repository(e.to_string()))?;

    Ok(tracks.iter().map(TrackDto::from).collect())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::domain::library::LibraryId;
    use crate::domain::track::{
        FileMtime, FileSize, Track, TrackArtist, TrackDuration, TrackFilePath, TrackId, TrackTitle,
    };

    use super::{ScanError, ScanErrorDto, TrackDto};

    #[test]
    fn track_dto_preserves_all_fields() {
        let track = Track::new(
            TrackId("trk-1".into()),
            LibraryId("lib-1".into()),
            TrackTitle("Hotel California".into()),
            TrackArtist("Eagles".into()),
            TrackDuration(391),
            TrackFilePath(PathBuf::from("/lib/Eagles/Hotel California.mp3")),
            FileSize(9_000_000),
            FileMtime(1_700_000_000),
        );

        let dto = TrackDto::from(&track);

        assert_eq!(dto.id, "trk-1");
        assert_eq!(dto.library_id, "lib-1");
        assert_eq!(dto.title, "Hotel California");
        assert_eq!(dto.artist, "Eagles");
        assert_eq!(dto.duration_seconds, 391);
        assert_eq!(dto.file_path, "/lib/Eagles/Hotel California.mp3");
    }

    #[test]
    fn scan_error_dto_maps_library_not_found() {
        assert_eq!(
            ScanErrorDto::from(ScanError::LibraryNotFound(LibraryId("lib-1".into()))),
            ScanErrorDto::LibraryNotFound("lib-1".into())
        );
    }
}

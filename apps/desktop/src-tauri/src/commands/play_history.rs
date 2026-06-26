use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::application::play_history::{
    list_play_counts as run_list_play_counts, record_track_play as run_record_track_play,
    RecordTrackPlayInput,
};
use crate::domain::library::{LibraryId, RepositoryError};
use crate::domain::play_history::{PlayCount, PlayedAtUnixSeconds};
use crate::domain::track::TrackId;
use crate::AppState;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordTrackPlayInputDto {
    pub library_id: String,
    pub track_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListPlayCountsInputDto {
    pub library_id: String,
}

#[derive(Serialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PlayCountDto {
    pub track_id: String,
    pub play_count: u32,
    pub last_played_at_unix_seconds: Option<i64>,
}

#[derive(Serialize, PartialEq, Eq, Debug)]
#[serde(tag = "kind", content = "message", rename_all = "camelCase")]
pub enum PlayHistoryErrorDto {
    Repository(String),
}

impl From<PlayCount> for PlayCountDto {
    fn from(count: PlayCount) -> Self {
        PlayCountDto {
            track_id: count.track_id().0.clone(),
            play_count: count.count(),
            last_played_at_unix_seconds: count.last_played_at().map(|played_at| played_at.0),
        }
    }
}

impl From<RepositoryError> for PlayHistoryErrorDto {
    fn from(error: RepositoryError) -> Self {
        PlayHistoryErrorDto::Repository(error.to_string())
    }
}

#[tauri::command]
pub fn record_track_play(
    state: tauri::State<'_, AppState>,
    input: RecordTrackPlayInputDto,
) -> Result<(), PlayHistoryErrorDto> {
    run_record_track_play(
        &*state.play_history_repository,
        RecordTrackPlayInput {
            library_id: LibraryId(input.library_id),
            track_id: TrackId(input.track_id),
            played_at: PlayedAtUnixSeconds(current_unix_seconds()),
        },
    )
    .map(|_| ())
    .map_err(PlayHistoryErrorDto::from)
}

#[tauri::command]
pub fn list_track_play_counts(
    state: tauri::State<'_, AppState>,
    input: ListPlayCountsInputDto,
) -> Result<Vec<PlayCountDto>, PlayHistoryErrorDto> {
    run_list_play_counts(
        &*state.play_history_repository,
        &LibraryId(input.library_id),
    )
    .map(|counts| counts.into_iter().map(PlayCountDto::from).collect())
    .map_err(PlayHistoryErrorDto::from)
}

fn current_unix_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use crate::domain::library::{LibraryId, RepositoryError};
    use crate::domain::play_history::{PlayCount, PlayedAtUnixSeconds};
    use crate::domain::track::TrackId;

    use super::{PlayCountDto, PlayHistoryErrorDto};

    #[test]
    fn play_count_dto_preserves_track_count_and_latest_play_time() {
        let dto = PlayCountDto::from(PlayCount::new(
            LibraryId("lib-1".into()),
            TrackId("track-1".into()),
            3,
            Some(PlayedAtUnixSeconds(1_719_000_000)),
        ));

        assert_eq!(dto.track_id, "track-1");
        assert_eq!(dto.play_count, 3);
        assert_eq!(dto.last_played_at_unix_seconds, Some(1_719_000_000));
    }

    #[test]
    fn play_history_error_dto_maps_repository_errors() {
        assert_eq!(
            PlayHistoryErrorDto::from(RepositoryError::Io("database locked".into())),
            PlayHistoryErrorDto::Repository("repository io error: database locked".into())
        );
    }
}

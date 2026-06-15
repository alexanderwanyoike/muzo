use std::fmt;

use crate::domain::library::{LibraryId, RepositoryError};
use crate::domain::track::{TrackFilePath, TrackId, TrackRepository};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedTrackAudioSource {
    pub mime_type: String,
    pub file_path: TrackFilePath,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolveTrackAudioSourceError {
    TrackNotFound(TrackId),
    UnsupportedFileType(String),
    Repository(RepositoryError),
}

impl fmt::Display for ResolveTrackAudioSourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResolveTrackAudioSourceError::TrackNotFound(id) => write!(f, "track not found: {}", id),
            ResolveTrackAudioSourceError::UnsupportedFileType(path) => {
                write!(f, "unsupported audio file type: {}", path)
            }
            ResolveTrackAudioSourceError::Repository(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for ResolveTrackAudioSourceError {}

pub fn resolve_track_audio_source(
    library_id: &LibraryId,
    track_id: &TrackId,
    track_repository: &impl TrackRepository,
) -> Result<ResolvedTrackAudioSource, ResolveTrackAudioSourceError> {
    let tracks = track_repository
        .list_for_library(library_id)
        .map_err(ResolveTrackAudioSourceError::Repository)?;
    let track = tracks
        .iter()
        .find(|track| track.id() == track_id)
        .ok_or_else(|| ResolveTrackAudioSourceError::TrackNotFound(track_id.clone()))?;
    let mime_type = mime_type_for_path(track.file_path())
        .ok_or_else(|| {
            ResolveTrackAudioSourceError::UnsupportedFileType(
                track.file_path().0.display().to_string(),
            )
        })?
        .to_string();

    Ok(ResolvedTrackAudioSource {
        mime_type,
        file_path: track.file_path().clone(),
    })
}

fn mime_type_for_path(file_path: &TrackFilePath) -> Option<&'static str> {
    let extension = file_path.0.extension()?.to_str()?.to_ascii_lowercase();
    match extension.as_str() {
        "mp3" => Some("audio/mpeg"),
        "flac" => Some("audio/flac"),
        "wav" => Some("audio/wav"),
        "ogg" => Some("audio/ogg"),
        "m4a" | "aac" => Some("audio/mp4"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::domain::track::{
        FileMtime, FileSize, Track, TrackArtist, TrackDuration, TrackTitle,
    };

    use super::*;

    #[test]
    fn it_resolves_a_scanned_track_to_a_mime_type_and_file_path() {
        let library_id = LibraryId("lib-1".into());
        let track_id = TrackId("trk-1".into());
        let repository = FakeTrackRepository::with_tracks(vec![make_track(
            &library_id,
            &track_id,
            "/music/song.mp3",
        )]);

        let source = resolve_track_audio_source(&library_id, &track_id, &repository).unwrap();

        assert_eq!(source.mime_type, "audio/mpeg");
        assert_eq!(
            source.file_path,
            TrackFilePath(PathBuf::from("/music/song.mp3"))
        );
    }

    #[test]
    fn it_returns_track_not_found_when_the_track_is_not_in_the_library() {
        let library_id = LibraryId("lib-1".into());
        let track_id = TrackId("missing".into());
        let repository = FakeTrackRepository::with_tracks(vec![make_track(
            &library_id,
            &TrackId("trk-1".into()),
            "/music/song.mp3",
        )]);

        let err = resolve_track_audio_source(&library_id, &track_id, &repository).unwrap_err();

        assert_eq!(err, ResolveTrackAudioSourceError::TrackNotFound(track_id));
    }

    #[test]
    fn it_rejects_files_without_a_supported_audio_extension() {
        let library_id = LibraryId("lib-1".into());
        let track_id = TrackId("trk-1".into());
        let repository = FakeTrackRepository::with_tracks(vec![make_track(
            &library_id,
            &track_id,
            "/music/song.txt",
        )]);

        let err = resolve_track_audio_source(&library_id, &track_id, &repository).unwrap_err();

        assert_eq!(
            err,
            ResolveTrackAudioSourceError::UnsupportedFileType("/music/song.txt".into())
        );
    }

    fn make_track(library_id: &LibraryId, track_id: &TrackId, path: &str) -> Track {
        Track::new(
            track_id.clone(),
            library_id.clone(),
            TrackTitle("Song".into()),
            TrackArtist("Artist".into()),
            TrackDuration(120),
            TrackFilePath(PathBuf::from(path)),
            FileSize(4),
            FileMtime(1),
        )
    }

    struct FakeTrackRepository {
        tracks: Vec<Track>,
    }

    impl FakeTrackRepository {
        fn with_tracks(tracks: Vec<Track>) -> Self {
            Self { tracks }
        }
    }

    impl TrackRepository for FakeTrackRepository {
        fn upsert(&self, _track: &Track) -> Result<(), RepositoryError> {
            Ok(())
        }

        fn list_for_library(&self, library_id: &LibraryId) -> Result<Vec<Track>, RepositoryError> {
            Ok(self
                .tracks
                .iter()
                .filter(|track| track.library_id() == library_id)
                .cloned()
                .collect())
        }

        fn delete_by_library_and_path(
            &self,
            _library_id: &LibraryId,
            _file_path: &TrackFilePath,
        ) -> Result<(), RepositoryError> {
            Ok(())
        }
    }
}

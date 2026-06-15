use std::fmt;

use crate::domain::library::{LibraryId, RepositoryError};
use crate::domain::track::{TrackFilePath, TrackId, TrackRepository};

#[derive(Debug)]
pub struct TrackAudioSource {
    pub mime_type: String,
    pub bytes: Vec<u8>,
}

pub trait TrackAudioReader {
    fn read(&self, file_path: &TrackFilePath) -> Result<Vec<u8>, TrackAudioReadError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrackAudioReadError {
    Io(String),
}

impl fmt::Display for TrackAudioReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TrackAudioReadError::Io(msg) => write!(f, "audio file read error: {}", msg),
        }
    }
}

impl std::error::Error for TrackAudioReadError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadTrackAudioSourceError {
    TrackNotFound(TrackId),
    UnsupportedFileType(String),
    Repository(RepositoryError),
    Read(TrackAudioReadError),
}

impl fmt::Display for LoadTrackAudioSourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadTrackAudioSourceError::TrackNotFound(id) => write!(f, "track not found: {}", id),
            LoadTrackAudioSourceError::UnsupportedFileType(path) => {
                write!(f, "unsupported audio file type: {}", path)
            }
            LoadTrackAudioSourceError::Repository(err) => err.fmt(f),
            LoadTrackAudioSourceError::Read(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for LoadTrackAudioSourceError {}

pub fn load_track_audio_source(
    library_id: &LibraryId,
    track_id: &TrackId,
    track_repository: &impl TrackRepository,
    audio_reader: &impl TrackAudioReader,
) -> Result<TrackAudioSource, LoadTrackAudioSourceError> {
    let tracks = track_repository
        .list_for_library(library_id)
        .map_err(LoadTrackAudioSourceError::Repository)?;
    let track = tracks
        .iter()
        .find(|track| track.id() == track_id)
        .ok_or_else(|| LoadTrackAudioSourceError::TrackNotFound(track_id.clone()))?;
    let mime_type = mime_type_for_path(track.file_path())
        .ok_or_else(|| {
            LoadTrackAudioSourceError::UnsupportedFileType(
                track.file_path().0.display().to_string(),
            )
        })?
        .to_string();
    let bytes = audio_reader
        .read(track.file_path())
        .map_err(LoadTrackAudioSourceError::Read)?;

    Ok(TrackAudioSource { mime_type, bytes })
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
    use std::cell::RefCell;
    use std::path::PathBuf;

    use crate::domain::track::{
        FileMtime, FileSize, Track, TrackArtist, TrackDuration, TrackTitle,
    };

    use super::*;

    #[test]
    fn it_loads_audio_bytes_for_a_scanned_track() {
        let library_id = LibraryId("lib-1".into());
        let track_id = TrackId("trk-1".into());
        let track = make_track(&library_id, &track_id, "/music/song.mp3");
        let repository = FakeTrackRepository::with_tracks(vec![track]);
        let reader = FakeTrackAudioReader::with_bytes(vec![1, 2, 3, 4]);

        let source = load_track_audio_source(&library_id, &track_id, &repository, &reader).unwrap();

        assert_eq!(source.mime_type, "audio/mpeg");
        assert_eq!(source.bytes, vec![1, 2, 3, 4]);
        assert_eq!(
            reader.read_paths.borrow().as_slice(),
            &[TrackFilePath(PathBuf::from("/music/song.mp3"))]
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
        let reader = FakeTrackAudioReader::with_bytes(vec![1, 2, 3]);

        let err =
            load_track_audio_source(&library_id, &track_id, &repository, &reader).unwrap_err();

        assert_eq!(err, LoadTrackAudioSourceError::TrackNotFound(track_id));
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
        let reader = FakeTrackAudioReader::with_bytes(vec![1, 2, 3]);

        let err =
            load_track_audio_source(&library_id, &track_id, &repository, &reader).unwrap_err();

        assert_eq!(
            err,
            LoadTrackAudioSourceError::UnsupportedFileType("/music/song.txt".into())
        );
    }

    #[test]
    fn it_propagates_audio_read_errors() {
        let library_id = LibraryId("lib-1".into());
        let track_id = TrackId("trk-1".into());
        let repository = FakeTrackRepository::with_tracks(vec![make_track(
            &library_id,
            &track_id,
            "/music/song.mp3",
        )]);
        let reader = FakeTrackAudioReader::with_error("permission denied");

        let err =
            load_track_audio_source(&library_id, &track_id, &repository, &reader).unwrap_err();

        assert_eq!(
            err,
            LoadTrackAudioSourceError::Read(TrackAudioReadError::Io("permission denied".into()))
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

    struct FakeTrackAudioReader {
        bytes: Result<Vec<u8>, TrackAudioReadError>,
        read_paths: RefCell<Vec<TrackFilePath>>,
    }

    impl FakeTrackAudioReader {
        fn with_bytes(bytes: Vec<u8>) -> Self {
            Self {
                bytes: Ok(bytes),
                read_paths: RefCell::new(Vec::new()),
            }
        }

        fn with_error(message: &str) -> Self {
            Self {
                bytes: Err(TrackAudioReadError::Io(message.into())),
                read_paths: RefCell::new(Vec::new()),
            }
        }
    }

    impl TrackAudioReader for FakeTrackAudioReader {
        fn read(&self, file_path: &TrackFilePath) -> Result<Vec<u8>, TrackAudioReadError> {
            self.read_paths.borrow_mut().push(file_path.clone());
            self.bytes.clone()
        }
    }
}

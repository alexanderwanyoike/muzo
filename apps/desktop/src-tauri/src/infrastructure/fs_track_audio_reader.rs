use std::fs;

use crate::application::load_track_audio_source::{TrackAudioReadError, TrackAudioReader};
use crate::domain::track::TrackFilePath;

pub struct FsTrackAudioReader;

impl FsTrackAudioReader {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FsTrackAudioReader {
    fn default() -> Self {
        Self::new()
    }
}

impl TrackAudioReader for FsTrackAudioReader {
    fn read(&self, file_path: &TrackFilePath) -> Result<Vec<u8>, TrackAudioReadError> {
        fs::read(&file_path.0).map_err(|e| TrackAudioReadError::Io(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    #[test]
    fn it_reads_bytes_from_a_track_file_path() {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        file.write_all(&[1, 2, 3, 4]).unwrap();
        let reader = FsTrackAudioReader::new();

        let bytes = reader
            .read(&TrackFilePath(file.path().to_path_buf()))
            .unwrap();

        assert_eq!(bytes, vec![1, 2, 3, 4]);
    }

    #[test]
    fn it_returns_an_io_error_for_a_missing_track_file() {
        let reader = FsTrackAudioReader::new();

        let err = reader
            .read(&TrackFilePath("/definitely/not/here.mp3".into()))
            .unwrap_err();

        assert!(err.to_string().contains("audio file read error"));
    }
}

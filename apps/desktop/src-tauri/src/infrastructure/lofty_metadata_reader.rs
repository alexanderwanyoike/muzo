use std::path::Path;

use lofty::file::{AudioFile, TaggedFileExt};
use lofty::read_from_path;

use crate::application::scan_library::{AudioMetadata, AudioMetadataReader, MetadataError};

/// `AudioMetadataReader` backed by the `lofty` crate. Reads ID3/Vorbis/MP4
/// tags and computes duration from the audio properties.
///
/// Falls back to the file stem for the title and "Unknown Artist" when the
/// tags are missing, rather than rejecting the file. A track without tags
/// is still a track.
pub struct LoftyMetadataReader;

impl LoftyMetadataReader {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LoftyMetadataReader {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioMetadataReader for LoftyMetadataReader {
    fn read(&self, path: &Path) -> Result<AudioMetadata, MetadataError> {
        let tagged = read_from_path(path).map_err(|e| MetadataError::Io(e.to_string()))?;

        let primary_tag = tagged.primary_tag();

        let title = primary_tag
            .and_then(|tag| {
                tag.get_string(&lofty::tag::ItemKey::TrackTitle)
                    .map(str::to_string)
            })
            .or_else(|| path.file_stem().map(|s| s.to_string_lossy().into_owned()))
            .unwrap_or_default();

        let artist = primary_tag
            .and_then(|tag| {
                tag.get_string(&lofty::tag::ItemKey::TrackArtist)
                    .map(str::to_string)
            })
            .unwrap_or_else(|| "Unknown Artist".to_string());

        let duration_seconds = tagged.properties().duration().as_secs();

        Ok(AudioMetadata {
            title,
            artist,
            duration_seconds,
        })
    }
}

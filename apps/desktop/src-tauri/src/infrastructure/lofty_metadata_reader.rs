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

        let album = primary_tag
            .and_then(|tag| tag.get_string(&lofty::tag::ItemKey::AlbumTitle))
            .map(str::to_string);
        let track_number = primary_tag
            .and_then(|tag| tag.get_string(&lofty::tag::ItemKey::TrackNumber))
            .and_then(parse_slash_prefixed_u32);
        let disc_number = primary_tag
            .and_then(|tag| tag.get_string(&lofty::tag::ItemKey::DiscNumber))
            .and_then(parse_slash_prefixed_u32);
        let genre = primary_tag
            .and_then(|tag| tag.get_string(&lofty::tag::ItemKey::Genre))
            .map(str::to_string);
        let year = primary_tag
            .and_then(|tag| tag.get_string(&lofty::tag::ItemKey::Year))
            .or_else(|| {
                primary_tag.and_then(|tag| tag.get_string(&lofty::tag::ItemKey::RecordingDate))
            })
            .and_then(parse_year);

        let duration_seconds = tagged.properties().duration().as_secs();

        Ok(AudioMetadata {
            title,
            artist,
            album,
            track_number,
            disc_number,
            genre,
            year,
            duration_seconds,
        })
    }
}

fn parse_slash_prefixed_u32(value: &str) -> Option<u32> {
    value.split('/').next()?.trim().parse().ok()
}

fn parse_year(value: &str) -> Option<i32> {
    value.get(0..4)?.parse().ok()
}

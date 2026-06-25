//! Track aggregate root and related value objects.
//!
//! A [`Track`] is a single playable audio file belonging to exactly one
//! [`Library`](super::library::Library). Pure domain types - no framework
//! imports, no serde on the wire.

use std::fmt;
use std::path::PathBuf;

use super::library::{LibraryId, RepositoryError};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TrackId(pub String);

impl fmt::Display for TrackId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrackTitle(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrackArtist(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrackAlbum(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrackNumber(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiscNumber(pub u32);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrackGenre(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrackYear(pub i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrackDuration(pub u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrackFilePath(pub PathBuf);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileSize(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileMtime(pub u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrackMetadata {
    title: TrackTitle,
    artist: TrackArtist,
    album: Option<TrackAlbum>,
    track_number: Option<TrackNumber>,
    disc_number: Option<DiscNumber>,
    genre: Option<TrackGenre>,
    year: Option<TrackYear>,
}

impl TrackMetadata {
    pub fn new(
        title: TrackTitle,
        artist: TrackArtist,
        album: Option<TrackAlbum>,
        track_number: Option<TrackNumber>,
        disc_number: Option<DiscNumber>,
        genre: Option<TrackGenre>,
        year: Option<TrackYear>,
    ) -> Self {
        Self {
            title,
            artist,
            album,
            track_number,
            disc_number,
            genre,
            year,
        }
    }

    pub fn title(&self) -> &TrackTitle {
        &self.title
    }

    pub fn artist(&self) -> &TrackArtist {
        &self.artist
    }

    pub fn album(&self) -> Option<&TrackAlbum> {
        self.album.as_ref()
    }

    pub fn track_number(&self) -> Option<TrackNumber> {
        self.track_number
    }

    pub fn disc_number(&self) -> Option<DiscNumber> {
        self.disc_number
    }

    pub fn genre(&self) -> Option<&TrackGenre> {
        self.genre.as_ref()
    }

    pub fn year(&self) -> Option<TrackYear> {
        self.year
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TrackMetadataOverride {
    title: Option<TrackTitle>,
    artist: Option<TrackArtist>,
    album: Option<TrackAlbum>,
    track_number: Option<TrackNumber>,
    disc_number: Option<DiscNumber>,
    genre: Option<TrackGenre>,
    year: Option<TrackYear>,
}

impl TrackMetadataOverride {
    pub fn new(
        title: Option<TrackTitle>,
        artist: Option<TrackArtist>,
        album: Option<TrackAlbum>,
        track_number: Option<TrackNumber>,
        disc_number: Option<DiscNumber>,
        genre: Option<TrackGenre>,
        year: Option<TrackYear>,
    ) -> Self {
        Self {
            title,
            artist,
            album,
            track_number,
            disc_number,
            genre,
            year,
        }
    }

    pub fn empty() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.title.is_none()
            && self.artist.is_none()
            && self.album.is_none()
            && self.track_number.is_none()
            && self.disc_number.is_none()
            && self.genre.is_none()
            && self.year.is_none()
    }

    pub fn title(&self) -> Option<&TrackTitle> {
        self.title.as_ref()
    }

    pub fn artist(&self) -> Option<&TrackArtist> {
        self.artist.as_ref()
    }

    pub fn album(&self) -> Option<&TrackAlbum> {
        self.album.as_ref()
    }

    pub fn track_number(&self) -> Option<TrackNumber> {
        self.track_number
    }

    pub fn disc_number(&self) -> Option<DiscNumber> {
        self.disc_number
    }

    pub fn genre(&self) -> Option<&TrackGenre> {
        self.genre.as_ref()
    }

    pub fn year(&self) -> Option<TrackYear> {
        self.year
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Track {
    id: TrackId,
    library_id: LibraryId,
    file_metadata: TrackMetadata,
    metadata_override: TrackMetadataOverride,
    duration: TrackDuration,
    file_path: TrackFilePath,
    file_size: FileSize,
    file_mtime: FileMtime,
}

impl Track {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: TrackId,
        library_id: LibraryId,
        title: TrackTitle,
        artist: TrackArtist,
        duration: TrackDuration,
        file_path: TrackFilePath,
        file_size: FileSize,
        file_mtime: FileMtime,
    ) -> Self {
        Self::new_with_metadata(
            id,
            library_id,
            TrackMetadata::new(title, artist, None, None, None, None, None),
            TrackMetadataOverride::empty(),
            duration,
            file_path,
            file_size,
            file_mtime,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_with_metadata(
        id: TrackId,
        library_id: LibraryId,
        file_metadata: TrackMetadata,
        metadata_override: TrackMetadataOverride,
        duration: TrackDuration,
        file_path: TrackFilePath,
        file_size: FileSize,
        file_mtime: FileMtime,
    ) -> Self {
        Self {
            id,
            library_id,
            file_metadata,
            metadata_override,
            duration,
            file_path,
            file_size,
            file_mtime,
        }
    }

    pub fn id(&self) -> &TrackId {
        &self.id
    }

    pub fn library_id(&self) -> &LibraryId {
        &self.library_id
    }

    pub fn title(&self) -> &TrackTitle {
        self.metadata_override
            .title
            .as_ref()
            .unwrap_or_else(|| self.file_metadata.title())
    }

    pub fn artist(&self) -> &TrackArtist {
        self.metadata_override
            .artist
            .as_ref()
            .unwrap_or_else(|| self.file_metadata.artist())
    }

    pub fn album(&self) -> Option<&TrackAlbum> {
        self.metadata_override
            .album
            .as_ref()
            .or_else(|| self.file_metadata.album())
    }

    pub fn track_number(&self) -> Option<TrackNumber> {
        self.metadata_override
            .track_number
            .or_else(|| self.file_metadata.track_number())
    }

    pub fn disc_number(&self) -> Option<DiscNumber> {
        self.metadata_override
            .disc_number
            .or_else(|| self.file_metadata.disc_number())
    }

    pub fn genre(&self) -> Option<&TrackGenre> {
        self.metadata_override
            .genre
            .as_ref()
            .or_else(|| self.file_metadata.genre())
    }

    pub fn year(&self) -> Option<TrackYear> {
        self.metadata_override
            .year
            .or_else(|| self.file_metadata.year())
    }

    pub fn file_metadata(&self) -> &TrackMetadata {
        &self.file_metadata
    }

    pub fn metadata_override(&self) -> &TrackMetadataOverride {
        &self.metadata_override
    }

    pub fn metadata_overridden(&self) -> bool {
        !self.metadata_override.is_empty()
    }

    pub fn duration(&self) -> TrackDuration {
        self.duration
    }

    pub fn file_path(&self) -> &TrackFilePath {
        &self.file_path
    }

    pub fn file_size(&self) -> FileSize {
        self.file_size
    }

    pub fn file_mtime(&self) -> FileMtime {
        self.file_mtime
    }
}

pub fn generate_track_id() -> TrackId {
    TrackId(ulid::Ulid::new().to_string())
}

/// Persists tracks. Implemented by infrastructure (SQLite in sprint 01).
/// Defined in the domain so the application layer depends on the
/// abstraction, not the concrete adapter.
pub trait TrackRepository {
    /// Insert or update a track keyed on (library_id, file_path).
    fn upsert(&self, track: &Track) -> Result<(), RepositoryError>;

    /// Every track belonging to a library, in unspecified order unless the
    /// implementation documents otherwise.
    fn list_for_library(&self, library_id: &LibraryId) -> Result<Vec<Track>, RepositoryError>;

    /// Remove a track from a library by its file path. Used by sync (card 005)
    /// when a file disappears from the backing source.
    fn delete_by_library_and_path(
        &self,
        library_id: &LibraryId,
        file_path: &TrackFilePath,
    ) -> Result<(), RepositoryError>;

    /// Store Muzo-specific metadata overrides. File-derived metadata remains
    /// separate so rescans can update source tags without erasing user edits.
    fn update_metadata_override(
        &self,
        library_id: &LibraryId,
        track_id: &TrackId,
        metadata_override: &TrackMetadataOverride,
    ) -> Result<(), RepositoryError>;
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn a_track_exposes_the_values_it_was_constructed_with() {
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

        assert_eq!(track.id(), &TrackId("trk-1".into()));
        assert_eq!(track.library_id(), &LibraryId("lib-1".into()));
        assert_eq!(track.title(), &TrackTitle("Hotel California".into()));
        assert_eq!(track.artist(), &TrackArtist("Eagles".into()));
        assert_eq!(track.duration(), TrackDuration(391));
        assert_eq!(
            track.file_path(),
            &TrackFilePath(PathBuf::from("/lib/Eagles/Hotel California.mp3"))
        );
        assert_eq!(track.file_size(), FileSize(9_000_000));
        assert_eq!(track.file_mtime(), FileMtime(1_700_000_000));
    }

    #[test]
    fn user_metadata_override_takes_precedence_over_file_metadata() {
        let track = Track::new_with_metadata(
            TrackId("trk-1".into()),
            LibraryId("lib-1".into()),
            TrackMetadata::new(
                TrackTitle("File Title".into()),
                TrackArtist("File Artist".into()),
                Some(TrackAlbum("File Album".into())),
                Some(TrackNumber(1)),
                Some(DiscNumber(1)),
                Some(TrackGenre("Rock".into())),
                Some(TrackYear(1976)),
            ),
            TrackMetadataOverride::new(
                Some(TrackTitle("Edited Title".into())),
                None,
                Some(TrackAlbum("Edited Album".into())),
                None,
                None,
                None,
                None,
            ),
            TrackDuration(391),
            TrackFilePath(PathBuf::from("/lib/song.mp3")),
            FileSize(9_000_000),
            FileMtime(1_700_000_000),
        );

        assert_eq!(track.title(), &TrackTitle("Edited Title".into()));
        assert_eq!(track.artist(), &TrackArtist("File Artist".into()));
        assert_eq!(track.album(), Some(&TrackAlbum("Edited Album".into())));
        assert_eq!(track.track_number(), Some(TrackNumber(1)));
        assert!(track.metadata_overridden());
    }
}
